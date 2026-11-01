# UNI-SHARE

Cross-platform music link converter. Paste or share a Spotify, Apple Music,
Deezer, or Tidal track URL and get the same recording on another service.

Built as a single Rust binary: HTTP API, landing page, and Shortcut install
docs.

## Features

- Convert track links across Spotify, Apple Music, Deezer, and Tidal
- Apple Shortcut flow via Share Sheet
- Simple landing page with Add Shortcut CTA
- Health check and CORS-enabled JSON API

## Quick start

```bash
cp .env.example .env
cargo run --release
```

Open [http://127.0.0.1:8080](http://127.0.0.1:8080).

### Docker

```bash
docker build -t uni-share .
docker run --rm -p 8080:8080 --env-file .env uni-share
```

## Environment

| Variable | Required | Description |
| --- | --- | --- |
| `PORT` | no | Listen port (default `8080`) |
| `PUBLIC_BASE_URL` | no | Public origin shown on the install page |
| `SHORTCUT_INSTALL_URL` | no | iCloud Shortcut link for one-tap Add Shortcut |
| `DEFAULT_COUNTRY` | no | Apple Music / iTunes storefront (default `US`) |
| `SPOTIFY_CLIENT_ID` | recommended | Spotify app client id |
| `SPOTIFY_CLIENT_SECRET` | recommended | Spotify app client secret |

No keys are required for Deezer or Apple Music lookups. Spotify credentials are
recommended so conversions *to* Spotify keep working when the upstream catalog
page omits a Spotify URL. Tidal matches rely on the song.link catalog page that
UNI-SHARE reads for cross-service links.

## API

### `GET /health`

```json
{
  "status": "ok",
  "service": "uni-share",
  "spotify_configured": false,
  "version": "0.1.0"
}
```

### `POST /api/v1/convert`

```bash
curl -s -X POST http://127.0.0.1:8080/api/v1/convert \
  -H 'content-type: application/json' \
  -d '{"url":"https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC","target":"deezer"}'
```

Also available as:

```bash
curl -s 'http://127.0.0.1:8080/api/v1/convert?url=URL&target=deezer'
```

**Body / query**

| Field | Type | Notes |
| --- | --- | --- |
| `url` | string | Track URL from Spotify, Apple Music, Deezer, or Tidal |
| `target` | string | `spotify` \| `apple_music` \| `deezer` \| `tidal` |
| `country` | string | Optional ISO country for Apple Music (default from env) |

**Success**

```json
{
  "url": "https://www.deezer.com/track/781592622",
  "target": "deezer",
  "title": "Never Gonna Give You Up",
  "artist": "Rick Astley",
  "isrc": "GBARL9300135",
  "source_url": "https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC",
  "thumbnail": "https://i.scdn.co/image/…"
}
```

**Errors** — JSON `{ "error": "…", "message": "…" }` with `400`, `404`, `502`,
or `503` (Spotify credentials missing when required).

### `GET /api/v1/config`

Public config for the landing page (`shortcutInstallUrl`, `publicBaseUrl`,
supported targets).

## How conversion works

1. Parse the source URL.
2. Resolve catalog metadata and sibling links via the song.link track page
   (`__NEXT_DATA__`), which still serves cross-service links even though the old
   public Odesli JSON API is closed.
3. Fill gaps with public providers:
   - Deezer API (ISRC / search)
   - iTunes Search / Lookup (Apple Music)
   - Spotify Web API when credentials are configured
4. Return the destination URL for the requested target.

## Apple Shortcut

1. Open `/shortcut/install.html` on a phone.
2. Build the Shortcut from the on-page steps (or `shortcut/blueprint.md`).
3. Point Get Contents of URL at `POST /api/v1/convert`.
4. Optionally share an iCloud link and set `SHORTCUT_INSTALL_URL` so the landing
   page Add Shortcut button installs in one tap.

iOS only imports signed Shortcuts. A Mac (or an already-published iCloud link)
is required for a distributable `.shortcut` / iCloud install URL. This repo
ships the blueprint and install guide so you can publish that link yourself.

## Deploy notes

- Run the release binary behind any HTTPS reverse proxy (Caddy, nginx, Fly,
  Railway, a VPS).
- Set `PUBLIC_BASE_URL` to the public HTTPS origin.
- Keep Spotify secrets server-side only — never put them in the Shortcut.
- Health check path for orchestrators: `/health`.

## License

MIT
