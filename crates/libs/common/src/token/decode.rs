use crate::{
  token::{Token, TokenType, error::TokenError, spec},
  utils::base32::base32_decode,
};

pub fn decode(input: &str) -> Result<Token, TokenError> {
  let bytes = base32_decode(input).map_err(|_| TokenError::InvalidEncoding)?;

  if bytes.len() != spec::TOTAL_BYTES {
    return Err(TokenError::InvalidLength);
  }

  let mut ts = [0u8; 8];
  ts[2..8].copy_from_slice(&bytes[0..6]);
  let timestamp = u64::from_be_bytes(ts);

  let version = bytes[6];

  if version != spec::VERSION {
    return Err(TokenError::UnsupportedVersion);
  }

  let token_type =
    TokenType::try_from(bytes[7]).map_err(|_| TokenError::UnknownType)?;

  let entropy: [u8; 16] = bytes[8..24]
    .try_into()
    .expect("slice is guaranteed to be 16 bytes");

  Ok(Token {
    timestamp,
    version,
    token_type,
    entropy,
  })
}
