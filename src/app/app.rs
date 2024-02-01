use std::env;

use time::Duration;
use tower_sessions_sqlx_store::PostgresStore;
use sqlx::{ postgres::PgPoolOptions, Pool, Postgres };
use oauth2::{ basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl };
use axum_login::{
  login_required,
  tower_sessions::{
    cookie::SameSite,
    ExpiredDeletion,
    Expiry,
    SessionManagerLayer,
  },
  AuthManagerLayerBuilder,
};

use crate::app::auth::{ auth, oauth, protected, users_file::Backend };

pub struct App {
  db: Pool<Postgres>,
  client: BasicClient,
}

impl App {
  pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let db = Self::init_db().await.expect("Database Error");
    let client = Self::init_github_client().await.expect(
      "OAuth Github Client Error"
    );

    Ok(Self { db, client })
  }

  pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
    let session_store = PostgresStore::new(self.db.clone());
    session_store.migrate().await?;

    let deletion_task = tokio::task::spawn(
      session_store
        .clone()
        .continuously_delete_expired(tokio::time::Duration::from_secs(60))
    );

    let session_layer = SessionManagerLayer::new(session_store)
      .with_secure(false)
      .with_same_site(SameSite::Lax)
      .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let backend = Backend::new(self.db, self.client);
    let auth_layer = AuthManagerLayerBuilder::new(
      backend,
      session_layer
    ).build();

    let app = protected
      ::router()
      .route_layer(login_required!(Backend, login_url = "/login"))
      .merge(auth::router())
      .merge(oauth::router())
      .layer(auth_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await?;

    deletion_task.await??;

    Ok(())
  }

  async fn init_db() -> Result<Pool<Postgres>, Box<dyn std::error::Error>> {
    let database_url = env
      ::var("DATABASE_URL")
      .expect("DATABASE_URL should be provided");
    let db = PgPoolOptions::new()
      .max_connections(5)
      .connect(&database_url).await?;

    sqlx::migrate!().run(&db).await?;

    return Ok(db);
  }

  async fn init_github_client() -> Result<
    BasicClient,
    Box<dyn std::error::Error>
  > {
    let client_id = env
      ::var("CLIENT_ID")
      .map(ClientId::new)
      .expect("CLIENT_ID should be provided.");

    let client_secret = env
      ::var("CLIENT_SECRET")
      .map(ClientSecret::new)
      .expect("CLIENT_SECRET should be provided");

    let auth_url = AuthUrl::new(
      "https://github.com/login/oauth/authorize".to_string()
    )?;
    let token_url = TokenUrl::new(
      "https://github.com/login/oauth/access_token".to_string()
    )?;

    return Ok(
      BasicClient::new(
        client_id,
        Some(client_secret),
        auth_url,
        Some(token_url)
      )
    );
  }
}
