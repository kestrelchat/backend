#![allow(dead_code)]

use std::{net::IpAddr, sync::Arc};

use chrono::{DateTime, Timelike, Utc};
use kestrel_config::{
  Config,
  structs::{
    database::{DatabaseConfig, PostgresConfig, RedisConfig},
    features::FeatureConfig,
    instance::InstanceConfig,
    server::{CorsConfig, ServerConfig},
  },
};
use kestrel_server::web;
use rocket::{
  futures::join,
  http::{Header, StatusClass},
  local::asynchronous::{Client, LocalRequest},
  tokio::task::JoinSet,
};
use serde_json::{Value, json};
use testcontainers_modules::{
  postgres::Postgres,
  redis::Redis,
  testcontainers::{ContainerAsync, runners::AsyncRunner},
};

pub struct Containers {
  pub postgres: ContainerAsync<Postgres>,
  pub redis: ContainerAsync<Redis>,
}

pub struct ContainerUrls {
  pub postgres: String,
  pub redis: String,
}

impl Containers {
  async fn up() -> Self {
    let (postgres, redis) =
      join!(Postgres::default().start(), Redis::default().start());
    Self {
      postgres: postgres.unwrap(),
      redis: redis.unwrap(),
    }
  }

  pub async fn get_urls(&self) -> ContainerUrls {
    let postgres_url = async {
      let host = self.postgres.get_host().await.unwrap();
      let port = self.postgres.get_host_port_ipv4(5432).await.unwrap();
      format!("postgres://postgres:postgres@{}:{}/postgres", host, port)
    };

    let redis_url = async {
      let host = self.redis.get_host().await.unwrap();
      let port = self.redis.get_host_port_ipv4(6379).await.unwrap();
      format!("redis://{}:{}", host, port)
    };

    let (postgres_url, redis_url) = join!(postgres_url, redis_url);

    ContainerUrls {
      postgres: postgres_url,
      redis: redis_url,
    }
  }
}

pub struct TestClient {
  inner: Client,
  default_ip: IpAddr,
}

impl TestClient {
  pub fn new(client: Client, default_ip: IpAddr) -> Self {
    Self {
      inner: client,
      default_ip: default_ip.to_canonical(),
    }
  }

  pub fn get<'c, 'u: 'c>(&'c self, uri: &'u str) -> LocalRequest<'c> {
    self.inner.get(uri).header(rocket::http::Header::new(
      "X-Real-IP",
      self.default_ip.to_string(),
    ))
  }

  pub fn post<'c, 'u: 'c>(&'c self, uri: &'u str) -> LocalRequest<'c> {
    self.inner.post(uri).header(rocket::http::Header::new(
      "X-Real-IP",
      self.default_ip.to_string(),
    ))
  }

  pub fn put<'c, 'u: 'c>(&'c self, uri: &'u str) -> LocalRequest<'c> {
    self.inner.put(uri).header(rocket::http::Header::new(
      "X-Real-IP",
      self.default_ip.to_string(),
    ))
  }

  pub fn delete<'c, 'u: 'c>(&'c self, uri: &'u str) -> LocalRequest<'c> {
    self.inner.delete(uri).header(rocket::http::Header::new(
      "X-Real-IP",
      self.default_ip.to_string(),
    ))
  }
}

pub async fn run_with_containers(
  visitor: impl AsyncFn(Containers, Arc<TestClient>),
) {
  let containers = Containers::up().await;
  let container_urls = containers.get_urls().await;
  let config = Config {
    is_production: false,
    instance: InstanceConfig {
      name: "Kestrel Test".to_string(),
      domain: "kestrel.local".to_string(),
      description: None,
    },
    server: ServerConfig {
      host: "127.0.0.1".to_string(),
      port: 5178,
      cors: CorsConfig {
        allowed_origins: vec!["*".to_string()],
        allow_credentials: true,
      },
    },
    database: DatabaseConfig {
      postgres: PostgresConfig {
        url: container_urls.postgres,
      },
      redis: RedisConfig {
        url: container_urls.redis,
      },
    },
    features: FeatureConfig::default(),
  };
  let rocket = web(Some(config)).await.unwrap().ignite().await.unwrap();
  let client = Client::tracked(rocket).await.unwrap();
  visitor(
    containers,
    Arc::new(TestClient::new(client, IpAddr::from([127, 0, 0, 1]))),
  )
  .await;
}

/// Credentials used for authentication and testing purposes.
#[derive(Debug, Clone)]
pub struct UserCredentials {
  pub email: String,
  pub username: String,
  pub password: String,
}

pub async fn register_test_users(
  client: &Arc<TestClient>,
  count: usize,
) -> Result<Vec<UserCredentials>, Box<dyn std::error::Error + Send + Sync>> {
  let time = DateTime::<Utc>::default().nanosecond();
  let mut join_set = JoinSet::new();
  for i in 0..count {
    let client = client.clone();
    join_set.spawn(async move {
      let username = format!("{time}_{i}");
      let password = "loremIpsum123!".to_string();
      let email = format!("{username}@example.com");

      let body = json!({
        "email": email.clone(),
        "username": username.clone(),
        "password": password.clone(),
        "birthday": "2005-03-12".to_string(),
      });
      let client = client.clone();
      let res = client.post("/auth/register").json(&body).dispatch().await;
      if res.status().class() != StatusClass::Success {
        return Err(Box::new(std::io::Error::other(format!(
          "HTTP Error: {}",
          res.status()
        ))) as Box<dyn std::error::Error + Send + Sync>);
      }
      Ok(UserCredentials {
        email,
        username,
        password,
      })
    });
  }

  join_set
    .join_all()
    .await
    .into_iter()
    .try_fold(vec![], |mut acc, res| {
      acc.push(res?);
      Ok(acc)
    })
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TokenPair {
  pub auth_token: String,
  pub refresh_token: String,
}

pub async fn login(
  client: &Arc<TestClient>,
  user: &UserCredentials,
) -> TokenPair {
  let req_body = json!({
    "email": user.email,
    "password": user.password,
    "token": ""
  });
  let res = client
    .post("/auth/login")
    .header(Header::new("X-Real-IP", "127.0.0.1"))
    .json(&req_body)
    .dispatch()
    .await;
  assert_eq!(
    res.status().class(),
    StatusClass::Success,
    "login failed: {}",
    res.into_string().await.unwrap()
  );
  let res_body = res.into_json::<Value>().await.unwrap();
  TokenPair {
    auth_token: res_body["auth_token"].as_str().unwrap().to_string(),
    refresh_token: res_body["refresh_token"].as_str().unwrap().to_string(),
  }
}

pub fn bearer_auth(token: &str) -> Header<'static> {
  Header::new("Authorization", format!("Bearer {}", token))
}
