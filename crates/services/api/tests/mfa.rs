use kestrel_common::utils::totp::TotpSetup;
use rocket::http::StatusClass;
use serde_json::{Value, json};

use crate::common::{bearer_auth, login, register_test_users, run_with_containers};

mod common;

#[rocket::async_test]
async fn mfa_login_flow() {
    run_with_containers(async |client| {
        // 1. Setup - Create a user and get a valid TOTP secret
        let user = register_test_users(&client, 1).await.pop().unwrap();
        let session = login(&client, &user);

        let totp_res = client
            .post("/auth/mfa/totp")
            .header(rocket::http::Header::new("X-Real-IP", "127.0.0.1"))
            .header(bearer_auth(&session.await.auth_token))
            .json(&json!({ "password": user.password }))
            .dispatch()
            .await;

        assert_eq!(totp_res.status().class(), StatusClass::Success);
        let totp_body: Value = totp_res.into_json().await.unwrap();
        let totp_secret = totp_body["secret"].as_str().unwrap();
        let totp = TotpSetup::from_secret_base32(totp_secret.to_string()).unwrap();

        // 2. Step 1: Initial Login Request
        let req_body = serde_json::json!({
            "email": user.email,
            "password": user.password,
            "token": "placeholder"
        });

        let response = client
            .post("/auth/login")
            .header(rocket::http::Header::new("X-Real-IP", "127.0.0.1"))
            .json(&req_body)
            .dispatch()
            .await;

        assert_eq!(
            response.status().class(),
            StatusClass::Success,
            "Initial login failed: {}",
            response.into_string().await.unwrap()
        );

        let res_body: Value = response.into_json().await.unwrap();

        // Assert that the response explicitly demands MFA verification
        assert_eq!(res_body["status"], "RequiresMfa");
        let temp_token = res_body["temp_token"].as_str().unwrap();
        assert_eq!(res_body["method"], "Totp");

        // 3. Step 2: Generate a current time-based verification code
        let code = totp.generate_current().unwrap();

        // 4. Step 3: Dispatch MFA verification challenge
        let mfa_body = serde_json::json!({
            "temp_token": temp_token,
            "code": code
        });

        let mfa_response = client
            .post("/auth/login/mfa")
            .header(rocket::http::Header::new("X-Real-IP", "127.0.0.1"))
            .json(&mfa_body)
            .dispatch()
            .await;

        assert_eq!(
            mfa_response.status().class(),
            StatusClass::Success,
            "MFA verification failed: {}",
            mfa_response.into_string().await.unwrap()
        );

        let mfa_res_body: Value = mfa_response.into_json().await.unwrap();

        // Assert that we successfully established a session and received standard tokens
        assert_eq!(mfa_res_body["status"], "Success");
        assert!(mfa_res_body["auth_token"].is_string());
        assert!(mfa_res_body["refresh_token"].is_string());
    })
    .await;
}
