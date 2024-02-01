use tracing_subscriber::{
  layer::SubscriberExt,
  util::SubscriberInitExt,
  EnvFilter,
};

use crate::app::App;

mod app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  tracing_subscriber
    ::registry()
    .with(
      EnvFilter::try_from_default_env().unwrap_or_else(|_|
        "auth-sqlx-axum=debug,axum_login=debug,tower_sessions=debug,sqlx=warn,tower_http=debug".into()
      )
    )
    .with(tracing_subscriber::fmt::layer())
    .try_init()?;

  App::new().await?.serve().await
}
