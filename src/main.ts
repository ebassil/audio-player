import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
import { isRegistered, register, unregister } from "@tauri-apps/plugin-global-shortcut";

interface PluginInfo {
  name: string;
  author: string | null;
  version: string | null;
  type: string;
  parameters: Array<{
    name: string;
    type: string;
    default: unknown;
    min: unknown;
    max: unknown;
  }>;
  has_ui: boolean;
}

interface GraphNode {
  id: number;
  name: string;
  node_type: string;
  enabled: boolean;
}

interface PlaylistTrack {
  file_path: string;
  mix_points: { mix_out: number | null; mix_in: number | null } | null;
  mix_pattern_override: string | null;
  metadata: { title: string | null; artist: string | null; album: string | null; duration_secs: number | null } | null;
}

interface LogEntry {
  timestamp: string;
  direction: string;
  name: string;
  detail: string;
  status: "success" | "error" | "event";
}

const MAX_LOG_ENTRIES = 1000;
const logEntries: LogEntry[] = [];

let currentTracks: PlaylistTrack[] = [];
let selectedTrackIndex: number | null = null;
let currentDurationSecs = 0;
let pluginWindow: WebviewWindow | null = null;

document.addEventListener("DOMContentLoaded", () => {
  const params = new URLSearchParams(window.location.search);
  if (params.get("view") === "plugins") {
    renderPluginPopup();
    return;
  }

  const app = document.getElementById("app")!;
  app.innerHTML = `
    <div class="layout">
      <aside class="sidebar">
        <div class="sidebar-section sidebar-section--playlist">
          <h3>Playlist</h3>
          <div class="playlist-toolbar">
            <button id="btn-load-playlist" title="Load JSON playlist">📂</button>
            <button id="btn-save-playlist" title="Save playlist">💾</button>
            <button id="btn-import-m3u8" title="Import M3U8">📄</button>
            <button id="btn-export-m3u8" title="Export M3U8">📤</button>
            <button id="btn-load-dir" title="Load directory">📁</button>
          </div>
          <ul id="playlist-view" class="playlist"></ul>
          <div id="playlist-info" class="playlist-info">No tracks</div>
        </div>
        
      </aside>
      <main class="content">
          <div id="header-toolbar" class="header-toolbar">
            <button id="btn-plugins">Plugins</button>
            <button id="btn-log">Log</button>
          </div>
          <div id="player-controls" class="player-controls">
            <button id="btn-prev">⏮</button>
            <button id="btn-play">▶</button>
            <button id="btn-pause">⏸</button>
            <button id="btn-stop">⏹</button>
            <button id="btn-next">⏭</button>
            <input type="range" id="volume-slider" min="0" max="100" value="100" />
            <span id="volume-label">100%</span>
            <button id="btn-mute">🔊</button>
            <span id="status-text">Stopped</span>
          </div>
        <div class="timeline-container">
          <div id="timeline" class="timeline">
            <div id="progress-bar" class="progress-bar" style="width: 0%"></div>
            <div id="mix-out-marker" class="mix-marker mix-out" title="Mix-Out Point" style="display: none"></div>
            <div id="mix-in-marker" class="mix-marker mix-in" title="Mix-In Point" style="display: none"></div>
          </div>
          <div class="mix-controls">
            <span class="mix-label">Mix:</span>
            <select id="mix-pattern-select">
              <option value="crossfade">Cross-Fade</option>
              <option value="fade">Fade</option>
              <option value="hardfade">Hard Fade</option>
            </select>
            <span class="mix-label">Duration:</span>
            <input type="range" id="mix-duration-slider" min="1" max="15" step="0.5" value="3" />
            <span id="mix-duration-label">3.0s</span>
            <button id="btn-set-mix-out">Set Mix-Out</button>
            <button id="btn-set-mix-in">Set Mix-In</button>
          </div>
        </div>
        <div id="plugin-ui-container" class="plugin-ui-container">
          <p class="hint">Select a plugin to see its UI</p>
        </div>
      </main>
    </div>
  `;

  // Volume control
  const volumeSlider = document.getElementById("volume-slider") as HTMLInputElement;
  const volumeLabel = document.getElementById("volume-label")!;
  const muteBtn = document.getElementById("btn-mute")!;

  volumeSlider.addEventListener("input", async () => {
    const gain = parseInt(volumeSlider.value) / 100;
    await loggedInvoke("set_volume", { gain });
    volumeLabel.textContent = `${Math.round(gain * 100)}%`;
    await saveAppConfig();
  });

  muteBtn.addEventListener("click", async () => {
    const muted = muteBtn.textContent === "🔊";
    await loggedInvoke("set_mute", { muted });
    muteBtn.textContent = muted ? "🔇" : "🔊";
    await saveAppConfig();
  });

  // Playback controls
  document.getElementById("btn-play")?.addEventListener("click", () => {
    loggedInvoke("play");
  });
  document.getElementById("btn-pause")?.addEventListener("click", () => {
    loggedInvoke("pause");
  });
  document.getElementById("btn-stop")?.addEventListener("click", () => {
    loggedInvoke("stop");
  });
  document.getElementById("btn-next")?.addEventListener("click", () => {
    playNextTrack();
  });
  document.getElementById("btn-prev")?.addEventListener("click", () => {
    playPrevTrack();
  });

  // Seek on timeline click
  const timeline = document.getElementById("timeline");
  timeline?.addEventListener("click", async (e) => {
    const rect = timeline.getBoundingClientRect();
    const pct = (e.clientX - rect.left) / rect.width;
    try {
      const dur = currentDurationSecs || 300;
      const seekPos = pct * dur;
      await loggedInvoke("seek", { positionSecs: seekPos });
    } catch {}
  });

  // Mix controls
  const mixPatternSelect = document.getElementById("mix-pattern-select") as HTMLSelectElement;
  const mixDurationSlider = document.getElementById("mix-duration-slider") as HTMLInputElement;
  const mixDurationLabel = document.getElementById("mix-duration-label")!;

  mixPatternSelect.addEventListener("change", async () => {
    await loggedInvoke("set_mix_config", {
      pattern: mixPatternSelect.value,
      durationSecs: parseFloat(mixDurationSlider.value),
    });
    await saveAppConfig();
  });

  mixDurationSlider.addEventListener("input", async () => {
    const val = parseFloat(mixDurationSlider.value);
    mixDurationLabel.textContent = `${val.toFixed(1)}s`;
    await loggedInvoke("set_mix_config", {
      pattern: mixPatternSelect.value,
      durationSecs: val,
    });
    await saveAppConfig();
  });

  document.getElementById("btn-set-mix-out")?.addEventListener("click", () => {
    const pct = 0.3;
    setMixOutPoint(pct);
  });

  document.getElementById("btn-set-mix-in")?.addEventListener("click", () => {
    const pct = 0.7;
    setMixInPoint(pct);
  });

  // Initialize plugin rack
  initPluginRack();
  loadMixConfig();
  initPlaylist();
  initGlobalShortcuts();
  initPlayerEvents();
  loadAppConfig();

  // Header toolbar buttons
  document.getElementById("btn-plugins")?.addEventListener("click", openPluginPopup);
  document.getElementById("btn-log")?.addEventListener("click", toggleLogPanel);

  // Add settings button
  const settingsBtn = document.createElement("button");
  settingsBtn.id = "btn-settings";
  settingsBtn.title = "Settings";
  settingsBtn.textContent = "⚙";
  settingsBtn.addEventListener("click", openSettingsPanel);
  document.getElementById("player-controls")?.appendChild(settingsBtn);
});

