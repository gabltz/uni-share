# UNI-SHARE

Cross-platform music link converter. Paste or share a Spotify, Apple Music,
Deezer, or Tidal track URL and get the same recording on another service.

Built as a single Rust binary: HTTP API, landing page, and Shortcut install
docs. The same `static/` tree can also be published to GitHub Pages while the
API runs on a separate always-on host (Fly.io).

## Features

- Convert track links across Spotify, Apple Music, Deezer, and Tidal
- Apple Shortcut flow via Share Sheet
- Landing page with Add Shortcut CTA
- Health check and CORS-enabled JSON API
- Optional split: static site on GitHub Pages + API on Fly.io

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
| `PUBLIC_BASE_URL` | recommended | Public HTTPS origin of the **API** (no trailing slash) |
| `SHORTCUT_INSTALL_URL` | no | iCloud Shortcut link for one-tap Add Shortcut |
| `DEFAULT_COUNTRY` | no | Apple Music / iTunes storefront (default `US`) |
| `SPOTIFY_CLIENT_ID` | recommended | Spotify app client id |
| `SPOTIFY_CLIENT_SECRET` | recommended | Spotify app client secret |

Put secrets in `.env` locally or in the host secret UI. Never commit them.

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
| `country` | string | optional storefront / market override |

**Success fields:** `url`, `target`, `title`, `artist`, `isrc`, `source_url`, `thumbnail`

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

1. Open `shortcut/install.html` on a phone (from the API host or GitHub Pages).
2. Build the Shortcut from the on-page steps (or `static/shortcut/blueprint.md`).
3. Point Get Contents of URL at `POST {PUBLIC_BASE_URL}/api/v1/convert`.
4. Optionally share an iCloud link and set `SHORTCUT_INSTALL_URL` on the API
   host so the landing Add Shortcut button installs in one tap.

iOS only imports signed Shortcuts. A Mac (or an already-published iCloud link)
is required for a distributable `.shortcut` / iCloud install URL. This repo
ships the blueprint and install guide so you can publish that link yourself.

## Deploy architecture

| Piece | Where | Notes |
| --- | --- | --- |
| Landing + Shortcut install UX | GitHub Pages | Static only — expected URL `https://gabltz.github.io/uni-share/` |
| Convert API | Fly.io (Dockerfile + `fly.toml`) | Always-on HTTPS; set `PUBLIC_BASE_URL` to this origin |
| Local / Docker | Same binary serves API + `static/` | Useful for development |

Pages cannot run Rust. The static site reads `assets/site-config.js`
(`apiBaseUrl`) so Try a link / config calls hit the Fly API, not github.io.

### GitHub Pages

1. Repo Settings → Pages → Source: **GitHub Actions**.
2. Add repository **Actions variables**:
   - `API_BASE_URL` = your Fly HTTPS origin (e.g. `https://uni-share.fly.dev`)
   - optional `SHORTCUT_INSTALL_URL`
   - optional `PAGES_BASE_PATH` (default `/uni-share`)
3. Push to `main` or run workflow **Deploy GitHub Pages**.

Private repositories need **GitHub Pro** (or a public repo) for User/Org Pages.
This repo stays private by default; enable Pro or switch visibility only if you
choose to.

Preview the static bundle locally:

```bash
API_BASE_URL=https://uni-share.fly.dev bash scripts/build-pages.sh
```

### Fly.io API

```bash
# once on your Mac
brew install flyctl   # or https://fly.io/docs/hands-on/install-flyctl/
fly auth login
fly apps create uni-share   # rename in fly.toml if taken
fly secrets set PUBLIC_BASE_URL=https://uni-share.fly.dev
# optional:
# fly secrets set SPOTIFY_CLIENT_ID=… SPOTIFY_CLIENT_SECRET=…
# fly secrets set SHORTCUT_INSTALL_URL=https://www.icloud.com/shortcuts/…
fly deploy
```

For GitHub Actions deploy: add repository secret `FLY_API_TOKEN`
(`fly tokens create deploy`), then run workflow **Deploy API (Fly.io)**.

## License

MIT
