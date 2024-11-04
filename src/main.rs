mod controller;

use std::{env, net::SocketAddr, time::Duration};

use axum::{extract::Query, routing::get, Router};
use controller::v1::releases::DescriptionsQuery;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let client = octocrab::instance();
    let cache = moka::future::CacheBuilder::new(10_000)
        .time_to_live(Duration::from_secs(60))
        .build();

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

    let port: u16 = env::var("PORT")
        .unwrap_or("8000".to_string())
        .parse()
        .unwrap();

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .unwrap();

    info!("Server({}) running... ", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
