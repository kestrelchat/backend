use serde_json::Value;

pub async fn handle(payload: Value) {
  println!("identify: {:?}", payload);
}
