use crate::error::AppError;
use crate::models::{Platform, TrackInfo};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub struct ItunesClient {
    http: Client,
}

#[derive(Debug, Deserialize)]
struct ItunesResponse {
    results: Vec<ItunesTrack>,
}

#[derive(Debug, Deserialize)]
struct ItunesTrack {
    #[serde(rename = "trackId")]
    track_id: Option<i64>,
    #[serde(rename = "trackName")]
    track_name: Option<String>,
    #[serde(rename = "artistName")]
    artist_name: Option<String>,
    #[serde(rename = "trackViewUrl")]
    track_view_url: Option<String>,
    #[serde(rename = "artworkUrl100")]
    artwork_url_100: Option<String>,
    #[serde(rename = "isrc")]
    isrc: Option<String>,
}

impl ItunesClient {
    pub fn new(http: Client) -> Self {
        Self { http }
    }

    pub async fn by_id(&self, id: &str, country: &str) -> Result<TrackInfo, AppError> {
        let response = self
            .http
            .get("https://itunes.apple.com/lookup")
            .query(&[("id", id), ("country", country), ("entity", "song")])
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let body: ItunesResponse = response.json().await?;
        body.results
            .into_iter()
            .find(|t| t.track_id.is_some())
            .map(|t| track_to_info(t, country))
            .ok_or(AppError::TrackNotFound)
    }

    pub async fn by_isrc(&self, isrc: &str, country: &str) -> Result<Option<TrackInfo>, AppError> {
        let response = self
            .http
            .get("https://itunes.apple.com/lookup")
            .query(&[("isrc", isrc), ("country", country)])
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let body: ItunesResponse = response.json().await?;
        Ok(body
            .results
            .into_iter()
            .find(|t| t.track_view_url.is_some())
            .map(|t| track_to_info(t, country)))
    }

    pub async fn search(
        &self,
        title: &str,
        artist: &str,
        country: &str,
    ) -> Result<Option<TrackInfo>, AppError> {
        let term = format!("{title} {artist}");
        let response = self
            .http
            .get("https://itunes.apple.com/search")
            .query(&[
                ("term", term.as_str()),
                ("entity", "song"),
                ("limit", "5"),
                ("country", country),
            ])
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let body: ItunesResponse = response.json().await?;
        Ok(body
            .results
            .into_iter()
            .find(|t| t.track_view_url.is_some())
            .map(|t| track_to_info(t, country)))
    }
}

fn track_to_info(track: ItunesTrack, country: &str) -> TrackInfo {
    let apple_url = track.track_view_url.clone().map(|u| {
        // Prefer music.apple.com host when iTunes returns it; otherwise rewrite.
        if u.contains("music.apple.com") {
            u
        } else if let Some(id) = track.track_id {
            format!("https://music.apple.com/{country}/song/{id}")
        } else {
            u.replace("itunes.apple.com", "music.apple.com")
        }
    });
    let thumbnail = track
        .artwork_url_100
        .map(|u| u.replace("100x100bb", "600x600bb"));
    let mut info = TrackInfo {
        title: track.track_name,
        artist: track.artist_name,
        isrc: track.isrc,
        thumbnail,
        source_url: apple_url.clone().unwrap_or_default(),
        source_platform: Some(Platform::AppleMusic),
        links: Default::default(),
    };
    info.links.apple_music = apple_url;
    info
}
