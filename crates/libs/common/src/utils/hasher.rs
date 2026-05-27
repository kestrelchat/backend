use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

pub async fn password_hash(input: &[u8]) -> Result<String, ()> {
    let argon2 = Argon2::default();

    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2.hash_password(input, &salt).map_err(|_| ())?;

    Ok(hash.to_string())
}

pub async fn password_verify(input: &[u8], digest: &str) -> Result<(), ()> {
    let parsed = PasswordHash::new(digest).map_err(|_| ())?;
    let argon2 = Argon2::default();

    argon2.verify_password(input, &parsed).map_err(|_| ())
}

pub fn hash(input: &[u8]) -> String {
    blake3::hash(input).to_string()
}
