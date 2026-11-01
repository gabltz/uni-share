use crate::error::AppError;
use crate::models::{Platform, TrackInfo};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SpotifyClient {
    http: Client,
    client_id: String,
    client_secret: String,
    token: Arc<RwLock<Option<CachedToken>>>,
}

struct CachedToken {
    access_token: String,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
}

#[derive(Debug, Deserialize)]
struct SpotifyTrack {
    id: String,
    name: Option<String>,
    #[serde(default)]
    artists: Vec<SpotifyArtist>,
    external_ids: Option<SpotifyExternalIds>,
    external_urls: Option<SpotifyExternalUrls>,
    album: Option<SpotifyAlbum>,
}

#[derive(Debug, Deserialize)]
struct SpotifyArtist {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyExternalIds {
    isrc: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyExternalUrls {
    spotify: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyAlbum {
    images: Option<Vec<SpotifyImage>>,
}

#[derive(Debug, Deserialize)]
struct SpotifyImage {
    url: String,
}

#[derive(Debug, Deserialize)]
struct SpotifySearch {
    tracks: Option<SpotifyTrackItems>,
}

#[derive(Debug, Deserialize)]
struct SpotifyTrackItems {
    items: Vec<SpotifyTrack>,
}

impl SpotifyClient {
    pub fn new(http: Client, client_id: String, client_secret: String) -> Self {
        Self {
            http,
            client_id,
            client_secret,
            token: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn by_id(&self, id: &str) -> Result<TrackInfo, AppError> {
        let token = self.access_token().await?;
        let response = self
            .http
            .get(format!("https://api.spotify.com/v1/tracks/{id}"))
            .bearer_auth(&token)
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(AppError::TrackNotFound);
        }
        if !response.status().is_success() {
            return Err(AppError::Upstream(format!(
                "spotify track lookup failed: {}",
                response.status()
            )));
        }
        let track: SpotifyTrack = response.json().await?;
        Ok(track_to_info(track))
    }

    pub async fn by_isrc(&self, isrc: &str) -> Result<Option<TrackInfo>, AppError> {
        let token = self.access_token().await?;
        let response = self
            .http
            .get("https://api.spotify.com/v1/search")
            .bearer_auth(&token)
            .query(&[("q", format!("isrc:{isrc}")), ("type", "track".into()), ("limit", "1".into())])
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(AppError::Upstream(format!(
                "spotify isrc search failed: {}",
                response.status()
            )));
        }
        let body: SpotifySearch = response.json().await?;
        Ok(body
            .tracks
            .and_then(|t| t.items.into_iter().next())
            .map(track_to_info))
    }

    pub async fn search(&self, title: &str, artist: &str) -> Result<Option<TrackInfo>, AppError> {
        let token = self.access_token().await?;
        let q = format!("track:{title} artist:{artist}");
        let response = self
            .http
            .get("https://api.spotify.com/v1/search")
            .bearer_auth(&token)
            .query(&[("q", q.as_str()), ("type", "track"), ("limit", "1")])
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(AppError::Upstream(format!(
                "spotify search failed: {}",
                response.status()
            )));
        }
        let body: SpotifySearch = response.json().await?;
        Ok(body
            .tracks
            .and_then(|t| t.items.into_iter().next())
            .map(track_to_info))
    }

    async fn access_token(&self) -> Result<String, AppError> {
        {
            let guard = self.token.read().await;
            if let Some(cached) = guard.as_ref() {
                if cached.expires_at > Utc::now() + chrono::Duration::seconds(60) {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        let response = self
            .http
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(AppError::Upstream(format!(
                "spotify token failed: {}",
                response.status()
            )));
        }
        let body: TokenResponse = response.json().await?;
        let expires_at = Utc::now() + chrono::Duration::seconds(body.expires_in);
        let mut guard = self.token.write().await;
        *guard = Some(CachedToken {
            access_token: body.access_token.clone(),
            expires_at,
        });
        Ok(body.access_token)
    }
}

fn track_to_info(track: SpotifyTrack) -> TrackInfo {
    let url = track
        .external_urls
        .and_then(|u| u.spotify)
        .unwrap_or_else(|| format!("https://open.spotify.com/track/{}", track.id));
    let artist = track
        .artists
        .into_iter()
        .filter_map(|a| a.name)
        .collect::<Vec<_>>()
        .join(", ");
    let thumbnail = track
        .album
        .and_then(|a| a.images)
        .and_then(|imgs| imgs.into_iter().next().map(|i| i.url));
    let mut info = TrackInfo {
        title: track.name,
        artist: if artist.is_empty() { None } else { Some(artist) },
        isrc: track.external_ids.and_then(|e| e.isrc),
        thumbnail,
        source_url: url.clone(),
        source_platform: Some(Platform::Spotify),
        links: Default::default(),
    };
    info.links.spotify = Some(url);
    info
}
