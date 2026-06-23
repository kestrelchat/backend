use crate::{
  adapters::base32::base32_encode,
  models::token::{spec, types::Token},
};

pub fn encode(token: &Token) -> String {
  let mut bytes = Vec::with_capacity(spec::TOTAL_BYTES);

  let ts = token.timestamp.to_be_bytes();
  bytes.extend_from_slice(&ts[2..8]);

  bytes.push(token.version);

  bytes.push(token.token_type as u8);

  bytes.extend_from_slice(&token.entropy);

  base32_encode(&bytes)
}
