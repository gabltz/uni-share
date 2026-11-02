# UNI-SHARE Shortcut blueprint

Build this once in the Shortcuts app, then optionally publish an iCloud link and
set `SHORTCUT_INSTALL_URL` so the landing page Add Shortcut button opens it
directly.

## Purpose

Accept a shared music URL, ask which service to open, call UNI-SHARE, open the
returned link.

## Inputs

- Share Sheet / Shortcut Input: URL or Text containing a Spotify, Apple Music,
  Deezer, or Tidal track link.

## Actions

1. **Receive** — URLs, Text (from Share Sheet).
2. **Choose from Menu**
   - Apple Music → Text `apple_music`
   - Deezer → Text `deezer`
   - Tidal → Text `tidal`
   - Spotify → Text `spotify`
3. **Get Contents of URL**
   - Method: `POST`
   - URL: `https://YOUR_HOST/api/v1/convert`
   - Headers: `Content-Type: application/json`
   - Body (JSON):
     ```json
     {
       "url": "<Shortcut Input>",
       "target": "<Chosen target text>"
     }
     ```
4. **Get Dictionary from Input** (if needed).
5. **Get Dictionary Value** — key `url`.
6. **Open URLs**.

## One-tap install (recommended after first export)

1. In Shortcuts, tap Share → Copy iCloud Link.
2. Set on the server:

   ```bash
   SHORTCUT_INSTALL_URL=https://www.icloud.com/shortcuts/……
   PUBLIC_BASE_URL=https://YOUR_HOST
   ```

3. Restart UNI-SHARE. The landing page Add Shortcut button then opens that iCloud
   link.

## Hosted files in this repo

| Path | Role |
| --- | --- |
| `static/shortcut/install.html` | Guided install + copy API endpoint |
| `static/shortcut/blueprint.md` | This action list |
| `static/shortcut/actions.json` | Structured action checklist for maintainers |

Unsigned `.shortcut` binaries cannot be imported on current iOS without Apple’s
signing step on a Mac. Prefer an iCloud link after you build the Shortcut once.
