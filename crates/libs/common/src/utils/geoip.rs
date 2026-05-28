use serde::Deserialize;
use std::net::IpAddr;

#[derive(Debug, Clone, Deserialize)]
pub struct GeoIpResponse {
  pub status: Option<String>,
  pub country: Option<String>,
  pub region: Option<String>,
  pub city: Option<String>,

  pub isp: Option<String>,
  pub org: Option<String>,

  pub proxy: Option<bool>,
  pub hosting: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct GeoInfo {
  pub country: Option<String>,
  pub region: Option<String>,
  pub city: Option<String>,
}

#[derive(Clone, Default)]
pub struct GeoIpClient {
  http: reqwest::Client,
}

impl GeoIpClient {
  pub async fn lookup(&self, ip: IpAddr) -> Option<GeoInfo> {
    let url = format!("{}/json/{}", "http://ip-api.com", ip);

    let resp = self.http.get(&url).send().await.ok()?;

    let bytes = resp.bytes().await.ok()?;
    let data: GeoIpResponse = serde_json::from_slice(&bytes).ok()?;

    if let Some(status) = &data.status
      && status != "success"
    {
      return None;
    }

    Some(GeoInfo {
      country: data.country,
      region: data.region,
      city: data.city,
    })
  }
}
