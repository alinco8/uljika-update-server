use axum::{extract::Query, response::IntoResponse, Json};
use moka::future::Cache;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, Sub},
    sync::Arc,
};

struct Version(semver::Version);
impl Deref for Version {
    type Target = semver::Version;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Sub for Version {
    type Output = Version;

    fn sub(self, rhs: Self) -> Self::Output {
        Version(semver::Version::new(
            self.major - rhs.major,
            self.minor - rhs.minor,
            self.patch - rhs.patch,
        ))
    }
}

#[derive(Serialize, Clone)]
pub struct LatestRelease {
    version: String,
    pub_date: Option<String>,
    url: String,
    signature: String,
    notes: String,
}

async fn get_latest_release(client: &Octocrab) -> LatestRelease {
    let raw = client
        .repos("alinco8", "ultimate-nokori-jikan-wakaru-yaatu")
        .releases()
        .get_latest()
        .await
        .unwrap();

    let signature_url = raw
        .assets
        .iter()
        .find(|asset| asset.name == "_aarch64.app.tar.gz.sig")
        .unwrap()
        .browser_download_url
        .to_string();

    let signature = reqwest::get(signature_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let url = raw
        .assets
        .iter()
        .find(|asset| asset.name == "_aarch64.app.tar.gz")
        .unwrap()
        .browser_download_url
        .to_string();

    LatestRelease {
        version: raw.tag_name[4..].to_string(),
        pub_date: raw.published_at.map(|date| date.format("%+").to_string()),
        url,
        signature,
        notes: raw.body.unwrap_or_default(),
    }
}

pub async fn latest(
    client: Arc<Octocrab>,
    moka: Cache<String, LatestRelease>,
) -> impl IntoResponse {
    Json(
        moka.get_with("/releases/latest".to_string(), async {
            get_latest_release(&client).await
        })
        .await,
    )
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DescriptionsQuery {
    pub start: Option<String>,
    pub end: Option<String>,
}
impl Default for DescriptionsQuery {
    fn default() -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}
pub async fn descriptions(
    query: Query<DescriptionsQuery>,
    client: Arc<Octocrab>,
    moka: Cache<String, LatestRelease>,
) -> impl IntoResponse {
    let Some(start) = &query.start else {
        return format!("Bad Request: param start is required").into_response();
    };
    let Some(end) = &query.end else {
        return format!("Bad Request: param end is required").into_response();
    };
    let Ok(start) = semver::Version::parse(start) else {
        return format!("Bad Request: param start is not a valid version").into_response();
    };
    let Ok(end) = semver::Version::parse(end) else {
        return format!("Bad Request: param end is not a valid version").into_response();
    };

    println!("{:?}", (Version(end) - Version(start)).deref());

    "UwU".into_response()
}
