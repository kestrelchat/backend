use argon2::password_hash::rand_core::{OsRng, RngCore};
use totp_rs::{Algorithm, Secret, TOTP};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// URL encoded name of the provider
const ISSUER: &str = "Kestrel";

/// A wrapper around [`TOTP`] with helper methods
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct TotpSetup(TOTP);

/// Errors that can occur when using [`TotpSetup`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TotpError {
    InvalidSecret,
    InvalidCode,
    TimeUnavailable,
}

impl TotpSetup {
    /// Generates a new TOTP secret and wraps it
    pub fn generate() -> Self {
        let mut buffer = vec![0; 16];
        OsRng.fill_bytes(&mut buffer);
        TotpSetup::from_secret_bytes(buffer).expect("broken config")
    }

    /// Constructs TOTP configuration from a secret
    ///
    /// `secret` must be at least 128 bits long
    pub fn from_secret_bytes(secret: Vec<u8>) -> Result<Self, TotpError> {
        let inner =
            TOTP::new(Algorithm::SHA1, 6, 1, 30, secret).map_err(|_| TotpError::InvalidSecret)?;
        Ok(Self(inner))
    }

    /// Reconstructs TOTP configuration from base32 secret
    pub fn from_secret_base32(secret: String) -> Result<Self, TotpError> {
        let secret = Secret::Encoded(secret)
            .to_bytes()
            .map_err(|_| TotpError::InvalidSecret)?;
        Self::from_secret_bytes(secret)
    }

    /// Will return the base32 representation of the secret
    ///
    /// Wrapper for [`TOTP::get_secret_base32`]
    pub fn get_secret_base32(&self) -> String {
        self.0.get_secret_base32()
    }

    /// Returns the raw secret bytes of the TOTP configuration
    pub fn get_secret_bytes(&self) -> &[u8] {
        &self.0.secret
    }

    /// Generates a current time-based verification code
    ///
    /// Wrapper for [`TOTP::generate_current`]
    pub fn generate_current(&self) -> Result<String, TotpError> {
        self.0
            .generate_current()
            .map_err(|_| TotpError::InvalidCode)
    }

    /// Will check if token is valid by current system time, accounting skew
    ///
    /// Wrapper for [`TOTP::check_current`]
    pub fn verify(&self, token: &str) -> Result<(), TotpError> {
        let success = self
            .0
            .check_current(token)
            .map_err(|_| TotpError::InvalidCode)?;
        success.then_some(()).ok_or(TotpError::InvalidCode)
    }

    /// Serializes to an URI format compatible with Google Authenticator
    ///
    /// For more info, see: [Key-Uri-Format](<https://github.com/google/google-authenticator/wiki/Key-Uri-Format/f1e21a2dc3b46cae34f6d5a0e6d69e425fc2bd31>)
    pub fn build_uri(&self, account_name: String) -> String {
        let label = format!("{ISSUER}:{account_name}");
        let parameters = format!("secret={}&issuer={}", self.0.get_secret_base32(), ISSUER);
        format!("otpauth://totp/{label}?{parameters}")
    }
}
