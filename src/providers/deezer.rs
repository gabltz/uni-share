use crate::error::AppError;
use crate::models::{Platform, TrackInfo};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct DeezerClient {
    http: Client,
}

#[derive(Debug, Deserialize)]
struct DeezerTrack {
    id: i64,
    title: Option<String>,
    isrc: Option<String>,
    link: Option<String>,
    #[serde(default)]
    artist: Option<DeezerArtist>,
    #[serde(default)]
    album: Option<DeezerAlbum>,
    error: Option<DeezerError>,
}

#[derive(Debug, Deserialize)]
struct DeezerArtist {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeezerAlbum {
    cover_xl: Option<String>,
    cover_big: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeezerError {
    code: Option<i64>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeezerSearch {
    data: Option<Vec<DeezerTrack>>,
}

impl DeezerClient {
    pub fn new(http: Client) -> Self {
        Self { http }
    }

    pub async fn by_id(&self, id: &str) -> Result<TrackInfo, AppError> {
        let url = format!("https://api.deezer.com/track/{id}");
        let track = self.get_track(&url).await?;
        Ok(track_to_info(track, None))
    }

    pub async fn by_isrc(&self, isrc: &str) -> Result<Option<TrackInfo>, AppError> {
        let url = format!("https://api.deezer.com/track/isrc:{isrc}");
        match self.get_track(&url).await {
            Ok(track) => Ok(Some(track_to_info(track, None))),
            Err(AppError::TrackNotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn search(&self, title: &str, artist: &str) -> Result<Option<TrackInfo>, AppError> {
        let q = format!("track:\"{title}\" artist:\"{artist}\"");
        let response = self
            .http
            .get("https://api.deezer.com/search/track")
            .query(&[("q", q.as_str()), ("limit", "5")])
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let body: DeezerSearch = response.json().await?;
        Ok(body
            .data
            .unwrap_or_default()
            .into_iter()
            .next()
            .map(|t| track_to_info(t, None)))
    }

    async fn get_track(&self, url: &str) -> Result<DeezerTrack, AppError> {
        let response = self
            .http
            .get(url)
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let track: DeezerTrack = response.json().await?;
        if let Some(err) = track.error {
            if err.code == Some(800) || err.message.as_deref() == Some("DataException") {
                return Err(AppError::TrackNotFound);
            }
            return Err(AppError::Upstream(format!(
                "deezer: {}",
                err.message.unwrap_or_else(|| "unknown".into())
            )));
        }
        if track.id == 0 {
            return Err(AppError::TrackNotFound);
        }
        Ok(track)
    }
}

fn track_to_info(track: DeezerTrack, source_url: Option<String>) -> TrackInfo {
    let url = track
        .link
        .unwrap_or_else(|| format!("https://www.deezer.com/track/{}", track.id));
    let thumbnail = track
        .album
        .as_ref()
        .and_then(|a| a.cover_xl.clone().or(a.cover_big.clone()));
    let mut info = TrackInfo {
        title: track.title,
        artist: track.artist.and_then(|a| a.name),
        isrc: track.isrc,
        thumbnail,
        source_url: source_url.unwrap_or_else(|| url.clone()),
        source_platform: Some(Platform::Deezer),
        links: Default::default(),
    };
    info.links.deezer = Some(url);
    info
}
