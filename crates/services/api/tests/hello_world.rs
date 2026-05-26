use crate::common::run_with_containers;

mod common;

#[rocket::async_test]
async fn hello_world() {
    run_with_containers(async |_client| {
        println!("Hello, World!");
    })
    .await;
}
