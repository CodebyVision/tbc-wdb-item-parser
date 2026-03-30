# TBC WDB Item Parser

<img width="1188" height="919" alt="image" src="https://github.com/user-attachments/assets/c442694e-0bfc-4402-baff-497e4dae7c59" />

Desktop app built with Tauri + TypeScript to parse `itemcache.wdb` files and export SQL output for item template workflows.

## Stack

- Frontend: Vite + TypeScript
- Desktop shell / backend commands: Tauri v2 (Rust)
- Dialogs: `@tauri-apps/plugin-dialog`

## Prerequisites

- [Node.js](https://nodejs.org/) 18+ and npm
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- Platform dependencies required by [Tauri v2](https://v2.tauri.app/start/prerequisites/)

## Install

```bash
npm install
```

## Development

Run the desktop app in dev mode:

```bash
npm run tauri dev
```

This starts Vite and launches the Tauri window.

## Build

Build frontend assets:

```bash
npm run build
```

Build distributable desktop bundles:

```bash
npm run tauri build
```

## Usage

1. Launch the app.
2. Select an input `.wdb` file (`itemcache.wdb`).
3. Choose an output `.sql` file path.
4. Click export to generate SQL and view exported item count.

## Project Structure

- `src/`: frontend TypeScript, styles, and app UI logic
- `src-tauri/`: Rust command implementation and Tauri config
- `src-tauri/src/bin/export_sql.rs`: CLI-style export binary
