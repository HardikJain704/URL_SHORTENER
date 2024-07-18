use axum::{
    routing::{get, post},
    Router,
    response::Redirect,
    http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
    extract::{Extension, Path},
};

use serde::Deserialize;

use sqlx::PgPool;
use url::Url;

use crate::{db::queries::{get_link_by_id, insert_link}, utils::internal_server_error};

use nanoid::nanoid;

#[derive(Deserialize)]
struct ShortenRequest {
    url: String,
}

fn parse_target_url(body: &str, content_type: Option<&str>) -> Result<String, StatusCode> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    if content_type.is_some_and(|value| value.contains("application/json")) || trimmed.starts_with('{') {
        let request: ShortenRequest = serde_json::from_str(trimmed)
            .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
        return Ok(request.url.trim().to_string());
    }

    Ok(trimmed.to_string())
}

pub async fn redirect(Path(id): Path<String>, Extension(pool): Extension<PgPool>) -> Result<Redirect, (StatusCode, String)> {
    let stored_url = get_link_by_id(&pool, &id)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Id Not Found".to_string()),
            _ => internal_server_error(),
        })?;

    Ok(Redirect::to(&stored_url.target_url))
}

pub async fn shorten(
    body: String,
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<String, StatusCode> {
    let content_type = headers.get(CONTENT_TYPE).and_then(|value| value.to_str().ok());
    let target_url = parse_target_url(&body, content_type)?;

    let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_$#!@".chars().collect();
    let id = nanoid!(10, &alphabet[..]);
    
    let parsed_url = Url::parse(&target_url).map_err(|_err| {
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    insert_link(&pool, &id, parsed_url.as_str())
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

#[cfg(test)]
mod tests {
    use super::parse_target_url;
    use axum::http::StatusCode;

    #[test]
    fn parses_plain_text_urls() {
        let parsed = parse_target_url("https://example.com", None).unwrap();
        assert_eq!(parsed, "https://example.com");
    }

    #[test]
    fn parses_json_requests() {
        let parsed = parse_target_url(r#"{"url":"https://example.com"}"#, Some("application/json"))
            .unwrap();
        assert_eq!(parsed, "https://example.com");
    }

    #[test]
    fn rejects_invalid_json() {
        let err = parse_target_url(r#"{"url": }"#, Some("application/json")).unwrap_err();
        assert_eq!(err, StatusCode::UNPROCESSABLE_ENTITY);
    }
}


