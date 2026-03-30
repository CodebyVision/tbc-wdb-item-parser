import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

let exportFormEl: HTMLFormElement | null;
let itemcachePathEl: HTMLInputElement | null;
let outputSqlPathEl: HTMLInputElement | null;
let exportMsgEl: HTMLElement | null;
let browseItemcacheBtnEl: HTMLButtonElement | null;
let browseOutputBtnEl: HTMLButtonElement | null;

window.addEventListener("DOMContentLoaded", () => {
  exportFormEl = document.querySelector("#export-form");
  itemcachePathEl = document.querySelector("#itemcache-path");
  outputSqlPathEl = document.querySelector("#output-sql-path");
  exportMsgEl = document.querySelector("#export-msg");
  browseItemcacheBtnEl = document.querySelector("#browse-itemcache-btn");
  browseOutputBtnEl = document.querySelector("#browse-output-btn");

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
      if (exportMsgEl) {
        exportMsgEl.textContent = `Browse failed: ${String(err)}`;
      }
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
      if (exportMsgEl) {
        exportMsgEl.textContent = `Browse failed: ${String(err)}`;
      }
    }
  });

  exportFormEl?.addEventListener("submit", async (e) => {
    e.preventDefault();
    if (!exportMsgEl || !itemcachePathEl || !outputSqlPathEl) return;

    const itemcachePath = itemcachePathEl.value;
    const outputSqlPath = outputSqlPathEl.value;

    exportMsgEl.textContent = "Exporting...";
    try {
      const count = await invoke("export_itemcache_to_item_template_sql", {
        itemcachePath,
        outputSqlPath,
      });
      exportMsgEl.textContent = `Done. Exported ${count} items.`;
    } catch (err) {
      exportMsgEl.textContent = `Export failed: ${String(err)}`;
    }
  });
});