let mixOutPercent = 0;
let mixInPercent = 1;

function setMixOutPoint(pct: number) {
  mixOutPercent = Math.min(pct, mixInPercent - 0.05);
  const marker = document.getElementById("mix-out-marker")!;
  marker.style.display = "block";
  marker.style.left = `${mixOutPercent * 100}%`;
}

function setMixInPoint(pct: number) {
  mixInPercent = Math.max(pct, mixOutPercent + 0.05);
  const marker = document.getElementById("mix-in-marker")!;
  marker.style.display = "block";
  marker.style.left = `${mixInPercent * 100}%`;
}

// --- Playlist Management ---

function initPlaylist() {
  document.getElementById("btn-load-playlist")?.addEventListener("click", loadPlaylistJson);
  document.getElementById("btn-save-playlist")?.addEventListener("click", savePlaylistJson);
  document.getElementById("btn-import-m3u8")?.addEventListener("click", importM3u8);
  document.getElementById("btn-export-m3u8")?.addEventListener("click", exportM3u8);
  document.getElementById("btn-load-dir")?.addEventListener("click", loadDirectory);
  setupPlaylistDragDrop();
  setupPlaylistKeyboard();
  setupPlaylistDragReorder();
}

async function loadPlaylistJson() {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: false,
      filters: [{ name: "Playlist", extensions: ["json"] }],
    });
    if (!selected) return;
    const tracks: PlaylistTrack[] = await loggedInvoke("load_playlist", { path: selected });
    currentTracks = tracks;
    renderPlaylist();
    await syncPlaylistContext();
  } catch (err) {
    console.error("Failed to load playlist:", err);
  }
}

async function savePlaylistJson() {
  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const selected = await save({
      filters: [{ name: "JSON Playlist", extensions: ["json"] }],
      defaultPath: "playlist.json",
    });
    if (!selected) return;
    await loggedInvoke("save_playlist", { path: selected });
  } catch (err) {
    console.error("Failed to save playlist:", err);
  }
}

async function importM3u8() {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: false,
      filters: [{ name: "M3U Playlist", extensions: ["m3u8", "m3u"] }],
    });
    if (!selected) return;
    const tracks: PlaylistTrack[] = await loggedInvoke("import_m3u8", { path: selected });
    currentTracks = tracks;
    renderPlaylist();
    await syncPlaylistContext();
  } catch (err) {
    console.error("Failed to import M3U8:", err);
  }
}

async function exportM3u8() {
  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const selected = await save({
      filters: [{ name: "M3U8 Playlist", extensions: ["m3u8"] }],
      defaultPath: "playlist.m3u8",
    });
    if (!selected) return;
    await loggedInvoke("export_m3u8", { path: selected });
  } catch (err) {
    console.error("Failed to export M3U8:", err);
  }
}

