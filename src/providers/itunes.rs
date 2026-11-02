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

#[derive(Debug, Deserialize, Clone)]
struct ItunesTrack {
    #[serde(rename = "wrapperType")]
    wrapper_type: Option<String>,
    #[serde(rename = "trackId")]
    track_id: Option<i64>,
    #[serde(rename = "artistId")]
    artist_id: Option<i64>,
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

    /// Resolve Apple Music via iTunes Search. Combined title+artist queries often
    /// return zero hits even when the song exists; we also search by artist and
    /// score candidates by normalized title/artist.
    pub async fn search(
        &self,
        title: &str,
        artist: &str,
        country: &str,
    ) -> Result<Option<TrackInfo>, AppError> {
        let mut candidates: Vec<ItunesTrack> = Vec::new();

        for term in [
            format!("{title} {artist}"),
            format!("{artist} {title}"),
            title.to_string(),
        ] {
            candidates.extend(self.search_songs(&term, country, 25).await?);
        }

        for artist_id in self.search_artist_ids(artist, country).await? {
            candidates.extend(self.songs_for_artist(artist_id, country).await?);
        }

        Ok(best_match(candidates, title, artist).map(|t| track_to_info(t, country)))
    }

    async fn search_songs(
        &self,
        term: &str,
        country: &str,
        limit: u32,
    ) -> Result<Vec<ItunesTrack>, AppError> {
        let limit_s = limit.to_string();
        let response = self
            .http
            .get("https://itunes.apple.com/search")
            .query(&[
                ("term", term),
                ("entity", "song"),
                ("limit", limit_s.as_str()),
                ("country", country),
            ])
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let body: ItunesResponse = response.json().await?;
        Ok(body
            .results
            .into_iter()
            .filter(|t| t.track_id.is_some() && t.track_view_url.is_some())
            .collect())
    }

    async fn search_artist_ids(&self, artist: &str, country: &str) -> Result<Vec<i64>, AppError> {
        let response = self
            .http
            .get("https://itunes.apple.com/search")
            .query(&[
                ("term", artist),
                ("entity", "musicArtist"),
                ("limit", "8"),
                ("country", country),
            ])
            .timeout(Duration::from_secs(15))
            .send()
            .await?;
        let body: ItunesResponse = response.json().await?;
        let want = norm(artist);
        let mut ids = Vec::new();
        for track in body.results {
            // musicArtist results reuse artist_id / artist_name fields.
            let Some(id) = track.artist_id else { continue };
            let name = track.artist_name.as_deref().unwrap_or("");
            if norm(name) == want || norm(name).contains(&want) || want.contains(&norm(name)) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        // Also try attribute=artistTerm album search to discover artistIds.
        if ids.is_empty() {
            let response = self
                .http
                .get("https://itunes.apple.com/search")
                .query(&[
                    ("term", artist),
                    ("entity", "album"),
                    ("attribute", "artistTerm"),
                    ("limit", "15"),
                    ("country", country),
                ])
                .timeout(Duration::from_secs(15))
                .send()
                .await?;
            let body: ItunesResponse = response.json().await?;
            for track in body.results {
                let Some(id) = track.artist_id else { continue };
                let name = track.artist_name.as_deref().unwrap_or("");
                if norm(name) == want && !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        Ok(ids)
    }

    async fn songs_for_artist(
        &self,
        artist_id: i64,
        country: &str,
    ) -> Result<Vec<ItunesTrack>, AppError> {
        let id = artist_id.to_string();
        let response = self
            .http
            .get("https://itunes.apple.com/lookup")
            .query(&[
                ("id", id.as_str()),
                ("entity", "song"),
                ("limit", "200"),
                ("country", country),
            ])
            .timeout(Duration::from_secs(20))
            .send()
            .await?;
        let body: ItunesResponse = response.json().await?;
        Ok(body
            .results
            .into_iter()
            .filter(|t| {
                t.wrapper_type.as_deref() == Some("track")
                    && t.track_id.is_some()
                    && t.track_view_url.is_some()
            })
            .collect())
    }
}

fn track_to_info(track: ItunesTrack, country: &str) -> TrackInfo {
    let apple_url = track.track_view_url.clone().map(|u| {
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

fn best_match(candidates: Vec<ItunesTrack>, title: &str, artist: &str) -> Option<ItunesTrack> {
    let title_n = norm(title);
    let artist_n = norm(artist);
    if title_n.is_empty() {
        return None;
    }

    let mut best: Option<(i32, ItunesTrack)> = None;
    for track in candidates {
        let t = norm(track.track_name.as_deref().unwrap_or(""));
        let a = norm(track.artist_name.as_deref().unwrap_or(""));
        if t.is_empty() {
            continue;
        }

        let mut score = 0;
        if t == title_n {
            score += 100;
        } else if t.contains(&title_n) || title_n.contains(&t) {
            score += 60;
        } else {
            continue;
        }

        if !artist_n.is_empty() {
            if a == artist_n {
                score += 50;
            } else if a.contains(&artist_n) || artist_n.contains(&a) {
                score += 25;
            } else if a.split('&').any(|p| norm(p) == artist_n) {
                score += 40;
            } else {
                // Title matched but artist looks unrelated — keep weak score only
                // for exact title hits so we can still recover odd cataloging.
                if score < 100 {
                    continue;
                }
                score -= 20;
            }
        }

        match &best {
            None => best = Some((score, track)),
            Some((best_score, _)) if score > *best_score => best = Some((score, track)),
            _ => {}
        }
    }

    // Require a solid title match at minimum.
    best.and_then(|(score, track)| if score >= 100 { Some(track) } else { None })
}

fn norm(s: &str) -> String {
    let folded: String = s
        .chars()
        .map(|c| match c {
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => 'e',
            'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => 'i',
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => 'u',
            'Ç' | 'ç' => 'c',
            'Ñ' | 'ñ' => 'n',
            'Ý' | 'Ÿ' | 'ý' | 'ÿ' => 'y',
            other => other,
        })
        .flat_map(|c| c.to_lowercase())
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_basic() {
        assert_eq!(norm("  SEUIL  "), "seuil");
        assert_eq!(norm("Été"), "ete");
    }
}
