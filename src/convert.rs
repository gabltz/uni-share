use crate::error::AppError;
use crate::models::{ConvertResult, Platform, TrackInfo};
use crate::providers::{
    deezer::DeezerClient, itunes::ItunesClient, songlink::SonglinkClient, spotify::SpotifyClient,
    urlparse,
};
use reqwest::Client;
use std::env;
use std::time::Duration;

pub struct Converter {
    songlink: SonglinkClient,
    deezer: DeezerClient,
    itunes: ItunesClient,
    spotify: Option<SpotifyClient>,
    default_country: String,
}

impl Converter {
    pub async fn from_env() -> Result<Self, AppError> {
        let http = Client::builder()
            .user_agent("UNI-SHARE/0.1")
            .timeout(Duration::from_secs(20))
            .build()?;

        let spotify = match (
            env::var("SPOTIFY_CLIENT_ID").ok().filter(|s| !s.is_empty()),
            env::var("SPOTIFY_CLIENT_SECRET")
                .ok()
                .filter(|s| !s.is_empty()),
        ) {
            (Some(id), Some(secret)) => Some(SpotifyClient::new(http.clone(), id, secret)),
            _ => None,
        };

        let default_country = env::var("DEFAULT_COUNTRY").unwrap_or_else(|_| "US".into());

        Ok(Self {
            songlink: SonglinkClient::new(http.clone()),
            deezer: DeezerClient::new(http.clone()),
            itunes: ItunesClient::new(http),
            spotify,
            default_country,
        })
    }

    pub fn spotify_configured(&self) -> bool {
        self.spotify.is_some()
    }

    pub async fn convert(
        &self,
        raw_url: &str,
        raw_target: &str,
        country: Option<&str>,
    ) -> Result<ConvertResult, AppError> {
        let url = raw_url.trim();
        if url.is_empty() {
            return Err(AppError::MissingUrl);
        }
        let target = Platform::parse(raw_target)
            .ok_or_else(|| AppError::InvalidTarget(raw_target.to_string()))?;
        let country = country.unwrap_or(&self.default_country);

        let parsed = urlparse::parse_music_url(url).ok_or(AppError::InvalidUrl)?;

        // Fast path: same platform — return canonical URL.
        if parsed.platform == target {
            let info = self.enrich_source(&parsed).await.unwrap_or(TrackInfo {
                source_url: parsed.canonical.clone(),
                source_platform: Some(parsed.platform),
                links: Default::default(),
                ..Default::default()
            });
            return Ok(ConvertResult {
                url: parsed.canonical,
                target: target.as_str().into(),
                title: info.title,
                artist: info.artist,
                isrc: info.isrc,
                source_url: url.to_string(),
                thumbnail: info.thumbnail,
            });
        }

        let mut info = self.resolve_track(url, &parsed, country).await?;

        if info.links.get(target).is_none() {
            self.fill_missing_link(&mut info, target, country).await?;
        }

        let dest = info
            .links
            .get(target)
            .map(str::to_string)
            .ok_or_else(|| {
                if target == Platform::Spotify && self.spotify.is_none() {
                    AppError::SpotifyRequired
                } else {
                    AppError::TargetMissing(target.as_str().into())
                }
            })?;

        Ok(ConvertResult {
            url: dest,
            target: target.as_str().into(),
            title: info.title,
            artist: info.artist,
            isrc: info.isrc,
            source_url: url.to_string(),
            thumbnail: info.thumbnail,
        })
    }

    async fn resolve_track(
        &self,
        raw_url: &str,
        parsed: &urlparse::ParsedMusicUrl,
        country: &str,
    ) -> Result<TrackInfo, AppError> {
        // Prefer song.link aggregated page — covers most platforms without keys.
        let mut aggregated = match self.songlink.resolve(&parsed.canonical).await {
            Ok(info) => info,
            Err(err) => {
                tracing::warn!(error = %err, "song.link resolve failed; using direct providers");
                self.enrich_source(parsed).await?
            }
        };
        aggregated.source_url = raw_url.to_string();
        if aggregated.source_platform.is_none() {
            aggregated.source_platform = Some(parsed.platform);
        }

        // Ensure we have metadata / ISRC from the native source when possible.
        if let Ok(native) = self.enrich_source(parsed).await {
            if aggregated.title.is_none() {
                aggregated.title = native.title.clone();
            }
            if aggregated.artist.is_none() {
                aggregated.artist = native.artist.clone();
            }
            if aggregated.isrc.is_none() {
                aggregated.isrc = native.isrc.clone();
            }
            if aggregated.thumbnail.is_none() {
                aggregated.thumbnail = native.thumbnail.clone();
            }
            if let Some(url) = native.links.get(parsed.platform) {
                aggregated.links.set(parsed.platform, url.to_string());
            }
        }

        // Seed Apple Music via iTunes when song.link omits it.
        if aggregated.links.apple_music.is_none() {
            if let Some(apple) = self.lookup_apple(&aggregated, country).await? {
                if let Some(url) = apple.links.apple_music {
                    aggregated.links.apple_music = Some(url);
                }
            }
        }

        Ok(aggregated)
    }

