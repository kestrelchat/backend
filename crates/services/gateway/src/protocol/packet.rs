use serde::Deserialize;
use serde_json::Value;

use crate::protocol::opcode::OpCode;

#[derive(Debug, Deserialize)]
pub struct Packet {
  pub op: OpCode,
  pub t: Option<String>,
  pub d: Value,
}