async function loadDirectory() {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (!selected) return;

    const { readDir } = await import("@tauri-apps/plugin-fs");
    const audioExtensions = new Set([".mp3", ".wav", ".flac", ".ogg", ".m4a", ".aac", ".opus"]);

    async function scanDir(dirPath: string): Promise<string[]> {
      const results: string[] = [];
      const entries = await readDir(dirPath);
      for (const entry of entries) {
        const fullPath = `${dirPath}/${entry.name}`;
        if (entry.isDirectory) {
          try {
            const sub = await scanDir(fullPath);
            results.push(...sub);
          } catch {
            // Skip directories that can't be read
          }
        } else if (entry.name) {
          const ext = entry.name.substring(entry.name.lastIndexOf(".")).toLowerCase();
          if (audioExtensions.has(ext)) {
            results.push(fullPath);
          }
        }
      }
      return results;
    }

    const files = await scanDir(selected);
    const tracks: PlaylistTrack[] = files.map((f) => ({
      file_path: f,
      mix_points: null,
      mix_pattern_override: null,
      metadata: null,
    }));
    currentTracks = tracks;
    await loggedInvoke("set_playlist_tracks", { tracks: tracks as unknown as Record<string, unknown>[] });
    renderPlaylist();
    await syncPlaylistContext();
  } catch (err) {
    console.error("Failed to load directory:", err);
  }
}

async function deleteTrackFromPlaylist(index: number) {
  await loggedInvoke("remove_tracks_from_playlist", { indices: [index] });
  currentTracks.splice(index, 1);
  if (selectedTrackIndex === index) selectedTrackIndex = null;
  else if (selectedTrackIndex !== null && selectedTrackIndex > index) selectedTrackIndex--;
  renderPlaylist();
}

async function deleteTrackPlus(index: number) {
  const track = currentTracks[index];
  if (!track) return;

  const { confirm: showConfirm } = await import("@tauri-apps/plugin-dialog");
  const sessionToggleKey = "deleteplus_confirm";
  const skipConfirm = sessionStorage.getItem(sessionToggleKey) === "true";

  if (!skipConfirm) {
    const confirmed = await showConfirm(
      `Permanently delete "${track.file_path}" from disk?\n\nThis cannot be undone.`,
      { title: "Delete from Disk", kind: "warning" }
    );
    if (!confirmed) return;

    const { ask } = await import("@tauri-apps/plugin-dialog");
    const dontAskAgain = await ask("Don't ask again this session?", {
      title: "DeletePlus",
      kind: "info",
    });
    if (dontAskAgain) {
      sessionStorage.setItem(sessionToggleKey, "true");
    }
  }

  try {
    const { remove } = await import("@tauri-apps/plugin-fs");
    await remove(track.file_path);
  } catch (err) {
    console.error("Failed to delete file from disk:", err);
  }

  await deleteTrackFromPlaylist(index);
}

function renderPlaylist() {
  const list = document.getElementById("playlist-view");
  const info = document.getElementById("playlist-info");
  if (!list || !info) return;

  info.textContent = `${currentTracks.length} track(s)`;

  list.innerHTML = "";
  currentTracks.forEach((track, index) => {
    const li = document.createElement("li");
    li.className = "playlist-item";
    li.draggable = true;
    li.dataset.trackIndex = String(index);
    if (index === selectedTrackIndex) li.classList.add("selected");

    const fileName = track.file_path.split("/").pop() || track.file_path;
    li.innerHTML = `
      <span class="track-num">${index + 1}</span>
      <span class="track-name">${fileName}</span>
      <span class="track-actions">
        <button class="btn-delete" data-index="${index}" title="Delete from playlist">✕</button>
        <button class="btn-delete-plus" data-index="${index}" title="Delete from playlist + disk">🗑</button>
      </span>
    `;

    li.addEventListener("click", () => {
      selectedTrackIndex = index;
      renderPlaylist();
      loadTrack(track.file_path);
    });

    const delBtn = li.querySelector(".btn-delete") as HTMLElement;
    delBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      deleteTrackFromPlaylist(index);
    });

    const delPlusBtn = li.querySelector(".btn-delete-plus") as HTMLElement;
    delPlusBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      deleteTrackPlus(index);
    });

    list.appendChild(li);
  });
}

async function loadTrack(filePath: string) {
  try {
    await loggedInvoke("load_track", { path: filePath });
    await loggedInvoke("play");
    await syncTrackIndex();
  } catch (err) {
    console.error("Failed to load track:", err);
  }
}

async function syncPlaylistContext() {
  try {
    const entries = currentTracks.map((t) => ({
      file_path: t.file_path,
      mix_out: t.mix_points?.mix_out ?? null,
      mix_in: t.mix_points?.mix_in ?? null,
    }));
    await loggedInvoke("set_playlist_context", { entries });
  } catch (err) {
    console.error("Failed to sync playlist context:", err);
  }
}

async function syncTrackIndex() {
  try {
    await loggedInvoke("set_current_track_index", { index: selectedTrackIndex !== null ? selectedTrackIndex : -1 });
  } catch (err) {
    console.error("Failed to sync track index:", err);
  }
}

async function playNextTrack() {
  if (selectedTrackIndex === null || selectedTrackIndex >= currentTracks.length - 1) return;
  selectedTrackIndex++;
  renderPlaylist();
  await loadTrack(currentTracks[selectedTrackIndex].file_path);
}

