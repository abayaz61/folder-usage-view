# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Disk Usage Analyzer** (`dua`) — a high-performance TUI disk space analyzer in Rust with interactive treemap visualization, file list, and live scanning. Single portable binary targeting Windows, Linux, and macOS.

## Build & Run Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build (LTO + strip)
cargo run --release -- --path "C:\"  # Run on Windows
cargo run --release -- --path /home  # Run on Linux/macOS
cargo test                           # Run tests (treemap unit tests)
cargo clippy                         # Lint
```

**CLI flags**: `-p/--path <PATH>` (default `.`), `--allow-delete`, `--follow-symlinks`, `--show-hidden`, `--elevated` (internal UAC flag).

## Architecture

### Core Data Flow

Scanner thread → `crossbeam_channel` → UI event loop (60fps at 16ms tick rate)

`ParallelScanner` uses `jwalk` + `rayon` in a spawned thread, sending `ScanMessage` variants (`Entry`, `Progress`, `Completed`, `Error`) over an unbounded channel. The UI calls `app.process_scan_messages()` each tick. Cancellation uses `Arc<AtomicBool>`.

### Tree Storage

`FileTree` is an arena-based tree using `SlotMap<NodeId, TreeNode>` with a `HashMap<PathBuf, NodeId>` path index. `TreeNode.children` uses `SmallVec<[NodeId; 8]>` for stack allocation of small child lists.

### Module Layout

- **`app/`** — `App` (central state), `Config` (builder pattern), `Settings` (JSON-persisted), `AppMode` state machine, history
- **`model/`** — `TreeNode`, `FileTree` (slotmap arena), `TreeStatistics`, `DriveInfo`
- **`scanner/`** — `ParallelScanner`, `ScannedEntry`, `ScanMessage` channel protocol
- **`treemap/`** — Squarified treemap layout algorithm (from-scratch implementation with unit tests)
- **`platform/`** — `PlatformOps` trait with `#[cfg]`-gated impls for Windows/Linux/macOS (context menus, PATH registration, shortcuts, elevation)
- **`ui/`** — `run_app()` event loop, `render_ui()`, all widgets in `ui/widgets/`
- **`actions/`** — File deletion (uses `trash` crate for recycle bin)
- **`util/`** — Size formatting, string truncation, i18n (`Language` enum with English/Turkish string tables)

### AppMode State Machine

`Scanning` → `Browsing` → `ComputerView` | `Help` | `About` | `Settings` | `DeleteConfirm` | `DriveSelect` | `Error` | `Quitting`

### UI Layout (ratatui 0.30 + crossterm 0.29)

Three-zone vertical: header (3 rows) / content (flexible) / footer (3 rows). Content splits by `ViewMode`: Treemap (full), List (60/40 file list + stats), Split (40/30/30 treemap + list + stats). Overlays use `Clear` widget + `centered_rect()`.

All widgets implement `ratatui::widgets::Widget` trait and receive `&App` at construction. Theme system has 10 color palettes; icon system has Unicode/ASCII modes.

### Platform Layer

Each platform file (`windows.rs`, `linux.rs`, `macos.rs`) implements `PlatformOps` functions for: context menu registration, PATH installation, shortcut/desktop entry creation, admin detection, and elevation. Windows uses registry + PowerShell + Win32 console font APIs. Linux uses Nautilus/KDE scripts + `.desktop` files. macOS uses Automator workflows + `.app` bundles + `osascript`.

`SettingsCache` caches expensive platform queries (registry, process spawns) — only the toggled item's cache is refreshed after changes.

### i18n

`Strings::new(lang)` returns a `HashMap<&'static str, &'static str>` from static string tables. UI code calls `s.get("key")` with key-as-fallback. Two languages: English (default), Turkish.

### Concurrency Pattern

During scanning, if the user navigates into an unscanned directory, `tree.populate_children_from_fs()` reads the filesystem directly to show immediate children before the background scanner reaches them.

## Settings Persistence

JSON at platform-specific paths: `%APPDATA%\folder-usage-view\settings.json` (Windows), `~/.config/folder-usage-view/settings.json` (Linux/macOS).
