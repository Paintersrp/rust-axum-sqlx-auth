use askama::Template;
use axum::{ http::StatusCode, response::IntoResponse, routing::get, Router };

#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
  username: &'a str,
}

pub fn router() -> Router<()> {
  Router::new().route("/", get(self::get::protected))
}

mod get {
  use crate::app::auth::users_file::AuthSession;

  use super::*;

  pub async fn protected(auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.user {
      Some(user) =>
        (ProtectedTemplate {
          username: &user.username,
        }).into_response(),

      None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
  }
}
