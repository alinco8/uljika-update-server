use axum::response::IntoResponse;

pub mod v1;

pub async fn index() -> impl IntoResponse {
    "Hello!"
}
