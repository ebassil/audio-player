import { invoke } from "@tauri-apps/api/core";

document.addEventListener("DOMContentLoaded", () => {
  const app = document.getElementById("app")!;
  app.innerHTML = `<h1>Audio Player</h1>`;
});
