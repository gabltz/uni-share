# UNI-SHARE Shortcut blueprint

A signed Shortcut file ships in this folder:

| File | Role |
| --- | --- |
| `UNI-SHARE.shortcut` | Signed for anyone — landing **Add Shortcut** downloads this when no iCloud URL is set |
| `install.html` | Install page (iCloud link if configured, else the `.shortcut` file) |
| `actions.json` | Structured action checklist |

## What the Shortcut does

1. Receives a music URL (Share Sheet / ask for URL).
2. **Text** action **API Convert URL** — full convert endpoint  
   (`https://YOUR_HOSTNAME/api/v1/convert`). **Edit this** after deploy if it still
   points at localhost.
3. **Choose from Menu** — Apple Music / Deezer / Tidal / Spotify  
   (`apple_music`, `deezer`, `tidal`, `spotify`).
4. **Get Contents of URL** — `POST` JSON  
   `{"url": Shortcut Input, "target": Menu Result}`.
5. **Get Dictionary Value** — key `url`.
6. **Open URLs**.

## One-tap install (preferred)

1. On the Mac, open `UNI-SHARE.shortcut` in Shortcuts and confirm **Add Shortcut** once
   (Automation cannot click that dialog without Accessibility permission).
2. Shortcuts → Share → **Copy iCloud Link**.
3. Set on the server:

   ```bash
   SHORTCUT_INSTALL_URL=https://www.icloud.com/shortcuts/……
   PUBLIC_BASE_URL=https://YOUR_HOSTNAME
   ```

4. Restart the container / process. Landing **Add Shortcut** then opens the iCloud link.

## File install limitation

Hosting `UNI-SHARE.shortcut` works for Mac and for iOS when **Untrusted Shortcuts**
is allowed. iCloud links are still the smoothest phone install path Apple supports
for sharing.

## Rebuild / resign (Mac)

```bash
# After changing PUBLIC_BASE_URL in .env
bash scripts/build-shortcut.sh
# Then open static/shortcut/UNI-SHARE.shortcut and click Add Shortcut if prompted
```
