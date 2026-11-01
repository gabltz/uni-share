use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    pub url: String,
    pub target: String,
    pub country: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConvertQuery {
    pub url: String,
    pub target: String,
    pub country: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    pub url: String,
    pub target: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub isrc: Option<String>,
    pub source_url: String,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub spotify_configured: bool,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Spotify,
    AppleMusic,
    Deezer,
    Tidal,
}

impl Platform {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "spotify" | "spot" => Some(Self::Spotify),
            "apple_music" | "applemusic" | "apple" | "am" => Some(Self::AppleMusic),
            "deezer" => Some(Self::Deezer),
            "tidal" => Some(Self::Tidal),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Spotify => "spotify",
            Self::AppleMusic => "apple_music",
            Self::Deezer => "deezer",
            Self::Tidal => "tidal",
        }
    }

}

#[derive(Debug, Clone, Default)]
pub struct TrackInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub isrc: Option<String>,
    pub thumbnail: Option<String>,
    pub source_url: String,
    pub source_platform: Option<Platform>,
    pub links: PlatformLinks,
}

#[derive(Debug, Clone, Default)]
pub struct PlatformLinks {
    pub spotify: Option<String>,
    pub apple_music: Option<String>,
    pub deezer: Option<String>,
    pub tidal: Option<String>,
}

impl PlatformLinks {
    pub fn get(&self, platform: Platform) -> Option<&str> {
        match platform {
            Platform::Spotify => self.spotify.as_deref(),
            Platform::AppleMusic => self.apple_music.as_deref(),
            Platform::Deezer => self.deezer.as_deref(),
            Platform::Tidal => self.tidal.as_deref(),
        }
    }

    pub fn set(&mut self, platform: Platform, url: String) {
        match platform {
            Platform::Spotify => self.spotify = Some(url),
            Platform::AppleMusic => self.apple_music = Some(url),
            Platform::Deezer => self.deezer = Some(url),
            Platform::Tidal => self.tidal = Some(url),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConvertResult {
    pub url: String,
    pub target: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub isrc: Option<String>,
    pub source_url: String,
    pub thumbnail: Option<String>,
}
