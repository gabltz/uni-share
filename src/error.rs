use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("missing url")]
    MissingUrl,
    #[error("unsupported or invalid music url")]
    InvalidUrl,
    #[error("unsupported target platform: {0}")]
    InvalidTarget(String),
    #[error("could not identify this track")]
    TrackNotFound,
    #[error("no {0} link found for this track")]
    TargetMissing(String),
    #[error("upstream provider error: {0}")]
    Upstream(String),
    #[error("spotify credentials are required for this conversion")]
    SpotifyRequired,
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingUrl | Self::InvalidUrl | Self::InvalidTarget(_) => StatusCode::BAD_REQUEST,
            Self::TrackNotFound | Self::TargetMissing(_) => StatusCode::NOT_FOUND,
            Self::SpotifyRequired => StatusCode::SERVICE_UNAVAILABLE,
            Self::Upstream(_) | Self::Http(_) => StatusCode::BAD_GATEWAY,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingUrl => "missing_url",
            Self::InvalidUrl => "invalid_url",
            Self::InvalidTarget(_) => "invalid_target",
            Self::TrackNotFound => "track_not_found",
            Self::TargetMissing(_) => "target_missing",
            Self::SpotifyRequired => "spotify_required",
            Self::Upstream(_) | Self::Http(_) => "upstream_error",
        }
    }
}
