use serde_json::Value;

pub async fn handle(data: Value) {
  println!("test event: {:?}", data);
}
