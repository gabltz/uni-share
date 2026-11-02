# UNI-SHARE

Cross-platform music link converter. Paste or share a Spotify, Apple Music,
Deezer, or Tidal track URL and get the same recording on another service.

Built as a single Rust binary: HTTP API, landing page, and Shortcut install
docs. The same `static/` tree deploys to **Vercel** (free) while the API runs
on a free Docker host (**Render** preferred).

## Features

- Convert track links across Spotify, Apple Music, Deezer, and Tidal
- Apple Shortcut flow via Share Sheet
- Landing page with Add Shortcut CTA
- Health check and CORS-enabled JSON API
- Optional split: static site on Vercel + API on Render (free tiers)

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
| Landing + Shortcut install UX | **Vercel** (free) | Static from `static/` via `vercel.json` + `scripts/build-site.sh` |
| Convert API | **Render** free Web Service (Docker) | `render.yaml` + existing Dockerfile; cold starts after idle |
| Local / Docker | Same binary serves API + `static/` | Development |
| GitHub Pages | Optional fallback | Needs Pro if the repo stays private |

Vercel cannot run the axum server. The static site reads `assets/site-config.js`
(`apiBaseUrl`) so Try a link / Shortcut docs hit the Render API origin.

Expected production URLs (after you create the projects):

- Frontend: `https://uni-share.vercel.app` (exact subdomain is chosen in Vercel)
- API: `https://uni-share-api.onrender.com` (from `render.yaml` service name)

### Vercel (frontend)

1. [vercel.com](https://vercel.com) → Add New → Project → Import `gabltz/uni-share` (private repos OK on the free hobby plan).
2. Framework Preset: **Other**. Build settings come from `vercel.json`.
3. Environment variables (Production):
   - `API_BASE_URL` = your Render HTTPS origin (no trailing slash)
   - optional `SHORTCUT_INSTALL_URL`
4. Deploy. Re-deploy after the API URL is known if the first build used an empty `API_BASE_URL`.

CLI (optional; stop at login if needed):

```bash
npm i -g vercel
cd ~/Documents/0_IT/1_DEV/uni-share
vercel login
vercel link
vercel env add API_BASE_URL
vercel --prod
```

Preview the static bundle locally:

```bash
API_BASE_URL=https://uni-share-api.onrender.com bash scripts/build-site.sh
```

### Render (API)

1. [render.com](https://render.com) → New → **Blueprint** (uses `render.yaml`) or **Web Service** → Docker → this repo.
2. Plan: **Free**.
3. Set env vars in the Render dashboard (do not commit secrets):
   - `PUBLIC_BASE_URL` = `https://uni-share-api.onrender.com` (your real service URL)
   - optional `SPOTIFY_CLIENT_ID` / `SPOTIFY_CLIENT_SECRET`
   - optional `SHORTCUT_INSTALL_URL`
4. Deploy. Confirm `GET /health` returns OK.

**Cold starts:** free Web Services sleep after inactivity. The first request (or
Shortcut run) after sleep can take ~30–60 seconds. Later requests are fast until
the next idle period.

Alternatives if Render is unavailable: Koyeb free / Railway trial credits — same
Dockerfile, set `PUBLIC_BASE_URL` to that host, then point Vercel `API_BASE_URL`
at it.

### GitHub Pages (optional)

Same `static/` tree via Actions workflow **Deploy GitHub Pages**. Set Actions
variable `API_BASE_URL` to the Render origin. Private repos need GitHub Pro for
user Pages; prefer Vercel instead.

## License

MIT
