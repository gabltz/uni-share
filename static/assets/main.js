(() => {
  const form = document.getElementById("convert-form");
  const result = document.getElementById("convert-result");
  const shortcutInstall = document.getElementById("shortcut-install");
  const addShortcut = document.getElementById("add-shortcut");
  const note = document.getElementById("shortcut-note");

  async function loadConfig() {
    try {
      const res = await fetch("/api/v1/config");
      if (!res.ok) return;
      const data = await res.json();
      if (data.shortcutInstallUrl) {
        shortcutInstall.href = data.shortcutInstallUrl;
        addShortcut.href = data.shortcutInstallUrl;
        note.textContent =
          "Install opens your published iCloud Shortcut link. Re-export from Shortcuts after changing the API base URL.";
      } else {
        note.textContent =
          "No iCloud Shortcut URL is configured yet. Use the install page to build the Shortcut, or set SHORTCUT_INSTALL_URL on the server.";
      }
    } catch (_) {
      /* ignore */
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
      const res = await fetch("/api/v1/convert", {
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
