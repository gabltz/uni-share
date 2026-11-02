# UMLS — Universal Music Link Share

Convert a Spotify, Apple Music, Deezer, or Tidal track link into the same
recording on another service. Ship it as an Apple Shortcut, a tiny web form, or
a JSON API.

Live: [https://umls.gab.rip](https://umls.gab.rip)

## Use

1. **Add Shortcut** from the landing page (iPhone / iPad).
2. Share a music link into the Shortcut, pick a target, open the result.
3. Or paste a URL in the web form / call the API.

## API

```bash
curl -s -X POST https://umls.gab.rip/api/v1/convert \
  -H 'content-type: application/json' \
  -d '{"url":"https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC","target":"apple_music"}'
```

Also: `GET /api/v1/convert?url=…&target=deezer`

**Targets:** `spotify` · `apple_music` · `deezer` · `tidal`

## Run locally

```bash
cp .env.example .env
cargo run --release
```

Open [http://127.0.0.1:8080](http://127.0.0.1:8080).

## Docker

Binds host port **38477** → container `8080`. Put a Cloudflare Tunnel in front
for public HTTPS (see `deploy/cloudflared/config.yml.example`).

```bash
cp .env.example .env
# PUBLIC_BASE_URL=https://umls.gab.rip
docker compose up -d --build
curl -s http://127.0.0.1:38477/health
```

## Env

| Variable | Notes |
| --- | --- |
| `PUBLIC_BASE_URL` | Public HTTPS origin (no trailing slash) |
| `SHORTCUT_INSTALL_URL` | Optional iCloud Shortcut link |
| `SPOTIFY_CLIENT_ID` / `SPOTIFY_CLIENT_SECRET` | Recommended for Spotify targets |
| `DEFAULT_COUNTRY` | Apple storefront (default `US`) |

## License

MIT
