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
  const FILE_INSTALL = "shortcut/UNI-SHARE.shortcut";

  function apiUrl(path) {
    return `${apiBase}${path}`;
  }

  function applyShortcutUrl(url, kind) {
    if (!url || !shortcutInstall) return false;
    shortcutInstall.href = url;
    if (addShortcut) addShortcut.href = url;
    if (note) {
      if (kind === "icloud") {
        note.textContent =
          "Install opens your published iCloud Shortcut link. After changing the API base URL in Shortcuts, re-share the iCloud link.";
      } else if (kind === "file") {
        note.textContent =
          "Installs the signed UNI-SHARE.shortcut file. On iPhone: allow Untrusted Shortcuts if asked. Edit the first Text action (API Convert URL) if your public host is not already set.";
      }
    }
    return true;
  }

  async function loadConfig() {
    if (healthLink) {
      healthLink.href = apiUrl("/health");
    }

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
      }
      if (data.shortcutInstallUrl) {
        usedIcloud = applyShortcutUrl(data.shortcutInstallUrl, "icloud");
      }
    } catch (_) {
      /* ignore — fall through to file install */
    }

    if (!usedIcloud) {
      applyShortcutUrl(FILE_INSTALL, "file");
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
