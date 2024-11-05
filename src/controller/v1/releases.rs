use crate::libs::CustomError;
use axum::{
    extract::Query,
    response::{self, IntoResponse},
    Json,
};
use moka::future::Cache;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, Sub},
    sync::Arc,
};
use tracing::info;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Version(semver::Version);
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
impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(VersionVisitor)
    }
}

pub struct VersionVisitor;
impl<'de> serde::de::Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a version string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        semver::Version::parse(v)
            .map(Version)
            .map_err(|e| serde::de::Error::custom(e))
    }
}

#[derive(Serialize, Clone, Deserialize)]
pub struct Release {
    version: String,
    pub_date: Option<String>,
    url: String,
    signature: String,
    notes: String,
}
impl Release {
    async fn from_release(
        release: octocrab::models::repos::Release,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let signature_url = release
            .assets
            .iter()
            .find(|asset| asset.name == "_aarch64.app.tar.gz.sig")
            .ok_or(CustomError::new("Asset(_aarch64.app.tar.gz.sig) not found"))?
            .browser_download_url
            .to_string();

        let signature = reqwest::get(signature_url).await?.text().await?;

        let url = release
            .assets
            .iter()
            .find(|asset| asset.name == "_aarch64.app.tar.gz")
            .ok_or(CustomError::new("Asset(_aarch64.app.tar.gz) not found"))?
            .browser_download_url
            .to_string();

        Ok(Self {
            version: release.tag_name[5..].to_string(),
            pub_date: release
                .published_at
                .map(|date| date.format("%+").to_string()),
            url,
            signature,
            notes: release.body.unwrap_or_default(),
        })
    }
}

async fn get_latest_release(client: &Octocrab) -> Result<Release, Box<dyn std::error::Error>> {
    let latest = client
        .repos("alinco8", "ultimate-nokori-jikan-wakaru-yaatu")
        .releases()
        .get_latest()
        .await
        .unwrap();

    Release::from_release(latest).await
}

pub async fn latest(
    client: Arc<Octocrab>,
    latest_cache: Cache<String, Release>,
) -> impl IntoResponse {
    let latest = latest_cache
        .get_with("/releases/latest".to_string(), async {
            info!("Fetching latest release");
            get_latest_release(&client).await.unwrap()
        })
        .await;

    Json(latest)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct DescriptionsQuery {
    pub start: Option<Version>,
    pub end: Option<Version>,
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
    cache: Cache<DescriptionsQuery, Vec<Release>>,
) -> impl IntoResponse {
    let Some(ref start) = query.start else {
        return format!("Bad Request: param start is required").into_response();
    };
    let Some(ref end) = query.end else {
        return format!("Bad Request: param end is required").into_response();
    };

    let repo = client.repos("alinco8", "ultimate-nokori-jikan-wakaru-yaatu");

    let c = cache
        .get_with(query.deref().clone(), async {
            info!("Fetching descriptions");
            let releases = repo
                .releases()
                .list()
                .per_page(100)
                .page(1u32)
                .send()
                .await
                .unwrap();

            let mut a = Vec::new();
            for f in releases.into_iter().filter_map(|release| {
                let version = semver::Version::parse(&release.tag_name[5..]).unwrap();
                if version >= **start && version <= **end {
                    Some(Release::from_release(release))
                } else {
                    None
                }
            }) {
                a.push(f.await.unwrap());
            }

            a
        })
        .await;

    response::Json(c).into_response()
}
