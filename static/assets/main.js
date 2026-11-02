(() => {
  const site = window.UNI_SHARE_SITE || {};
  let apiBase = String(site.apiBaseUrl || "")
    .trim()
    .replace(/\/$/, "");
  const form = document.getElementById("convert-form");
  const result = document.getElementById("convert-result");
  const shortcutInstall = document.getElementById("shortcut-install");
  const note = document.getElementById("shortcut-note");
  const healthLink = document.getElementById("health-link");
  const apiSnippet = document.getElementById("api-snippet");
  const FILE_INSTALL = "shortcut/UNI-SHARE.shortcut";

  function apiUrl(path) {
    return `${apiBase}${path}`;
  }

  function refreshSnippet() {
    if (!apiSnippet) return;
    const origin = apiBase || window.location.origin;
    apiSnippet.textContent = `curl -s -X POST ${origin}/api/v1/convert \\
  -H 'content-type: application/json' \\
  -d '{"url":"https://open.spotify.com/track/…","target":"apple_music"}'`;
  }

  function applyShortcutUrl(url, kind) {
    if (!url || !shortcutInstall) return false;
    shortcutInstall.href = url;
    if (note) {
      note.textContent =
        kind === "icloud"
          ? "Installation via iCloud."
          : "Sur iPhone : autorise les raccourcis non fiables si demandé.";
    }
    return true;
  }

  async function loadConfig() {
    if (healthLink) healthLink.href = apiUrl("/health");
    refreshSnippet();

    let usedIcloud = false;
    if (site.shortcutInstallUrl) {
      usedIcloud = applyShortcutUrl(site.shortcutInstallUrl, "icloud");
    }

    try {
      const res = await fetch(apiUrl("/api/v1/config"));
      if (!res.ok) throw new Error("config unavailable");
      const data = await res.json();
      if (!apiBase && data.publicBaseUrl) {
        apiBase = String(data.publicBaseUrl).replace(/\/$/, "");
        if (healthLink) healthLink.href = apiUrl("/health");
        refreshSnippet();
      }
      if (data.shortcutInstallUrl) {
        usedIcloud = applyShortcutUrl(data.shortcutInstallUrl, "icloud");
      }
    } catch (_) {
      /* fall through to file install */
    }

    if (!usedIcloud) applyShortcutUrl(FILE_INSTALL, "file");
  }

  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const fd = new FormData(form);
    const url = String(fd.get("url") || "").trim();
    const target = String(fd.get("target") || "apple_music");
    result.hidden = false;
    result.classList.remove("error");
    result.textContent = "…";
    try {
      const res = await fetch(apiUrl("/api/v1/convert"), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url, target }),
      });
      const data = await res.json();
      if (!res.ok) {
        result.classList.add("error");
        result.textContent = data.message || data.error || "Conversion failed";
        return;
      }
      result.textContent = data.url || JSON.stringify(data, null, 2);
    } catch (err) {
      result.classList.add("error");
      result.textContent = String(err);
    }
  });

  loadConfig();
})();
