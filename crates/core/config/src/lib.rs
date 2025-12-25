use std::path::PathBuf;
use tokio::{fs, sync::OnceCell};

pub use sentry::{capture_error, capture_message, Level};
pub use sentry_anyhow::capture_anyhow;

mod schema;
pub use schema::Config;

static CONFIG: OnceCell<Config> = OnceCell::const_new();

#[inline]
fn config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("kestrel.toml")
}

/// Returns the global application config.
/// Loaded once and cached for the lifetime of the program.
pub async fn config() -> &'static Config {
    CONFIG
        .get_or_init(|| async {
            let contents = fs::read_to_string(config_path())
                .await
                .unwrap_or_else(|e| {
                    panic!("failed to read kestrel.toml: {e}");
                });

            toml::from_str(&contents)
                .unwrap_or_else(|e| {
                    panic!("invalid kestrel.toml: {e}");
                })
        })
        .await
}

/// Print the loaded configuration.
pub async fn init() {
    println!(
        ":: Kestrel Configuration ::\n\x1b[32m{:?}\x1b[0m",
        config().await
    );
}

/// Configure logging and initialize Sentry.
pub async fn setup_logging(release: &'static str, dsn: String) -> Option<sentry::ClientInitGuard> {
    let _ = std::env::var("RUST_LOG");
    let _ = std::env::var("ROCKET_ADDRESS");

    pretty_env_logger::init();
    log::info!("Starting {release}");

    if dsn.is_empty() {
        None
    } else {
        Some(sentry::init((
            dsn,
            sentry::ClientOptions {
                release: Some(release.into()),
                ..Default::default()
            },
        )))
    }
}

/// Helper macro to configure logging and Sentry.
#[macro_export]
macro_rules! configure {
    ($dsn: expr) => {
        let config = $crate::config().await;
        let _sentry = $crate::setup_logging(
            concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION")),
            $dsn.to_string(),
        )
        .await;
    };
}
