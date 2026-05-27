use chacha20poly1305::{
    AeadInPlace, KeyInit, XChaCha20Poly1305, XNonce,
    aead::{OsRng, rand_core::RngCore},
};
use kestrel_common::utils::hasher;

/// Errors that can occur during the protection lifecycle of a TOTP secret.
pub enum TotpSecretProtectionError {
    /// Occurs when key derivation fails during the hashing or stretching phase.
    KeyDerivation,
    /// Occurs when authenticated encryption or decryption fails (e.g., ciphertext tampering).
    Encryption,
}

/// Domain separation context string used during key derivation.
const CONTEXT: &str = "kestrel 2026-05-28 01:11:58 totp secret protection key";

/// Encrypts a TOTP secret in-place using a key derived from the user's password.
///
/// This function generates a cryptographically secure 24-byte nonce, encrypts the provided
/// `secret` buffer using XChaCha20-Poly1305, and appends the raw nonce to the end of the vector.
///
/// # Arguments
///
/// * `password` - The plain-text password used for key derivation stretching.
/// * `password_hash` - The pre-existing hash of the password, used to securely salt the derivation process.
/// * `secret` - A mutable vector containing the plaintext secret on input. On success, it contains
///   the ciphertext payload immediately followed by the 24-byte nonce.
///
/// # Errors
///
/// Returns a [`TotpSecretProtectionError`] if key derivation fails or if the AEAD encryption
/// routine encounters an unexpected error.
pub async fn encrypt_totp_secret(
    password: &str,
    password_hash: &str,
    secret: &mut Vec<u8>,
) -> Result<(), TotpSecretProtectionError> {
    let salt = hasher::derive_key(CONTEXT, password_hash.as_bytes());
    let key = hasher::password_derive_key(CONTEXT, password.as_bytes(), &salt)
        .await
        .map_err(|_| TotpSecretProtectionError::KeyDerivation)?;
    let cipher = XChaCha20Poly1305::new(&key.into());

    let mut nonce = XNonce::default();
    OsRng.fill_bytes(&mut nonce);

    cipher
        .encrypt_in_place(&nonce, b"", secret)
        .map_err(|_| TotpSecretProtectionError::Encryption)?;

    secret.extend_from_slice(&nonce);

    Ok(())
}

/// Decrypts a TOTP secret in-place using a key derived from the user's password.
///
/// This function extracts the trailing 24-byte nonce from the end of the `secret` buffer,
/// truncates the vector to isolate the pure ciphertext, and decrypts the contents in-place.
///
/// # Arguments
///
/// * `password` - The plain-text password used for key derivation stretching.
/// * `password_hash` - The pre-existing hash of the password, used to securely salt the derivation process.
/// * `secret` - A mutable vector containing the ciphertext payload with its trailing nonce on input.
///   On success, it is overwritten with the original plaintext secret.
///
/// # Errors
///
/// Returns a [`TotpSecretProtectionError`] if key derivation fails or if decryption/authentication fails
/// (indicating the data has been altered, corrupted, or the password credentials are incorrect).
pub async fn decrypt_totp_secret(
    password: &str,
    password_hash: &str,
    secret: &mut Vec<u8>,
) -> Result<(), TotpSecretProtectionError> {
    let salt = hasher::derive_key(CONTEXT, password_hash.as_bytes());
    let key = hasher::password_derive_key(CONTEXT, password.as_bytes(), &salt)
        .await
        .map_err(|_| TotpSecretProtectionError::KeyDerivation)?;
    let cipher = XChaCha20Poly1305::new(&key.into());

    let nonce = XNonce::clone_from_slice(&secret[secret.len() - 24..]);
    secret.truncate(secret.len() - nonce.len());

    cipher
        .decrypt_in_place(&nonce, b"", secret)
        .map_err(|_| TotpSecretProtectionError::Encryption)?;

    Ok(())
}
