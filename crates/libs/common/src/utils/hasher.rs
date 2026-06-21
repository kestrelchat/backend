use std::sync::LazyLock;

use argon2::{
  Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
  Version,
  password_hash::{SaltString, rand_core::OsRng},
};
use zeroize::Zeroizing;

use crate::utils::base32::base32_encode;

/// Errors that can occur during cryptographic or hashing operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HasherError {
  /// The password hash format or parameters are invalid.
  InvalidFormat,
  /// The password does not match the provided hash.
  PasswordMismatch,
  /// An error occurred during internal cryptographic processing or parameter constraints.
  InternalError,
}

/// Hashes a password using Argon2.
///
/// Returns a PHC-encoded hash string on success.
pub async fn password_hash(
  input: Zeroizing<Vec<u8>>,
) -> Result<String, HasherError> {
  let argon2 = Argon2::default();

  let hash = tokio::task::spawn_blocking(move || {
    let salt = SaltString::generate(&mut OsRng);
    let result = argon2
      .hash_password(input.as_slice(), &salt)
      .map_err(|_| HasherError::InternalError);
    Ok(result?.to_string())
  })
  .await
  .map_err(|_| HasherError::InternalError)??
  .to_string();

  Ok(hash.to_string())
}

/// Verifies a password against a PHC-encoded hash string.
///
/// Returns `Ok` if the password matches the hash.
pub async fn password_verify(
  input: &[u8],
  digest: &str,
) -> Result<(), HasherError> {
  let parsed =
    PasswordHash::new(digest).map_err(|_| HasherError::InvalidFormat)?;
  let argon2 = Argon2::default();

  argon2.verify_password(input, &parsed).map_err(|e| match e {
    argon2::password_hash::Error::Password => HasherError::PasswordMismatch,
    _ => HasherError::InternalError,
  })
}

/// Hashes an input using BLAKE3 and returns a base32-encoded hash string.
pub fn hash(input: &[u8]) -> String {
  base32_encode(blake3::hash(input).as_bytes())
}

/// Derives a key from an input and context using BLAKE3.
pub fn derive_key(context: &str, key_material: &[u8]) -> [u8; blake3::KEY_LEN] {
  blake3::derive_key(context, key_material)
}

/// Derives a key from a password, salt, and context string using Argon2.
pub async fn password_derive_key(
  context: &str,
  password: &[u8],
  salt: &[u8],
) -> Result<[u8; 32], HasherError> {
  let params = Params::new(
    Params::DEFAULT_M_COST,
    Params::DEFAULT_T_COST,
    Params::DEFAULT_P_COST,
    Some(32),
  )
  .map_err(|_| HasherError::InvalidFormat)?;

  let argon2 = Argon2::new_with_secret(
    context.as_bytes(),
    Algorithm::default(),
    Version::default(),
    params,
  )
  .map_err(|_| HasherError::InternalError)?;

  let mut output = [0u8; 32];
  argon2
    .hash_password_into(password, salt, &mut output)
    .map_err(|_| HasherError::InternalError)?;

  Ok(output)
}

pub static DECOY_PASSWORD_HASH: LazyLock<String> = LazyLock::new(|| {
  let argon2 = Argon2::default();

  let salt = SaltString::generate(&mut OsRng);
  let hash = argon2
    .hash_password(b"decoy password", &salt)
    .expect("failed to hash decoy password");

  hash.to_string()
});