    async fn enrich_source(
        &self,
        parsed: &urlparse::ParsedMusicUrl,
    ) -> Result<TrackInfo, AppError> {
        match parsed.platform {
            Platform::Deezer => self.deezer.by_id(&parsed.id).await,
            Platform::AppleMusic => self.itunes.by_id(&parsed.id, &self.default_country).await,
            Platform::Spotify => {
                if let Some(spotify) = &self.spotify {
                    spotify.by_id(&parsed.id).await
                } else {
                    let mut links = crate::models::PlatformLinks::default();
                    links.spotify = Some(parsed.canonical.clone());
                    Ok(TrackInfo {
                        source_url: parsed.canonical.clone(),
                        source_platform: Some(Platform::Spotify),
                        links,
                        ..Default::default()
                    })
                }
            }
            Platform::Tidal => {
                let mut links = crate::models::PlatformLinks::default();
                links.tidal = Some(parsed.canonical.clone());
                Ok(TrackInfo {
                    source_url: parsed.canonical.clone(),
                    source_platform: Some(Platform::Tidal),
                    links,
                    ..Default::default()
                })
            }
        }
    }

    async fn fill_missing_link(
        &self,
        info: &mut TrackInfo,
        target: Platform,
        country: &str,
    ) -> Result<(), AppError> {
        match target {
            Platform::Deezer => {
                if let Some(found) = self.lookup_deezer(info).await? {
                    if let Some(ref url) = found.links.deezer {
                        info.links.deezer = Some(url.clone());
                    }
                    merge_meta(info, &found);
                }
            }
            Platform::AppleMusic => {
                if let Some(found) = self.lookup_apple(info, country).await? {
                    if let Some(ref url) = found.links.apple_music {
                        info.links.apple_music = Some(url.clone());
                    }
                    merge_meta(info, &found);
                }
            }
            Platform::Spotify => {
                if let Some(spotify) = &self.spotify {
                    let found = if let Some(isrc) = info.isrc.as_deref() {
                        spotify.by_isrc(isrc).await?
                    } else if let (Some(title), Some(artist)) =
                        (info.title.as_deref(), info.artist.as_deref())
                    {
                        spotify.search(title, artist).await?
                    } else {
                        None
                    };
                    if let Some(found) = found {
                        if let Some(ref url) = found.links.spotify {
                            info.links.spotify = Some(url.clone());
                        }
                        merge_meta(info, &found);
                    }
                }
            }
            Platform::Tidal => {
                // No public Tidal catalog API without credentials.
                // Rely on song.link; if missing, leave unset.
            }
        }
        Ok(())
    }

    async fn lookup_deezer(&self, info: &TrackInfo) -> Result<Option<TrackInfo>, AppError> {
        if let Some(isrc) = info.isrc.as_deref() {
            if let Some(found) = self.deezer.by_isrc(isrc).await? {
                return Ok(Some(found));
            }
        }
        if let (Some(title), Some(artist)) = (info.title.as_deref(), info.artist.as_deref()) {
            return self.deezer.search(title, artist).await;
        }
        Ok(None)
    }

    async fn lookup_apple(
        &self,
        info: &TrackInfo,
        country: &str,
    ) -> Result<Option<TrackInfo>, AppError> {
        // Try the requested storefront first, then common fallbacks (FR/US).
        let mut countries = vec![country.to_string()];
        for extra in ["FR", "US"] {
            if !countries.iter().any(|c| c.eq_ignore_ascii_case(extra)) {
                countries.push(extra.to_string());
            }
        }

        for c in &countries {
            if let Some(isrc) = info.isrc.as_deref() {
                if let Some(found) = self.itunes.by_isrc(isrc, c).await? {
                    return Ok(Some(found));
                }
            }
            if let (Some(title), Some(artist)) = (info.title.as_deref(), info.artist.as_deref()) {
                if let Some(found) = self.itunes.search(title, artist, c).await? {
                    return Ok(Some(found));
                }
            }
        }
        Ok(None)
    }
}

fn merge_meta(into: &mut TrackInfo, from: &TrackInfo) {
    if into.title.is_none() {
        into.title = from.title.clone();
    }
    if into.artist.is_none() {
        into.artist = from.artist.clone();
    }
    if into.isrc.is_none() {
        into.isrc = from.isrc.clone();
    }
    if into.thumbnail.is_none() {
        into.thumbnail = from.thumbnail.clone();
    }
}