async function playPrevTrack() {
  if (selectedTrackIndex === null || selectedTrackIndex <= 0) return;
  selectedTrackIndex--;
  renderPlaylist();
  await loadTrack(currentTracks[selectedTrackIndex].file_path);
}

function setupPlaylistDragDrop() {
  const audioExtensions = new Set([".mp3", ".wav", ".flac", ".ogg", ".m4a", ".aac", ".opus"]);

  getCurrentWindow().onDragDropEvent(async (event) => {
    if (event.payload.type === "over") {
      document.body.classList.add("drag-over");
      return;
    }
    if (event.payload.type === "leave") {
      document.body.classList.remove("drag-over");
      return;
    }
    if (event.payload.type !== "drop") return;
    document.body.classList.remove("drag-over");

    const paths = event.payload.paths;
    if (!paths || paths.length === 0) return;

    const droppedPaths: string[] = [];

    async function processPath(path: string) {
      const ext = path.substring(path.lastIndexOf(".")).toLowerCase();
      if (audioExtensions.has(ext)) {
        droppedPaths.push(path);
      }
    }

    for (const p of paths) {
      await processPath(p);
    }

    if (droppedPaths.length === 0) return;

    const newTracks: PlaylistTrack[] = droppedPaths.map((f) => ({
      file_path: f,
      mix_points: null,
      mix_pattern_override: null,
      metadata: null,
    }));
    currentTracks = [...currentTracks, ...newTracks];
    await loggedInvoke("set_playlist_tracks", { tracks: currentTracks as unknown as Record<string, unknown>[] });
    renderPlaylist();
    await syncPlaylistContext();
  });
}

function setupPlaylistKeyboard() {
  document.addEventListener("keydown", async (e) => {
    if (selectedTrackIndex === null) return;

    if (e.key === "Delete" || e.key === "Backspace") {
      if (e.shiftKey) {
        await deleteTrackPlus(selectedTrackIndex);
      } else {
        await deleteTrackFromPlaylist(selectedTrackIndex);
      }
    }

    if (e.key === "ArrowDown" && selectedTrackIndex < currentTracks.length - 1) {
      selectedTrackIndex++;
      renderPlaylist();
      if (currentTracks[selectedTrackIndex]) {
        loadTrack(currentTracks[selectedTrackIndex].file_path);
      }
    }
    if (e.key === "ArrowUp" && selectedTrackIndex > 0) {
      selectedTrackIndex--;
      renderPlaylist();
      if (currentTracks[selectedTrackIndex]) {
        loadTrack(currentTracks[selectedTrackIndex].file_path);
      }
    }
  });
}

function setupPlaylistDragReorder() {
  const list = document.getElementById("playlist-view");
  if (!list) return;

  let dragItem: HTMLElement | null = null;

  list.addEventListener("dragstart", (e) => {
    const target = (e.target as HTMLElement).closest(".playlist-item") as HTMLElement;
    if (target) {
      dragItem = target;
      target.style.opacity = "0.5";
      e.dataTransfer?.setData("text/plain", target.dataset.trackIndex ?? "");
    }
  });

  list.addEventListener("dragend", (e) => {
    const target = (e.target as HTMLElement).closest(".playlist-item") as HTMLElement;
    if (target) {
      target.style.opacity = "1";
    }
    dragItem = null;
    document.querySelectorAll(".drag-over").forEach((el) => el.classList.remove("drag-over"));
  });

  list.addEventListener("dragover", (e) => {
    e.preventDefault();
    const target = (e.target as HTMLElement).closest(".playlist-item") as HTMLElement;
    if (target && target !== dragItem) {
      target.classList.add("drag-over");
    }
  });

  list.addEventListener("dragleave", (e) => {
    const target = (e.target as HTMLElement).closest(".playlist-item") as HTMLElement;
    target?.classList.remove("drag-over");
  });

  list.addEventListener("drop", async (e) => {
    e.preventDefault();
    const target = (e.target as HTMLElement).closest(".playlist-item") as HTMLElement;
    if (!dragItem || !target || dragItem === target) return;

    target.classList.remove("drag-over");

    const items = [...list.querySelectorAll(".playlist-item")];
    const fromPos = items.indexOf(dragItem);
    const toPos = items.indexOf(target);

    if (fromPos < toPos) {
      target.insertAdjacentElement("afterend", dragItem);
    } else {
      target.insertAdjacentElement("beforebegin", dragItem);
    }

    // Update data model
    const [moved] = currentTracks.splice(fromPos, 1);
    currentTracks.splice(toPos, 0, moved);

    // Update selected index
    if (selectedTrackIndex === fromPos) {
      selectedTrackIndex = toPos;
    } else if (selectedTrackIndex !== null) {
      if (fromPos < selectedTrackIndex && toPos >= selectedTrackIndex) {
        selectedTrackIndex--;
      } else if (fromPos > selectedTrackIndex && toPos <= selectedTrackIndex) {
        selectedTrackIndex++;
      }
    }

    await loggedInvoke("set_playlist_tracks", { tracks: currentTracks as unknown as Record<string, unknown>[] });
    renderPlaylist();
  });
}

