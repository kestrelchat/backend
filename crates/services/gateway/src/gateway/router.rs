use crate::{
  handlers,
  protocol::{opcode::OpCode, packet::Packet},
};

pub async fn route(packet: Packet) {
  match packet.op {
    OpCode::Dispatch => {
      if let Some(t) = packet.t {
        handlers::dispatch::handle(t, packet.d).await;
      }
    }

    OpCode::Heartbeat => {
      handlers::heartbeat::handle().await;
    }

    OpCode::Identify => {
      handlers::identify::handle(packet.d).await;
    }
  }
}
