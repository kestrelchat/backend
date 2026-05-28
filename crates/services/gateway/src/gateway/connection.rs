use rocket::futures::StreamExt;
use rocket_ws::WebSocket;

use crate::{gateway::router, protocol::packet::Packet};

pub fn gateway(ws: WebSocket) -> rocket_ws::Channel<'static> {
  ws.channel(|mut stream| {
    Box::pin(async move {
      while let Some(message) = stream.next().await {
        let msg = match message {
          Ok(m) => m,
          Err(_) => break,
        };

        if let rocket_ws::Message::Text(text) = msg
          && let Ok(packet) = serde_json::from_str::<Packet>(&text)
        {
          router::route(packet).await;
        }
      }

      Ok(())
    })
  })
}
