import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

let exportFormEl: HTMLFormElement | null;
let itemcachePathEl: HTMLInputElement | null;
let outputSqlPathEl: HTMLInputElement | null;
let exportMsgEl: HTMLElement | null;
let browseItemcacheBtnEl: HTMLButtonElement | null;
let browseOutputBtnEl: HTMLButtonElement | null;
let exportBtnEl: HTMLButtonElement | null;

function setStatus(message: string, type: "neutral" | "success" | "error" = "neutral"): void {
  if (!exportMsgEl) return;
  exportMsgEl.textContent = message;
  exportMsgEl.classList.remove("success", "error");
  if (type === "success") exportMsgEl.classList.add("success");
  if (type === "error") exportMsgEl.classList.add("error");
}

function setBusyState(isBusy: boolean): void {
  if (browseItemcacheBtnEl) browseItemcacheBtnEl.disabled = isBusy;
  if (browseOutputBtnEl) browseOutputBtnEl.disabled = isBusy;
  if (exportBtnEl) {
    exportBtnEl.disabled = isBusy;
    exportBtnEl.textContent = isBusy ? "Exporting..." : "Export SQL";
  }
}

window.addEventListener("DOMContentLoaded", () => {
  exportFormEl = document.querySelector("#export-form");
  itemcachePathEl = document.querySelector("#itemcache-path");
  outputSqlPathEl = document.querySelector("#output-sql-path");
  exportMsgEl = document.querySelector("#export-msg");
  browseItemcacheBtnEl = document.querySelector("#browse-itemcache-btn");
  browseOutputBtnEl = document.querySelector("#browse-output-btn");
  exportBtnEl = document.querySelector("#export-btn");

  browseItemcacheBtnEl?.addEventListener("click", async () => {
    if (!itemcachePathEl) return;
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "WDB Files", extensions: ["wdb"] }],
      });
      if (typeof selected === "string") {
        itemcachePathEl.value = selected;
      }
    } catch (err) {
      setStatus(`Browse failed: ${String(err)}`, "error");
    }
  });

  browseOutputBtnEl?.addEventListener("click", async () => {
    if (!outputSqlPathEl) return;
    try {
      const selected = await save({
        filters: [{ name: "SQL Files", extensions: ["sql"] }],
      });
      if (selected) {
        outputSqlPathEl.value = selected;
      }
    } catch (err) {
      setStatus(`Browse failed: ${String(err)}`, "error");
    }
  });

  exportFormEl?.addEventListener("submit", async (e) => {
    e.preventDefault();
    if (!exportMsgEl || !itemcachePathEl || !outputSqlPathEl) return;

    const itemcachePath = itemcachePathEl.value.trim();
    const outputSqlPath = outputSqlPathEl.value.trim();

    if (!itemcachePath || !outputSqlPath) {
      setStatus("Please provide both input and output paths.", "error");
      return;
    }

    setBusyState(true);
    setStatus("Export in progress...");
    try {
      const count = await invoke("export_itemcache_to_item_template_sql", {
        itemcachePath,
        outputSqlPath,
      });
      setStatus(`Done. Exported ${count} items.`, "success");
    } catch (err) {
      setStatus(`Export failed: ${String(err)}`, "error");
    } finally {
      setBusyState(false);
    }
  });
});