async function loadMixConfig() {
  try {
    const config: { pattern: string; duration_secs: number } = await loggedInvoke("get_mix_config");
    const select = document.getElementById("mix-pattern-select") as HTMLSelectElement;
    const slider = document.getElementById("mix-duration-slider") as HTMLInputElement;
    const label = document.getElementById("mix-duration-label")!;
    if (select) select.value = config.pattern.toLowerCase();
    if (slider) {
      slider.value = String(config.duration_secs);
      label.textContent = `${config.duration_secs.toFixed(1)}s`;
    }
  } catch (err) {
    console.error("Failed to load mix config:", err);
  }
}

async function initPluginRack() {
  const rack = document.getElementById("plugin-rack");
  if (!rack) return;

  try {
    const plugins: PluginInfo[] = await loggedInvoke("get_plugins");
    const nodes: GraphNode[] = await loggedInvoke("get_graph_nodes");

    rack.innerHTML = "";

    if (plugins.length === 0) {
      rack.innerHTML = `<p class="empty">No plugins found</p>`;
      return;
    }

    plugins.forEach((plugin, index) => {
      const node = nodes[index];
      const card = document.createElement("div");
      card.className = "plugin-card";
      card.draggable = true;
      card.dataset.pluginIndex = String(index);
      card.innerHTML = `
        <div class="plugin-header">
          <span class="drag-handle">⠿</span>
          <span class="plugin-name">${plugin.name}</span>
          <span class="plugin-type">${plugin.type}</span>
        </div>
        <div class="plugin-controls">
          <label>
            <input type="checkbox" class="plugin-toggle" ${node?.enabled ? "checked" : ""} />
            Enable
          </label>
          ${plugin.has_ui ? `<button class="btn-show-ui">UI</button>` : ""}
        </div>
      `;
      rack.appendChild(card);

      const toggle = card.querySelector(".plugin-toggle") as HTMLInputElement;
      toggle?.addEventListener("change", async () => {
        if (node) {
          await loggedInvoke("enable_plugin", { nodeId: node.id, enabled: toggle.checked });
        }
      });

      const uiBtn = card.querySelector(".btn-show-ui");
      uiBtn?.addEventListener("click", async () => {
        await loadPluginUi(index);
      });
    });

    // Drag-and-drop reordering
    setupDragReorder(rack);
  } catch (err) {
    rack.innerHTML = `<p class="error">Failed to load plugins: ${err}</p>`;
  }
}

function renderPluginPopup() {
  const app = document.getElementById("app")!;
  app.innerHTML = `
    <div class="plugin-popup-layout">
      <div id="plugin-popup-header" class="plugin-popup-header">
        <h2>Plugins</h2>
      </div>
      <div id="plugin-rack" class="plugin-rack"></div>
      <div id="plugin-ui-container" class="plugin-ui-container"></div>
    </div>
  `;
  document.title = "Plugins";
  initPluginRack();
}

async function openPluginPopup() {
  if (pluginWindow) {
    try {
      await pluginWindow.show();
      await pluginWindow.setFocus();
      return;
    } catch {
      pluginWindow = null;
    }
  }

  try {
    pluginWindow = new WebviewWindow("plugins", {
      url: "/?view=plugins",
      title: "Plugins",
      width: 800,
      height: 600,
      center: true,
      resizable: true,
    });

    pluginWindow.once("tauri://error", () => {
      pluginWindow = null;
    }).catch(() => {});

    pluginWindow.once("tauri://destroyed", () => {
      pluginWindow = null;
    }).catch(() => {});
  } catch (err) {
    console.error("Failed to create plugin window:", err);
    pluginWindow = null;
  }
}

function setupDragReorder(container: HTMLElement) {
  let dragItem: HTMLElement | null = null;

  container.addEventListener("dragstart", (e) => {
    const target = (e.target as HTMLElement).closest(".plugin-card") as HTMLElement;
    if (target) {
      dragItem = target;
      target.style.opacity = "0.5";
      e.dataTransfer?.setData("text/plain", target.dataset.pluginIndex ?? "");
    }
  });

  container.addEventListener("dragend", (e) => {
    const target = (e.target as HTMLElement).closest(".plugin-card") as HTMLElement;
    if (target) {
      target.style.opacity = "1";
    }
    dragItem = null;
    document.querySelectorAll(".drag-over").forEach((el) => el.classList.remove("drag-over"));
  });

  container.addEventListener("dragover", (e) => {
    e.preventDefault();
    const target = (e.target as HTMLElement).closest(".plugin-card") as HTMLElement;
    if (target && target !== dragItem) {
      target.classList.add("drag-over");
    }
  });

  container.addEventListener("dragleave", (e) => {
    const target = (e.target as HTMLElement).closest(".plugin-card") as HTMLElement;
    target?.classList.remove("drag-over");
  });

  container.addEventListener("drop", async (e) => {
    e.preventDefault();
    const target = (e.target as HTMLElement).closest(".plugin-card") as HTMLElement;
    if (!dragItem || !target || dragItem === target) return;

    target.classList.remove("drag-over");

    const cards = [...container.querySelectorAll(".plugin-card")];
    const fromPos = cards.indexOf(dragItem);
    const toPos = cards.indexOf(target);
    if (fromPos < toPos) {
      target.insertAdjacentElement("afterend", dragItem);
    } else {
      target.insertAdjacentElement("beforebegin", dragItem);
    }

    const updatedCards = [...container.querySelectorAll(".plugin-card")] as HTMLElement[];
    updatedCards.forEach((card, i) => {
      card.dataset.pluginIndex = String(i);
    });

    const newOrder = updatedCards.map((_, i) => i);
    try {
      await loggedInvoke("reorder_plugins", { order: newOrder });
    } catch (err) {
      console.error("Reorder failed:", err);
    }
  });
}

