use std::io::{Error, ErrorKind};

use hcaptcha::Hcaptcha;

#[derive(Hcaptcha)]
pub struct HCaptchaForm<'a> {
    #[captcha]
    pub token: &'a str,
}

#[derive(serde::Deserialize)]
struct HcaptchaVerifyResponse {
    success: bool,
}

pub async fn handle_form(form: HCaptchaForm<'_>, secret_key: &str) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://api.hcaptcha.com/siteverify")
        .form(&[("secret", secret_key), ("response", form.token)])
        .send()
        .await
        .map_err(|_| Error::new(ErrorKind::ConnectionRefused, "FAILED_CAPTCHA"))?
        .json::<HcaptchaVerifyResponse>()
        .await
        .map_err(|_| Error::new(ErrorKind::ConnectionRefused, "FAILED_CAPTCHA"))?;

    if !response.success {
        return Err(Error::new(ErrorKind::ConnectionRefused, "FAILED_CAPTCHA"));
    }

    Ok(())
}
