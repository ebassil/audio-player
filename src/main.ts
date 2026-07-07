import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

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

let currentTracks: PlaylistTrack[] = [];
let selectedTrackIndex: number | null = null;

document.addEventListener("DOMContentLoaded", () => {
  const app = document.getElementById("app")!;
  app.innerHTML = `
    <div class="layout">
      <aside class="sidebar">
        <div class="sidebar-section">
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
        <div class="sidebar-section">
          <h3>Plugin Rack</h3>
          <div id="plugin-rack" class="plugin-rack"></div>
        </div>
      </aside>
      <main class="content">
        <div id="player-controls" class="player-controls">
          <button id="btn-play">▶</button>
          <button id="btn-pause">⏸</button>
          <button id="btn-stop">⏹</button>
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
    await invoke("set_volume", { gain });
    volumeLabel.textContent = `${Math.round(gain * 100)}%`;
  });

  muteBtn.addEventListener("click", async () => {
    const muted = muteBtn.textContent === "🔊";
    await invoke("set_mute", { muted });
    muteBtn.textContent = muted ? "🔇" : "🔊";
  });

  // Playback controls
  document.getElementById("btn-play")?.addEventListener("click", () => {
    invoke("play");
  });
  document.getElementById("btn-pause")?.addEventListener("click", () => {
    invoke("pause");
  });
  document.getElementById("btn-stop")?.addEventListener("click", () => {
    invoke("stop");
  });

  // Mix controls
  const mixPatternSelect = document.getElementById("mix-pattern-select") as HTMLSelectElement;
  const mixDurationSlider = document.getElementById("mix-duration-slider") as HTMLInputElement;
  const mixDurationLabel = document.getElementById("mix-duration-label")!;

  mixPatternSelect.addEventListener("change", async () => {
    await invoke("set_mix_config", {
      pattern: mixPatternSelect.value,
      duration_secs: parseFloat(mixDurationSlider.value),
    });
  });

  mixDurationSlider.addEventListener("input", async () => {
    const val = parseFloat(mixDurationSlider.value);
    mixDurationLabel.textContent = `${val.toFixed(1)}s`;
    await invoke("set_mix_config", {
      pattern: mixPatternSelect.value,
      duration_secs: val,
    });
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
}

async function loadPlaylistJson() {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: false,
      filters: [{ name: "Playlist", extensions: ["json"] }],
    });
    if (!selected) return;
    const tracks: PlaylistTrack[] = await invoke("load_playlist", { path: selected });
    currentTracks = tracks;
    renderPlaylist();
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
    await invoke("save_playlist", { path: selected });
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
    const tracks: PlaylistTrack[] = await invoke("import_m3u8", { path: selected });
    currentTracks = tracks;
    renderPlaylist();
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
    await invoke("export_m3u8", { path: selected });
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
        if (!entry.isFile) {
          const sub = await scanDir(fullPath);
          results.push(...sub);
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
    await invoke("set_playlist_tracks", { tracks: tracks as unknown as Record<string, unknown>[] });
    renderPlaylist();
  } catch (err) {
    console.error("Failed to load directory:", err);
  }
}

async function deleteTrackFromPlaylist(index: number) {
  await invoke("remove_tracks_from_playlist", { indices: [index] });
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
    await invoke("load_track", { path: filePath });
    await invoke("play");
  } catch (err) {
    console.error("Failed to load track:", err);
  }
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
    await invoke("set_playlist_tracks", { tracks: currentTracks as unknown as Record<string, unknown>[] });
    renderPlaylist();
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

async function loadMixConfig() {
  try {
    const config: { pattern: string; duration_secs: number } = await invoke("get_mix_config");
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
    const plugins: PluginInfo[] = await invoke("get_plugins");
    const nodes: GraphNode[] = await invoke("get_graph_nodes");

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
          await invoke("enable_plugin", { nodeId: node.id, enabled: toggle.checked });
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
      await invoke("reorder_plugins", { order: newOrder });
    } catch (err) {
      console.error("Reorder failed:", err);
    }
  });
}

async function loadPluginUi(pluginIndex: number) {
  const container = document.getElementById("plugin-ui-container");
  if (!container) return;

  try {
    const html: string = await invoke("get_plugin_ui", { pluginIndex });
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
        await invoke("set_plugin_parameter", {
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
