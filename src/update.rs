use std::path::PathBuf;

use iced::{
    futures::StreamExt,
    task::{Straw, sipper},
};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    browser_download_url: String,
    name: String,
}

pub async fn fetch_latest_release() -> Option<(String, String)> {
    const RELEASE_URL: &'static str =
        "https://api.github.com/repos/AbaCord/project-catalog/releases/latest";

    let client = reqwest::Client::new();

    let release: Release = client
        .get(RELEASE_URL)
        .header("User-Agent", "project-catalog")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;

    #[cfg(target_os = "windows")]
    let binary_name = "project-catalog-windows.exe";

    #[cfg(target_os = "linux")]
    let binary_name = "project-catalog-linux";

    #[cfg(target_os = "macos")]
    let binary_name = "project-catalog-macos";

    let asset = release
        .assets
        .into_iter()
        .find(|a| a.name.contains(binary_name))?;

    Some((release.tag_name, asset.browser_download_url))
}

pub fn download_stream(url: String) -> impl Straw<PathBuf, Progress, Error> {
    sipper(async move |mut progress| {
        let response = reqwest::get(&url).await?;
        let total = response.content_length().ok_or(Error::NoContentLength)?;

        let mut byte_stream = response.bytes_stream();
        let mut downloaded = 0;

        let temp_file = tempfile::NamedTempFile::new()?;
        let mut file = tokio::fs::File::from_std(temp_file.reopen()?);
        let temp_path = temp_file.into_temp_path();
        let path = temp_path.to_path_buf();

        while let Some(next_bytes) = byte_stream.next().await {
            let bytes = next_bytes?;
            downloaded += bytes.len();

            file.write_all(&bytes).await?;

            let _ = progress
                .send(Progress {
                    percent: 100.0 * downloaded as f32 / total as f32,
                })
                .await;
        }

        file.flush().await?;

        temp_path.keep()?;

        Ok(path)
    })
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub percent: f32,
}

#[derive(Debug, Clone)]
pub enum Error {
    RequestFailed,
    Io,
    Persist,
    PathPersist,
    NoContentLength,
}

impl From<reqwest::Error> for Error {
    fn from(_value: reqwest::Error) -> Self {
        Error::RequestFailed
    }
}

impl From<std::io::Error> for Error {
    fn from(_value: std::io::Error) -> Self {
        Error::Io
    }
}

impl From<tempfile::PersistError> for Error {
    fn from(_value: tempfile::PersistError) -> Self {
        Error::Persist
    }
}

impl From<tempfile::PathPersistError> for Error {
    fn from(_value: tempfile::PathPersistError) -> Self {
        Error::PathPersist
    }
}

pub async fn replace_binary(path: PathBuf) -> Result<(), std::io::Error> {
    self_replace::self_replace(&path).unwrap();
    tokio::fs::remove_file(&path).await?;
    Ok(())
}
