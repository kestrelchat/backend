use crate::common::{register_test_users, run_with_containers};

mod common;

#[rocket::async_test]
async fn rate_limiting() {
  run_with_containers(async |_, client| {
    let result = register_test_users(&client, 20).await;
    assert!(matches!(result, Err(err) if err.to_string().contains("Too Many Requests")));
  })
  .await;
}
