use rand::TryRng;
use rand::rngs::SysRng;

use crate::token::encode::encode;
use crate::token::spec::VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TokenType {
  Refresh = 1,
  Auth = 2,
  PendingMfa = 3,
  PasswordReset = 4,
}

impl TryFrom<u8> for TokenType {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(Self::Refresh),
      2 => Ok(Self::Auth),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Token {
  pub version: u8,
  pub timestamp: u64,
  pub token_type: TokenType,
  pub entropy: [u8; 16],
}

impl Token {
  fn new(token_type: TokenType) -> Self {
    Self {
      version: VERSION,
      timestamp: current_time_millis(),
      token_type,
      entropy: secure_random_16(),
    }
  }

  pub fn generate(token_type: TokenType) -> String {
    let token = Self::new(token_type);
    encode(&token)
  }

  pub fn is_expired(&self, ttl_ms: u64) -> bool {
    let now = current_time_millis();
    now.saturating_sub(self.timestamp) > ttl_ms
  }
}

fn current_time_millis() -> u64 {
  use std::time::{SystemTime, UNIX_EPOCH};

  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock went backwards")
    .as_millis() as u64
}

fn secure_random_16() -> [u8; 16] {
  let mut buf = [0u8; 16];
  SysRng.try_fill_bytes(&mut buf).expect("OS RNG failed");
  buf
}
