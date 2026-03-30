import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

let exportFormEl: HTMLFormElement | null;
let itemcachePathEl: HTMLInputElement | null;
let outputSqlPathEl: HTMLInputElement | null;
let exportMsgEl: HTMLElement | null;
let browseItemcacheBtnEl: HTMLButtonElement | null;
let browseOutputBtnEl: HTMLButtonElement | null;
let exportBtnEl: HTMLButtonElement | null;
let useReplaceEl: HTMLInputElement | null;

const ITEMCACHE_PATH_KEY = "tbc-wdb-parser:itemcachePath";
const OUTPUT_SQL_PATH_KEY = "tbc-wdb-parser:outputSqlPath";
const USE_REPLACE_KEY = "tbc-wdb-parser:useReplace";

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

function saveRememberedPaths(): void {
  if (itemcachePathEl) {
    localStorage.setItem(ITEMCACHE_PATH_KEY, itemcachePathEl.value.trim());
  }
  if (outputSqlPathEl) {
    localStorage.setItem(OUTPUT_SQL_PATH_KEY, outputSqlPathEl.value.trim());
  }
  if (useReplaceEl) {
    localStorage.setItem(USE_REPLACE_KEY, useReplaceEl.checked ? "1" : "0");
  }
}

function loadRememberedPaths(): void {
  if (itemcachePathEl) {
    itemcachePathEl.value = localStorage.getItem(ITEMCACHE_PATH_KEY) ?? "";
  }
  if (outputSqlPathEl) {
    outputSqlPathEl.value = localStorage.getItem(OUTPUT_SQL_PATH_KEY) ?? "";
  }
  if (useReplaceEl) {
    useReplaceEl.checked = localStorage.getItem(USE_REPLACE_KEY) === "1";
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
  useReplaceEl = document.querySelector("#use-replace");
  loadRememberedPaths();

  itemcachePathEl?.addEventListener("input", saveRememberedPaths);
  outputSqlPathEl?.addEventListener("input", saveRememberedPaths);
  useReplaceEl?.addEventListener("change", saveRememberedPaths);

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
        saveRememberedPaths();
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
        saveRememberedPaths();
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
    const useReplace = useReplaceEl?.checked ?? false;
    const sqlMode = useReplace ? "REPLACE" : "INSERT";

    if (!itemcachePath || !outputSqlPath) {
      setStatus("Please provide both input and output paths.", "error");
      return;
    }

    saveRememberedPaths();
    setBusyState(true);
    setStatus(`Export in progress... Mode: ${sqlMode}`);
    try {
      const count = await invoke("export_itemcache_to_item_template_sql", {
        itemcachePath,
        outputSqlPath,
        useReplace,
      });
      setStatus(`Done. Exported ${count} items.`, "success");
    } catch (err) {
      setStatus(`Export failed: ${String(err)}`, "error");
    } finally {
      setBusyState(false);
    }
  });
});
