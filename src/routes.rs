use axum::{
    routing::{get, post},
    Router,
    response::Redirect,
    http::StatusCode,
    extract::{Extension, Path},
};

use serde::{Deserialize, Serialize};

use sqlx::{prelude::FromRow, PgPool};
use url::Url;

use crate::utils::internal_server_error;

use nanoid::nanoid;

#[derive(Deserialize, Serialize, FromRow)]
struct StoredURL {
    pub id: String,
    pub target_url: String,
}

pub async fn redirect(Path(id): Path<String>, Extension(pool): Extension<PgPool>) -> Result<Redirect, (StatusCode, String)> {
    let stored_url: StoredURL = sqlx::query_as("SELECT * FROM links WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Id Not Found".to_string()),
            _ => internal_server_error(),

        })?;

    Ok(Redirect::to(&stored_url.target_url))
}

pub async fn shorten(url: String, Extension(pool): Extension<PgPool>) -> Result<String, StatusCode> {

    let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_$#!@".chars().collect();
    let id = nanoid!(10, &alphabet[..]);
    
    let parsed_url = Url::parse(&url).map_err(|_err| {
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    sqlx::query("INSERT INTO links(id, target_url) VALUES ($1, $2)")
        .bind(id.clone())
        .bind(parsed_url.as_str())
        .execute(&pool)
        .await
        .map_err(|_| {
            internal_server_error().0
        })?;

    Ok(format!("http://localhost:3008/{}", id))
}

pub async fn root() -> &'static str {
    "Hello, World!"
}

pub fn create_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/hello", get(root))
        .route("/:id", get(redirect))
        .route("/", post(shorten))
        .layer(Extension(pool))
}


