# UNI-SHARE — plan du raccourci

Fichier signé : `UNI-SHARE.shortcut` (bouton **Ajouter le raccourci** du site).

## Déroulement

1. Entrée Share Sheet **ou** presse-papiers (si aucune entrée).
2. Texte **URL API de conversion** = `https://umls.gab.rip/api/v1/convert`.
3. **Demander** le lien du morceau (prérempli) — vérifie / colle si besoin.
4. Menu **Ouvrir sur** → Apple Music / Deezer / Tidal / Spotify.
5. `POST` JSON vers l’API.
6. Si une URL est renvoyée → **copie presse-papiers** + notification + **ouvre** le lien.
7. Sinon → affiche le message d’erreur de l’API (plus de « Aucune URL indiquée » à tort).

## Install / update iPhone

1. Ouvre https://umls.gab.rip/shortcut/UNI-SHARE.shortcut
2. **Ajouter le raccourci** (remplace l’ancien).
3. Autorise les raccourcis non fiables si iOS le demande.

## Rebuild (Mac)

```bash
bash scripts/build-shortcut.sh
```
