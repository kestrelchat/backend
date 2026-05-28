use base64::{Engine, prelude::BASE64_URL_SAFE};
use chacha20poly1305::{
    AeadInPlace, KeyInit, XChaCha20Poly1305, XNonce,
    aead::{OsRng, rand_core::RngCore},
};
use kestrel_common::utils::{hasher, totp::TotpSetup};
use zeroize::Zeroize;

/// Errors that can occur during the protection lifecycle of a TOTP secret.
pub enum TotpSecretProtectionError {
    /// Occurs when the provided secret string cannot be decoded from base64.
    Decoding,
    /// Occurs when key derivation fails during the hashing or stretching phase.
    KeyDerivation,
    /// Occurs when authenticated encryption or decryption fails (e.g., ciphertext tampering).
    Encryption,
    /// Occurs when the decrypted secret cannot be parsed as a valid TOTP secret.
    InvalidSecret,
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
pub async fn encrypt_totp_secret(
    password: &str,
    password_hash: &str,
    totp: TotpSetup,
) -> Result<String, TotpSecretProtectionError> {
    let salt = hasher::derive_key(CONTEXT, password_hash.as_bytes());
    let mut key = hasher::password_derive_key(CONTEXT, password.as_bytes(), &salt)
        .await
        .map_err(|_| TotpSecretProtectionError::KeyDerivation)?;
    let mut secret = totp.get_secret_bytes().to_vec();
    let cipher = XChaCha20Poly1305::new(&key.into());
    key.zeroize();

    let mut nonce = XNonce::default();
    OsRng.fill_bytes(&mut nonce);

    cipher
        .encrypt_in_place(&nonce, b"", &mut secret)
        .map_err(|_| TotpSecretProtectionError::Encryption)?;

    secret.extend_from_slice(&nonce);

    Ok(BASE64_URL_SAFE.encode(secret))
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
/// * `secret_string` - A base64-encoded string containing the ciphertext payload with its trailing nonce on input.
///   On success, it is overwritten with the original plaintext secret.
pub async fn decrypt_totp_secret(
    password: &str,
    password_hash: &str,
    secret_string: String,
) -> Result<TotpSetup, TotpSecretProtectionError> {
    let mut secret = BASE64_URL_SAFE
        .decode(secret_string)
        .map_err(|_| TotpSecretProtectionError::Decoding)?;

    let salt = hasher::derive_key(CONTEXT, password_hash.as_bytes());
    let mut key = hasher::password_derive_key(CONTEXT, password.as_bytes(), &salt)
        .await
        .map_err(|_| TotpSecretProtectionError::KeyDerivation)?;
    let cipher = XChaCha20Poly1305::new(&key.into());
    key.zeroize();

    let nonce = XNonce::clone_from_slice(&secret[secret.len() - 24..]);
    secret.truncate(secret.len() - nonce.len());

    cipher
        .decrypt_in_place(&nonce, b"", &mut secret)
        .map_err(|_| TotpSecretProtectionError::Encryption)?;

    TotpSetup::from_secret_bytes(secret).map_err(|_| TotpSecretProtectionError::InvalidSecret)
}
