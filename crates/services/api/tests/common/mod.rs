use kestrel_api::web;
use kestrel_config::{
    Config,
    structs::{
        api::{ApiConfig, RegistrationConfig},
        database::DatabaseConfig,
        network::{Cors, NetworkConfig, Ports},
    },
};
use rocket::{futures::join, local::asynchronous::Client};
use testcontainers_modules::{
    postgres::Postgres,
    redis::Redis,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};

struct Containers {
    postgres: ContainerAsync<Postgres>,
    redis: ContainerAsync<Redis>,
}

impl Containers {
    async fn up() -> Self {
        let (postgres, redis) = join!(Postgres::default().start(), Redis::default().start());
        Self {
            postgres: postgres.unwrap(),
            redis: redis.unwrap(),
        }
    }

    async fn get_connections(&self) -> (String, String) {
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

        join!(postgres_url, redis_url)
    }
}

pub async fn run_with_containers(visitor: impl AsyncFn(Client)) {
    let containers = Containers::up().await;
    let (postgres_url, redis_url) = containers.get_connections().await;
    let config = Config {
        is_production: false,
        network: NetworkConfig {
            host: "127.0.0.1".to_string(),
            ports: Ports { gateway: 0, api: 0 },
            cors: Cors {
                allowed_origins: vec!["*".to_string()],
                allow_credentials: true,
            },
        },
        database: DatabaseConfig {
            postgres: postgres_url,
            redis: redis_url,
        },
        api: ApiConfig {
            registration: RegistrationConfig { minimum_age: 16 },
        },
        hcaptcha: None,
    };
    let rocket = web(Some(config)).await.unwrap().ignite().await.unwrap();
    let client = Client::tracked(rocket).await.unwrap();
    visitor(client).await;
}