async function loadPluginUi(pluginIndex: number) {
  const container = document.getElementById("plugin-ui-container");
  if (!container) return;

  try {
    const html: string = await loggedInvoke("get_plugin_ui", { pluginIndex });
    const iframe = document.createElement("iframe");
    iframe.className = "plugin-iframe";
    iframe.setAttribute("sandbox", "allow-scripts allow-same-origin");
    container.innerHTML = "";
    container.appendChild(iframe);

    // Write the plugin HTML into the iframe
    const doc = iframe.contentDocument || iframe.contentWindow?.document;
    if (doc) {
      doc.open();
      doc.write(html);
      doc.close();
    }

    // Listen for parameter changes from plugin via postMessage
    window.addEventListener("message", async (event) => {
      if (event.source === iframe.contentWindow && event.data?.type === "param_change") {
        await loggedInvoke("set_plugin_parameter", {
          pluginIndex,
          parameter: event.data.name,
          value: event.data.value,
        });
      }
    });
  } catch (err) {
    container.innerHTML = `<p class="error">Failed to load plugin UI: ${err}</p>`;
  }
}

// --- Global Shortcut Integration (task 6.3) ---

interface ShortcutBinding {
  action: string;
  action_label: string;
  key_combo: string;
}

async function initGlobalShortcuts() {
  try {
    const shortcuts: ShortcutBinding[] = await loggedInvoke("get_shortcuts");
    for (const s of shortcuts) {
      const registered = await isRegistered(s.key_combo);
      if (!registered) {
        await register(s.key_combo, (event) => {
          if (event.state === "Pressed") {
            handleShortcutAction(s.action);
          }
        });
      }
    }
  } catch (err) {
    console.error("Failed to register global shortcuts:", err);
  }
}

async function reinitShortcuts() {
  try {
    // Unregister all, re-register from config
    const shortcuts: ShortcutBinding[] = await loggedInvoke("get_shortcuts");
    for (const s of shortcuts) {
      try {
        await unregister(s.key_combo);
      } catch {}
      try {
        await register(s.key_combo, (event) => {
          if (event.state === "Pressed") {
            handleShortcutAction(s.action);
          }
        });
      } catch (e) {
        console.error(`Failed to register ${s.key_combo}:`, e);
      }
    }
  } catch (err) {
    console.error("Failed to reinit shortcuts:", err);
  }
}

let logPanelVisible = false;
let lastPlayerStatusLog = 0;

let logAutoScroll = true;

function addLogEntry(entry: LogEntry) {
  logEntries.push(entry);
  if (logEntries.length > MAX_LOG_ENTRIES) {
    logEntries.splice(0, logEntries.length - MAX_LOG_ENTRIES);
    if (logPanelVisible) {
      renderLogPanel();
    }
    return;
  }
  if (logPanelVisible) {
    const entriesDiv = document.getElementById("log-entries");
    if (entriesDiv) {
      entriesDiv.appendChild(createLogEntryElement(entry));
      if (logAutoScroll) {
        entriesDiv.scrollTop = entriesDiv.scrollHeight;
      }
    }
  }
}

async function loggedInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const ts = new Date().toISOString();
  const argsStr = args ? JSON.stringify(args).substring(0, 200) : "";
  try {
    const result = await invoke<T>(command, args);
    addLogEntry({
      timestamp: ts,
      direction: "→",
      name: command,
      detail: argsStr,
      status: "success",
    });
    return result;
  } catch (err) {
    addLogEntry({
      timestamp: ts,
      direction: "→",
      name: command,
      detail: `${argsStr} → ${err}`,
      status: "error",
    });
    throw err;
  }
}

function createLogEntryElement(entry: LogEntry): HTMLElement {
  const div = document.createElement("div");
  div.className = `log-entry log-${entry.status}`;
  div.innerHTML = `
    <span class="log-time">${entry.timestamp.slice(11, 23)}</span>
    <span class="log-dir">${entry.direction}</span>
    <span class="log-name">${entry.name}</span>
    <span class="log-detail">${escapeHtml(entry.detail)}</span>
  `;
  return div;
}

