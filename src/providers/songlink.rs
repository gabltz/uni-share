use crate::error::AppError;
use crate::models::{Platform, PlatformLinks, TrackInfo};
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub struct SonglinkClient {
    http: Client,
}

impl SonglinkClient {
    pub fn new(http: Client) -> Self {
        Self { http }
    }

    pub async fn resolve(&self, music_url: &str) -> Result<TrackInfo, AppError> {
        let nested = format!("https://song.link/{}", music_url.trim());
        let response = self
            .http
            .get(&nested)
            .timeout(Duration::from_secs(25))
            .header(
                "User-Agent",
                "UNI-SHARE/0.1 (+https://github.com/uni-share)",
            )
            .send()
            .await
            .map_err(|e| AppError::Upstream(format!("song.link request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::Upstream(format!(
                "song.link returned {}",
                response.status()
            )));
        }

        let html = response
            .text()
            .await
            .map_err(|e| AppError::Upstream(format!("song.link body: {e}")))?;

        extract_track_info(&html, music_url)
            .ok_or_else(|| AppError::Upstream("song.link page missing track data".into()))
    }
}

fn extract_track_info(html: &str, source_url: &str) -> Option<TrackInfo> {
    let re = Regex::new(r#"<script id="__NEXT_DATA__" type="application/json">(.*?)</script>"#)
        .ok()?;
    let caps = re.captures(html)?;
    let data: Value = serde_json::from_str(&caps[1]).ok()?;
    let page = data.pointer("/props/pageProps/pageData")?;

    let entity = page.get("entityData");
    let mut info = TrackInfo {
        title: entity
            .and_then(|e| e.get("title"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        artist: entity
            .and_then(|e| e.get("artistName"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        isrc: entity
            .and_then(|e| e.get("isrc"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        thumbnail: entity
            .and_then(|e| e.get("thumbnailUrl"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        source_url: source_url.to_string(),
        source_platform: entity
            .and_then(|e| e.get("provider"))
            .and_then(|v| v.as_str())
            .and_then(provider_to_platform),
        links: PlatformLinks::default(),
    };

    if let Some(sections) = page.get("sections").and_then(|s| s.as_array()) {
        for section in sections {
            if let Some(links) = section.get("links").and_then(|l| l.as_array()) {
                for link in links {
                    let platform = link.get("platform").and_then(|p| p.as_str());
                    let url = link.get("url").and_then(|u| u.as_str());
                    if let (Some(platform), Some(url)) = (platform, url) {
                        if let Some(p) = songlink_platform(platform) {
                            info.links.set(p, url.to_string());
                        }
                    }
                }
            }
        }
    }

    // Also scrape anchor hrefs as a backup when JSON omits a URL.
    scrape_anchor_links(html, &mut info.links);

    Some(info)
}

fn scrape_anchor_links(html: &str, links: &mut PlatformLinks) {
    let patterns = [
        (Platform::Spotify, r#"href="(https://open\.spotify\.com/track/[^"]+)""#),
        (
            Platform::AppleMusic,
            r#"href="(https://music\.apple\.com/[^"]+)""#,
        ),
        (Platform::Deezer, r#"href="(https://www\.deezer\.com/track/[^"]+)""#),
        (
            Platform::Tidal,
            r#"href="(https://listen\.tidal\.com/track/[^"]+)""#,
        ),
    ];
    for (platform, pattern) in patterns {
        if links.get(platform).is_some() {
            continue;
        }
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(html) {
                links.set(platform, caps[1].replace("&amp;", "&"));
            }
        }
    }
}

fn songlink_platform(key: &str) -> Option<Platform> {
    match key {
        "spotify" => Some(Platform::Spotify),
        "appleMusic" | "itunes" => Some(Platform::AppleMusic),
        "deezer" => Some(Platform::Deezer),
        "tidal" => Some(Platform::Tidal),
        _ => None,
    }
}

fn provider_to_platform(provider: &str) -> Option<Platform> {
    match provider {
        "spotify" => Some(Platform::Spotify),
        "appleMusic" | "itunes" => Some(Platform::AppleMusic),
        "deezer" => Some(Platform::Deezer),
        "tidal" => Some(Platform::Tidal),
        _ => None,
    }
}
