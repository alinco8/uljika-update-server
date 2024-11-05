mod controller;
mod libs;

use axum::{extract::Query, routing::get, Router};
use controller::v1::releases::DescriptionsQuery;
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    println!("{:?}", env::var("TOKEN"));

    let client = Arc::new(
        octocrab::OctocrabBuilder::new()
            .personal_token(env::var("TOKEN").unwrap_or("".to_string()))
            .build()
            .unwrap(),
    );
    let latest_cache = moka::future::CacheBuilder::new(10_000)
        .time_to_live(Duration::from_secs(60))
        .build();
    let descriptions_cache = moka::future::CacheBuilder::new(10_000)
        .time_to_live(Duration::from_secs(60))
        .build();

    let app = Router::new()
        .route("/", get(controller::index))
        .route(
            "/releases/latest",
            get({
                let client = client.clone();
                let cache = latest_cache.clone();
                move || controller::v1::releases::latest(client, cache)
            }),
        )
        .route(
            "/releases/descriptions",
            get({
                let client = client.clone();
                let cache = descriptions_cache.clone();
                move |query: Query<DescriptionsQuery>| {
                    controller::v1::releases::descriptions(query, client, cache)
                }
            }),
        )
        .layer(ServiceBuilder::new().layer(CorsLayer::new().allow_origin(Any)));

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