function renderLogPanel() {
  const container = document.getElementById("plugin-ui-container");
  if (!container) return;
  container.innerHTML = `
    <div class="log-panel" id="log-panel">
      <div class="log-panel-header">
        <span>Debug Log</span>
        <button id="btn-log-clear" class="log-btn-small">Clear</button>
      </div>
      <div class="log-entries" id="log-entries">
        ${logEntries.map(e => `
          <div class="log-entry log-${e.status}">
            <span class="log-time">${e.timestamp.slice(11, 23)}</span>
            <span class="log-dir">${e.direction}</span>
            <span class="log-name">${e.name}</span>
            <span class="log-detail">${escapeHtml(e.detail)}</span>
          </div>
        `).join("")}
      </div>
    </div>
  `;

  document.getElementById("btn-log-clear")?.addEventListener("click", () => {
    logEntries.length = 0;
    renderLogPanel();
  });

  const entriesDiv = document.getElementById("log-entries");
  if (entriesDiv) {
    entriesDiv.scrollTop = entriesDiv.scrollHeight;
    logAutoScroll = true;
    entriesDiv.addEventListener("scroll", () => {
      const { scrollTop, scrollHeight, clientHeight } = entriesDiv;
      logAutoScroll = scrollHeight - scrollTop - clientHeight < 1;
    });
  }
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function toggleLogPanel() {
  logPanelVisible = !logPanelVisible;
  const container = document.getElementById("plugin-ui-container");
  if (!container) return;
  if (logPanelVisible) {
    container.style.display = "block";
    renderLogPanel();
  } else {
    container.innerHTML = `<p class="hint">Toggle Log to see debug output</p>`;
    container.style.display = "block";
  }
}

async function initPlayerEvents() {
  await listen<{
    state: string;
    volume: number;
    muted: boolean;
    progress: number;
    position_secs: number;
    duration_secs: number;
  }>("player-status", (event) => {
    const status = event.payload;
    currentDurationSecs = status.duration_secs;
    const statusText = document.getElementById("status-text");
    if (statusText) {
      statusText.textContent = status.state;
    }
    const progressBar = document.getElementById("progress-bar");
    if (progressBar) {
      progressBar.style.width = `${status.progress * 100}%`;
    }

    const now = Date.now();
    if (now - lastPlayerStatusLog >= 1000) {
      lastPlayerStatusLog = now;
      addLogEntry({
        timestamp: new Date().toISOString(),
        direction: "←",
        name: "player-status",
        detail: `state=${status.state} vol=${Math.round(status.volume * 100)}% pos=${Math.round(status.position_secs)}s`,
        status: "event",
      });
    }
  });

  await listen<{ track_index: number }>("track-changed", (event) => {
    const { track_index } = event.payload;
    selectedTrackIndex = track_index;
    renderPlaylist();
    addLogEntry({
      timestamp: new Date().toISOString(),
      direction: "←",
      name: "track-changed",
      detail: `track_index=${track_index}`,
      status: "event",
    });
  });

  await listen<{ timestamp: string; message: string }>("audio-log", (event) => {
    addLogEntry({
      timestamp: event.payload.timestamp,
      direction: "←",
      name: "audio-log",
      detail: event.payload.message,
      status: "event",
    });
  });
}

async function loadAppConfig() {
  try {
    const config: { mix_pattern: string; mix_duration_secs: number; volume: number; muted: boolean } =
      await loggedInvoke("load_app_config");
    const volumeSlider = document.getElementById("volume-slider") as HTMLInputElement;
    const muteBtn = document.getElementById("btn-mute");
    if (volumeSlider) {
      volumeSlider.value = String(Math.round(config.volume * 100));
    }
    if (muteBtn) {
      muteBtn.textContent = config.muted ? "🔇" : "🔊";
    }
    const mixPattern = document.getElementById("mix-pattern-select") as HTMLSelectElement;
    const mixDuration = document.getElementById("mix-duration-slider") as HTMLInputElement;
    const mixDurationLabel = document.getElementById("mix-duration-label");
    if (mixPattern) mixPattern.value = config.mix_pattern;
    if (mixDuration) mixDuration.value = String(config.mix_duration_secs);
    if (mixDurationLabel) mixDurationLabel.textContent = `${config.mix_duration_secs.toFixed(1)}s`;
  } catch (err) {
    console.error("Failed to load app config:", err);
  }
}

async function saveAppConfig() {
  try {
    await loggedInvoke("save_app_config");
  } catch (err) {
    console.error("Failed to save app config:", err);
  }
}

function handleShortcutAction(action: string) {
  switch (action) {
    case "PlayPause":
      loggedInvoke("play");
      break;
    case "NextTrack":
      playNextTrack();
      break;
    case "PreviousTrack":
      playPrevTrack();
      break;
    case "Delete":
      if (selectedTrackIndex !== null) deleteTrackFromPlaylist(selectedTrackIndex);
      break;
    case "DeletePlus":
      if (selectedTrackIndex !== null) deleteTrackPlus(selectedTrackIndex);
      break;
    case "VolumeUp":
      {
        const slider = document.getElementById("volume-slider") as HTMLInputElement;
        if (slider) {
          const val = Math.min(100, parseInt(slider.value) + 5);
          slider.value = String(val);
          slider.dispatchEvent(new Event("input"));
        }
      }
      break;
    case "VolumeDown":
      {
        const slider = document.getElementById("volume-slider") as HTMLInputElement;
        if (slider) {
          const val = Math.max(0, parseInt(slider.value) - 5);
          slider.value = String(val);
          slider.dispatchEvent(new Event("input"));
        }
      }
      break;
    case "Mute":
      {
        const btn = document.getElementById("btn-mute");
        if (btn) btn.click();
      }
      break;
    case "SeekForward":
      break;
    case "SeekBackward":
      break;
  }
}

// Extensible action registry (task 6.6)
// Future actions can be added by extending handleShortcutAction()

// --- Settings Panel (tasks 6.4, 8.5) ---

async function openSettingsPanel() {
  const container = document.getElementById("plugin-ui-container");
  if (!container) return;
  logPanelVisible = false;

  try {
    const shortcuts: ShortcutBinding[] = await loggedInvoke("get_shortcuts");

    container.innerHTML = `
      <div class="settings-panel">
        <h3>Settings</h3>

        <section class="settings-section">
          <h4>Keyboard Shortcuts</h4>
          <div class="shortcut-list" id="shortcut-list">
            ${shortcuts.map((s, i) => `
              <div class="shortcut-item" data-index="${i}">
                <span class="shortcut-label">${s.action_label}</span>
                <span class="shortcut-key" data-action="${s.action}" data-key="${s.key_combo}">
                  ${s.key_combo}
                </span>
                <button class="btn-rebind" data-action="${s.action}">Rebind</button>
              </div>
            `).join("")}
          </div>
          <div class="settings-actions">
            <button id="btn-reset-shortcuts">Reset to Defaults</button>
            <button id="btn-save-shortcuts">Save Shortcuts</button>
          </div>
        </section>

        <section class="settings-section">
          <h4>Mix Defaults</h4>
          <div class="setting-row">
            <label>Default Mix Pattern:</label>
            <select id="settings-mix-pattern">
              <option value="crossfade">Cross-Fade</option>
              <option value="fade">Fade</option>
              <option value="hardfade">Hard Fade</option>
            </select>
          </div>
          <div class="setting-row">
            <label>Default Mix Duration (s):</label>
            <input type="range" id="settings-mix-duration" min="1" max="15" step="0.5" value="3" />
            <span id="settings-mix-duration-label">3.0s</span>
          </div>
        </section>

        <section class="settings-section">
          <h4>Audio Device</h4>
          <div class="setting-row">
            <label>Output Device:</label>
            <select id="settings-audio-device"></select>
          </div>
        </section>
      </div>
    `;

    // Wire up rebind buttons
    document.querySelectorAll(".btn-rebind").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const action = (btn as HTMLElement).dataset.action!;
        const keySpan = document.querySelector(`.shortcut-key[data-action="${action}"]`) as HTMLElement;
        if (!keySpan) return;

        keySpan.textContent = "Press a key combination...";
        (keySpan as HTMLElement).style.color = "#e67e22";

        const handler = async (e: KeyboardEvent) => {
          const combo = buildKeyCombo(e);
          if (!combo) return;
          e.preventDefault();
          document.removeEventListener("keydown", handler);

          try {
            await loggedInvoke("set_shortcut", { action, keyCombo: combo });
            keySpan.textContent = combo;
            (keySpan as HTMLElement).dataset.key = combo;
            (keySpan as HTMLElement).style.color = "";
            await reinitShortcuts();
          } catch (err) {
            keySpan.textContent = `Error: ${err}`;
            (keySpan as HTMLElement).style.color = "#d32f2f";
            setTimeout(() => {
              keySpan.textContent = (keySpan as HTMLElement).dataset.key || combo;
              (keySpan as HTMLElement).style.color = "";
            }, 2000);
          }
        };

        document.addEventListener("keydown", handler, { once: true });
      });
    });

    document.getElementById("btn-reset-shortcuts")?.addEventListener("click", async () => {
      await loggedInvoke("reset_shortcuts");
      await openSettingsPanel();
      await reinitShortcuts();
    });

    document.getElementById("btn-save-shortcuts")?.addEventListener("click", async () => {
      await loggedInvoke("save_shortcuts");
    });

    // Mix defaults sync
    const mixPattern = document.getElementById("settings-mix-pattern") as HTMLSelectElement;
    const mixDuration = document.getElementById("settings-mix-duration") as HTMLInputElement;
    const mixDurationLabel = document.getElementById("settings-mix-duration-label")!;

    try {
      const mixConfig: { pattern: string; duration_secs: number } = await loggedInvoke("get_mix_config");
      mixPattern.value = mixConfig.pattern.toLowerCase();
      mixDuration.value = String(mixConfig.duration_secs);
      mixDurationLabel.textContent = `${mixConfig.duration_secs.toFixed(1)}s`;
    } catch {}

    mixPattern.addEventListener("change", async () => {
      await loggedInvoke("set_mix_config", {
        pattern: mixPattern.value,
        durationSecs: parseFloat(mixDuration.value),
      });
    });

    mixDuration.addEventListener("input", async () => {
      const val = parseFloat(mixDuration.value);
      mixDurationLabel.textContent = `${val.toFixed(1)}s`;
      await loggedInvoke("set_mix_config", {
        pattern: mixPattern.value,
        durationSecs: val,
      });
    });
  } catch (err) {
    container.innerHTML = `<p class="error">Failed to load settings: ${err}</p>`;
  }
}

function buildKeyCombo(e: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");

  const keyMap: Record<string, string> = {
    " ": "Space",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Escape: "Esc",
    Enter: "Enter",
    Tab: "Tab",
  };

  let key = e.key;
  if (keyMap[key]) key = keyMap[key];
  if (key.length === 1) key = key.toUpperCase();

  // Require at least one modifier
  if (parts.length === 0 && key.length > 1) {
    // Function keys without modifier
  } else if (parts.length === 0) {
    return null;
  }

  parts.push(key);
  return parts.join("+");
}
