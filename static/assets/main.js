(() => {
  const site = window.UNI_SHARE_SITE || {};
  let apiBase = String(site.apiBaseUrl || "")
    .trim()
    .replace(/\/$/, "");
  const form = document.getElementById("convert-form");
  const result = document.getElementById("convert-result");
  const shortcutInstall = document.getElementById("shortcut-install");
  const addShortcut = document.getElementById("add-shortcut");
  const note = document.getElementById("shortcut-note");
  const healthLink = document.getElementById("health-link");

  function apiUrl(path) {
    return `${apiBase}${path}`;
  }

  function applyShortcutUrl(url) {
    if (!url || !shortcutInstall) return false;
    shortcutInstall.href = url;
    if (addShortcut) addShortcut.href = url;
    if (note) {
      note.textContent =
        "Install opens your published iCloud Shortcut link. Re-export from Shortcuts after changing the API base URL.";
    }
    return true;
  }

  async function loadConfig() {
    if (healthLink) {
      healthLink.href = apiUrl("/health");
    }

    if (site.shortcutInstallUrl) {
      applyShortcutUrl(site.shortcutInstallUrl);
    }

    try {
      const res = await fetch(apiUrl("/api/v1/config"));
      if (!res.ok) throw new Error("config unavailable");
      const data = await res.json();
      if (!apiBase && data.publicBaseUrl) {
        apiBase = String(data.publicBaseUrl).replace(/\/$/, "");
        if (healthLink) healthLink.href = apiUrl("/health");
      }
      if (data.shortcutInstallUrl) {
        applyShortcutUrl(data.shortcutInstallUrl);
      } else if (!site.shortcutInstallUrl && note) {
        note.textContent =
          "No iCloud Shortcut URL is configured yet. Use the install page to build the Shortcut, or set SHORTCUT_INSTALL_URL on the API host.";
      }
    } catch (_) {
      if (!site.shortcutInstallUrl && note) {
        note.textContent = apiBase
          ? "Could not reach the API config endpoint. Convert still uses the configured API base URL."
          : "No API base URL configured for this static site. Set API_BASE_URL in the Pages build, or open the site served by the API.";
      }
    }
  }

  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const fd = new FormData(form);
    const url = String(fd.get("url") || "").trim();
    const target = String(fd.get("target") || "apple_music");
    result.hidden = false;
    result.classList.remove("error");
    result.textContent = "Converting…";
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
      result.textContent = JSON.stringify(data, null, 2);
    } catch (err) {
      result.classList.add("error");
      result.textContent = String(err);
    }
  });

  loadConfig();
})();
