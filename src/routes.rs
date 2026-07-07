use axum::{
    routing::{get, post},
    Router,
    response::Redirect,
    http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
    extract::{Extension, Path},
    middleware,
};

use serde::Deserialize;

use sqlx::PgPool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use url::Url;

use crate::{db::queries::{get_link_by_id, insert_link}, utils::internal_server_error};

use nanoid::nanoid;

#[derive(Deserialize)]
struct ShortenRequest {
    url: String,
}

#[derive(Clone)]
struct AuthConfig {
    api_key: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("API_KEY").unwrap_or_else(|_| "dev-secret".to_string()),
        }
    }
}

fn parse_target_url(body: &str, content_type: Option<&str>) -> Result<String, StatusCode> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    if content_type.is_some_and(|value| value.contains("application/json")) || trimmed.starts_with('{') {
        if let Ok(request) = serde_json::from_str::<ShortenRequest>(trimmed) {
            return Ok(request.url.trim().to_string());
        }

        if let Some(url) = trimmed
            .trim_matches(|c| c == '{' || c == '}')
            .trim()
            .strip_prefix("url:")
        {
            let cleaned = url.trim().trim_matches('"').trim_matches('\'').trim_end_matches(|c| c == '}');
            if !cleaned.is_empty() {
                return Ok(cleaned.to_string());
            }
        }

        return Err(StatusCode::UNPROCESSABLE_ENTITY);
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

#[derive(Default, Clone)]
struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Instant>>>,
}

impl RateLimiter {
    fn allow(&self, key: &str) -> bool {
        let mut map = self.requests.lock().unwrap();
        let now = Instant::now();
        map.retain(|_, timestamp| now.duration_since(*timestamp) < Duration::from_secs(10));
        if map.contains_key(key) {
            return false;
        }
        map.insert(key.to_string(), now);
        true
    }
}

async fn logging_middleware<B>(req: axum::http::Request<B>, next: axum::middleware::Next<B>) -> Result<axum::response::Response, StatusCode> {
    println!("request: {} {}", req.method(), req.uri());
    Ok(next.run(req).await)
}

async fn rate_limit_middleware<B>(
    req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<axum::response::Response, StatusCode> {
    let limiter = req.extensions().get::<RateLimiter>().cloned().unwrap_or_else(|| RateLimiter::default());
    let key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    if !limiter.allow(&key) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}

fn is_authorized(headers: &HeaderMap, auth_config: &AuthConfig) -> bool {
    headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|key| key == auth_config.api_key)
}

async fn auth_middleware<B>(
    req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<axum::response::Response, StatusCode> {
    let auth_config = req
        .extensions()
        .get::<AuthConfig>()
        .cloned()
        .unwrap_or_default();

    if is_authorized(req.headers(), &auth_config) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn create_routes(pool: PgPool) -> Router {
    let limiter = RateLimiter::default();
    let auth_config = AuthConfig::default();

    let protected_routes = Router::new()
        .route("/hello", get(root))
        .route("/", post(shorten))
        .layer(Extension(pool.clone()))
        .layer(Extension(limiter.clone()))
        .layer(Extension(auth_config.clone()))
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .merge(
            Router::new()
                .route("/:id", get(redirect))
                .layer(Extension(pool))
        )
        .merge(protected_routes)
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

    #[test]
    fn parses_json_like_payloads_without_quoted_keys() {
        let parsed = parse_target_url("{url:https://example.com}", Some("application/json")).unwrap();
        assert_eq!(parsed, "https://example.com");
    }

    #[test]
    fn rate_limiter_blocks_repeated_requests_from_the_same_key() {
        let limiter = super::RateLimiter::default();
        assert!(limiter.allow("127.0.0.1"));
        assert!(!limiter.allow("127.0.0.1"));
    }

    #[test]
    fn authorizes_requests_with_the_expected_api_key() {
        let auth_config = super::AuthConfig {
            api_key: "secret".to_string(),
        };
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "secret".parse().unwrap());

        assert!(super::is_authorized(&headers, &auth_config));
    }
}


