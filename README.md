# UNI-SHARE

Cross-platform music link converter. Paste or share a Spotify, Apple Music,
Deezer, or Tidal track URL and get the same recording on another service.

Single Rust binary: HTTP API, landing page, and Shortcut install docs. Production
deploy is **Docker on a private host** with a **Cloudflare Tunnel** for public
HTTPS (no inbound firewall ports required).

## Features

- Convert track links across Spotify, Apple Music, Deezer, and Tidal
- Apple Shortcut flow via Share Sheet
- Landing page with Add Shortcut CTA
- Health check and CORS-enabled JSON API

## Quick start (local)

```bash
cp .env.example .env
cargo run --release
```

Open [http://127.0.0.1:8080](http://127.0.0.1:8080).

## Docker + Cloudflare Tunnel (production)

The app binds a high local port so it does not clash with common services.
Cloudflare Tunnel is the only public entry.

| Piece | Value |
| --- | --- |
| Host bind | `127.0.0.1`-reachable **`38477`** → container `8080` |
| Public URL | `https://YOUR_HOSTNAME` (Cloudflare) |
| Compose | `docker-compose.yml` |
| Tunnel example | `deploy/cloudflared/config.yml.example` |

```bash
cp .env.example .env
# Set PUBLIC_BASE_URL=https://YOUR_HOSTNAME
docker compose up -d --build
curl -s http://127.0.0.1:38477/health
```

### Cloudflare Tunnel

1. Install and log in: `brew install cloudflared` then `cloudflared tunnel login`
2. `cloudflared tunnel create uni-share`
3. Copy `deploy/cloudflared/config.yml.example` → `config.yml`, set tunnel id,
   credentials path, and hostname → `http://127.0.0.1:38477`
4. `cloudflared tunnel route dns uni-share YOUR_HOSTNAME`
5. `cloudflared tunnel --config deploy/cloudflared/config.yml run`
   (or install as a system service)

Optional Compose sidecar: put `config.yml` + `credentials.json` under
`deploy/cloudflared/` (gitignored), point ingress at `http://uni-share:8080`,
then `docker compose --profile tunnel up -d --build`.

Set `PUBLIC_BASE_URL` to `https://YOUR_HOSTNAME` (no trailing slash). The landing
page and Shortcut use that same origin through the tunnel.

## Environment

| Variable | Required | Description |
| --- | --- | --- |
| `PORT` | no | Listen port inside the container (Compose sets `8080`) |
| `PUBLIC_BASE_URL` | recommended | Public HTTPS origin via Cloudflare (no trailing slash) |
| `SHORTCUT_INSTALL_URL` | no | iCloud Shortcut link for one-tap Add Shortcut |
| `DEFAULT_COUNTRY` | no | Apple Music / iTunes storefront (default `US`) |
| `SPOTIFY_CLIENT_ID` | recommended | Spotify app client id |
| `SPOTIFY_CLIENT_SECRET` | recommended | Spotify app client secret |

Put secrets in `.env` on the host (gitignored). Never commit them.

No keys are required for Deezer or Apple Music lookups. Spotify credentials are
recommended so conversions *to* Spotify keep working when the upstream catalog
page omits a Spotify URL.

## API

### `GET /health`

### `POST /api/v1/convert`

```bash
curl -s -X POST http://127.0.0.1:38477/api/v1/convert \
  -H 'content-type: application/json' \
  -d '{"url":"https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC","target":"deezer"}'
```

Also: `GET /api/v1/convert?url=URL&target=deezer`

**Targets:** `spotify` | `apple_music` | `deezer` | `tidal`

**Success fields:** `url`, `target`, `title`, `artist`, `isrc`, `source_url`, `thumbnail`

### `GET /api/v1/config`

Public config for the landing page (`shortcutInstallUrl`, `publicBaseUrl`,
supported targets).

## Apple Shortcut

1. Open `https://YOUR_HOSTNAME/shortcut/install.html` on a phone.
2. Confirm the API base URL is `https://YOUR_HOSTNAME`.
3. Build from the on-page steps (or `static/shortcut/blueprint.md`).
4. Optional: publish an iCloud link and set `SHORTCUT_INSTALL_URL` in `.env`.

## License

MIT
