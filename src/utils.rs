use axum::http::StatusCode;

pub fn internal_server_error() -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong".to_string())
}