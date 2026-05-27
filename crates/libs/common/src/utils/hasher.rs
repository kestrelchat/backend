use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::utils::base32::base32_encode;

/// Hashes a password using Argon2.
///
/// Returns a PHC-encoded hash string on success, or an error if hashing fails.
pub async fn password_hash(input: &[u8]) -> Result<String, ()> {
    let argon2 = Argon2::default();

    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2.hash_password(input, &salt).map_err(|_| ())?;

    Ok(hash.to_string())
}

/// Verifies a password against a PHC-encoded hash string.
///
/// Returns `Ok` if the password matches the hash, or an error if it does not.
pub async fn password_verify(input: &[u8], digest: &str) -> Result<(), ()> {
    let parsed = PasswordHash::new(digest).map_err(|_| ())?;
    let argon2 = Argon2::default();

    argon2.verify_password(input, &parsed).map_err(|_| ())
}

/// Hashes an input using BLAKE3 and returns a base32-encoded hash string.
pub fn hash(input: &[u8]) -> String {
    base32_encode(blake3::hash(input).as_bytes())
}
