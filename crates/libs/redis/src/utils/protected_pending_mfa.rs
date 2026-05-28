use chacha20poly1305::{
  AeadInPlace, KeyInit, XChaCha20Poly1305, XNonce,
  aead::{OsRng, rand_core::RngCore},
};
use kestrel_common::{
  models::session::PendingMfa,
  utils::{
    base32::{base32_decode, base32_encode},
    hasher::derive_key,
  },
};
use serde::{Deserialize, Serialize};

/// A protective wrapper around [`PendingMfa`] that handles in-place encryption
/// and decryption of its sensitive internal payload using XChaCha20-Poly1305.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProtectedPendingMfa {
  pending_mfa: PendingMfa,
}

/// Errors that can occur during the cryptographic transformation of a pending MFA payload.
#[derive(Debug)]
pub enum ProtectedPendingMfaError {
  /// Occurs if the underlying AEAD cipher fails to encrypt the payload.
  Encryption,
  /// Occurs if the payload is corrupted or tampering is detected during decryption.
  Decryption,
}

impl ProtectedPendingMfa {
  /// Contextual string binding used for the cryptographic key derivation function.
  const CONTEXT: &str =
    "kestrel 2026-05-28 00:11:13 pending mfa protected payload";

  /// Wraps and encrypts a [`PendingMfa`]'s protected payload in-place.
  ///
  /// The encryption key is derived safely from a unique user temporary token and a fixed context.
  /// The generated 24-byte random nonce is appended directly to the end of the encrypted payload.
  ///
  /// # Errors
  ///
  /// Returns a [`ProtectedPendingMfaError::Encryption`] if the encryption operation fails.
  pub fn new(
    mut pending_mfa: PendingMfa,
    temp_token: &str,
  ) -> Result<Self, ProtectedPendingMfaError> {
    let protection_key = derive_key(Self::CONTEXT, temp_token.as_bytes());
    let cipher = XChaCha20Poly1305::new(&protection_key.into());

    // A unique, random 24-byte nonce must be used for every execution of XChaCha20-Poly1305.
    let mut nonce = XNonce::default();
    OsRng.fill_bytes(nonce.as_mut());

    let mut protected_payload = pending_mfa.protected_payload.into_bytes();
    protected_payload.reserve(24);

    cipher
      .encrypt_in_place(&nonce, b"", &mut protected_payload)
      .map_err(|_| ProtectedPendingMfaError::Encryption)?;

    // Append the nonce to the payload so it can be recovered during decryption.
    protected_payload.extend_from_slice(nonce.as_ref());

    pending_mfa.protected_payload = base32_encode(&protected_payload);

    Ok(Self { pending_mfa })
  }

  /// Consumes the wrapper, decrypts the internal payload in-place, and returns the underlying [`PendingMfa`].
  ///
  /// The nonce is extracted from the tail of the encrypted payload before executing the cipher.
  ///
  /// # Errors
  ///
  /// Returns a [`ProtectedPendingMfaError::Decryption`] if verification or decryption fails.
  pub fn decrypt(
    mut self,
    temp_token: &str,
  ) -> Result<PendingMfa, ProtectedPendingMfaError> {
    let protection_key = derive_key(Self::CONTEXT, temp_token.as_bytes());
    let cipher = XChaCha20Poly1305::new(&protection_key.into());

    let mut protected_payload =
      base32_decode(&self.pending_mfa.protected_payload)
        .map_err(|_| ProtectedPendingMfaError::Decryption)?;

    // Safety guard: Ensure the buffer contains at least the 24 bytes of the XNonce tail.
    if protected_payload.len() < 24 {
      return Err(ProtectedPendingMfaError::Decryption);
    }

    // Extract the 24-byte XNonce from the end of the buffer where it was written by `new()`.
    let nonce_index = protected_payload.len() - 24;
    let nonce = XNonce::clone_from_slice(&protected_payload[nonce_index..]);
    protected_payload.truncate(nonce_index);

    cipher
      .decrypt_in_place(&nonce, b"", &mut protected_payload)
      .map_err(|_| ProtectedPendingMfaError::Decryption)?;

    self.pending_mfa.protected_payload =
      String::from_utf8(protected_payload)
        .map_err(|_| ProtectedPendingMfaError::Decryption)?;

    Ok(self.pending_mfa)
  }
}
