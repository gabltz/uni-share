# UNI-SHARE Shortcut blueprint

A signed Shortcut file ships in this folder:

| File | Role |
| --- | --- |
| `UNI-SHARE.shortcut` | Signed for anyone — landing **Add Shortcut** downloads this when no iCloud URL is set |
| `install.html` | Install page (iCloud link if configured, else the `.shortcut` file) |
| `actions.json` | Structured action checklist |

## What the Shortcut does

1. Share Sheet **or** presse-papiers (si lancé sans entrée).
2. **Text** **URL API de conversion** —  
   `https://YOUR_HOSTNAME/api/v1/convert` (à éditer si besoin).
3. **Ask** — confirme / colle le lien du morceau (prérempli Share ou clipboard).
4. **Menu** « Ouvrir sur » — Apple Music / Deezer / Tidal / Spotify.
5. **Get Contents of URL** — `POST` JSON `{ "url", "target" }`.
6. **Set Clipboard** — copie l’URL résultat.
7. **Menu** « Lien copié. Ouvrir maintenant ? » → Ouvrir / OK.

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
