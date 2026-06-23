use base32::{Alphabet, decode, encode};

const ALPHABET: Alphabet = Alphabet::Crockford;

pub fn base32_encode(data: &[u8]) -> String {
  encode(ALPHABET, data)
}

pub fn base32_decode(input: &str) -> Result<Vec<u8>, &'static str> {
  decode(ALPHABET, input).ok_or("invalid base32 string")
}
