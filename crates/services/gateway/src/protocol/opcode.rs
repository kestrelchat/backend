use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(try_from = "u8")]
pub enum OpCode {
  Dispatch,
  Heartbeat,
  Identify,
}

impl TryFrom<u8> for OpCode {
  type Error = String;
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(OpCode::Dispatch),
      1 => Ok(OpCode::Heartbeat),
      2 => Ok(OpCode::Identify),
      other => Err(format!("unknown opcode: {other}")),
      // Everything Bagels. - Stribes
    }
  }
}
