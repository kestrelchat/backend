use serde_json::Value;

use crate::protocol::dispatch::DispatchEvent;

pub mod test;

pub async fn handle(t: String, d: Value) {
  match DispatchEvent::from_str(&t) {
    DispatchEvent::Test => {
      test::handle(d).await;
    }

    DispatchEvent::Unknown(name) => {
      println!("unknown event: {}", name);
    }
  }
}
