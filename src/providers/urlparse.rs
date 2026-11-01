use crate::models::Platform;
use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

static SPOTIFY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:open\.)?spotify\.com/(?:intl-[a-z]{2}/)?track/([A-Za-z0-9]+)").unwrap()
});
static APPLE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"music\.apple\.com/[a-z]{2}/(?:album|song)/[^/]+/(?:.+\?i=)?(\d+)").unwrap()
});
static APPLE_SONG_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"music\.apple\.com/[a-z]{2}/song/[^/]+/(\d+)").unwrap());
static DEEZER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:www\.)?deezer\.com/(?:[a-z]{2}/)?track/(\d+)").unwrap());
static TIDAL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:listen\.)?tidal\.com/(?:browse/)?track/(\d+)|tidal\.com/track/(\d+)").unwrap()
});

#[derive(Debug, Clone)]
pub struct ParsedMusicUrl {
    pub platform: Platform,
    pub id: String,
    pub canonical: String,
}

pub fn parse_music_url(raw: &str) -> Option<ParsedMusicUrl> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else if trimmed.starts_with("spotify:track:") {
        let id = trimmed.trim_start_matches("spotify:track:");
        return Some(ParsedMusicUrl {
            platform: Platform::Spotify,
            id: id.to_string(),
            canonical: format!("https://open.spotify.com/track/{id}"),
        });
    } else {
        format!("https://{trimmed}")
    };

    let url = Url::parse(&with_scheme).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    let full = url.as_str().to_string();

    if host.contains("spotify.com") {
        if let Some(c) = SPOTIFY_RE.captures(&full) {
            let id = c[1].to_string();
            return Some(ParsedMusicUrl {
                platform: Platform::Spotify,
                id: id.clone(),
                canonical: format!("https://open.spotify.com/track/{id}"),
            });
        }
    }

    if host.contains("music.apple.com") || host.contains("itunes.apple.com") {
        if let Some(c) = APPLE_SONG_RE.captures(&full) {
            let id = c[1].to_string();
            return Some(ParsedMusicUrl {
                platform: Platform::AppleMusic,
                id,
                canonical: full,
            });
        }
        if let Some(c) = APPLE_RE.captures(&full) {
            let id = c[1].to_string();
            return Some(ParsedMusicUrl {
                platform: Platform::AppleMusic,
                id,
                canonical: full,
            });
        }
        // album URL with ?i=trackId
        if let Some(id) = url
            .query_pairs()
            .find(|(k, _)| k == "i")
            .map(|(_, v)| v.to_string())
        {
            return Some(ParsedMusicUrl {
                platform: Platform::AppleMusic,
                id,
                canonical: full,
            });
        }
    }

    if host.contains("deezer.com") {
        if let Some(c) = DEEZER_RE.captures(&full) {
            let id = c[1].to_string();
            return Some(ParsedMusicUrl {
                platform: Platform::Deezer,
                id: id.clone(),
                canonical: format!("https://www.deezer.com/track/{id}"),
            });
        }
    }

    if host.contains("tidal.com") {
        if let Some(c) = TIDAL_RE.captures(&full) {
            let id = c
                .get(1)
                .or_else(|| c.get(2))
                .map(|m| m.as_str().to_string())?;
            return Some(ParsedMusicUrl {
                platform: Platform::Tidal,
                id: id.clone(),
                canonical: format!("https://listen.tidal.com/track/{id}"),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spotify() {
        let p = parse_music_url("https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC?si=abc")
            .unwrap();
        assert_eq!(p.platform, Platform::Spotify);
        assert_eq!(p.id, "4uLU6hMCjMI75M1A2tKUQC");
    }

    #[test]
    fn parses_deezer() {
        let p = parse_music_url("https://www.deezer.com/track/781592622").unwrap();
        assert_eq!(p.platform, Platform::Deezer);
    }

    #[test]
    fn parses_tidal() {
        let p = parse_music_url("https://listen.tidal.com/track/491206012").unwrap();
        assert_eq!(p.platform, Platform::Tidal);
    }
}
