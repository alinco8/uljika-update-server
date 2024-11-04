mod controller;

use std::net::SocketAddr;

use axum::{extract::Query, routing::get, Router};
use controller::v1::releases::{DescriptionsQuery, LatestRelease};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let client = octocrab::instance();
    let cache = moka::future::Cache::<String, LatestRelease>::new(10_000);

    let app = Router::new()
        .route("/", get(controller::index))
        .route(
            "/releases/latest",
            get({
                let client = client.clone();
                let cache = cache.clone();
                move || controller::v1::releases::latest(client, cache)
            }),
        )
        .route(
            "/releases/descriptions",
            get({
                let client = client.clone();
                let cache = cache.clone();
                move |query: Query<DescriptionsQuery>| {
                    controller::v1::releases::descriptions(query, client, cache)
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 8000)))
        .await
        .unwrap();

    info!("Server({}) running... ", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
