# Dayplanner TUI

A high-performance, keyboard-driven Day Planner Terminal User Interface (TUI) built in Rust using [Ratatui](https://github.com/ratatui/ratatui) and native **Vim motions**.

---

## Features

- **Microsoft Teams-Style Calendar**:
  - Vertical 24-hour scrollable timeline with continuous `00:00` to `23:59` navigation.
  - Multi-day columns: **Day View** (`1`), **3-Day View** (`3`), and **Week View** (`w` or `W`).
  - Block Inspector sidebar (`<Tab>`) showing selected event details, descriptions, notes, and daily planned vs. free time statistics.
- **True Vim Modal Engine**:
  - `NORMAL`: `h`/`j`/`k`/`l` grid navigation, `Ctrl-d`/`Ctrl-u` 4-hour jumps, `gg`/`G` top/bottom of day, `t` jump to now, `w`/`b` jump between blocks.
  - `VISUAL`: `v` + `j`/`k` to select time ranges, `c` to turn selection into an event block.
  - `INSERT`: Form editor for adding and editing events (`i`/`a`/`c`) with title, category color-picker, start/end time, and notes.
  - `COMMAND`: `:` command line (`:w` to sync, `:q` to exit, `:db <path>` to switch database, `:goto <date>` to jump dates).
- **Easily Accessible Granularity / Zoom**:
  - Header displays active time resolution: `[15m]`, `[30m]`, or `[60m]`.
  - Press `+` (or `=`) to zoom in (finer resolution).
  - Press `-` to zoom out (broader view).
  - Press `z` for a quick 1-2-3 resolution picker popup.
- **Block Manipulation**:
  - `J` / `K` (Shift+j/k): Nudge block start time ±15 minutes.
  - `>` / `<`: Increase or decrease block duration ±15 minutes.
  - `dd`: Delete selected block.
  - `y` / `p`: Yank (copy) and paste block to cursor position.
- **Portable Single-Binary SQLite**:
  - Embedded SQLite (`rusqlite` bundled) with zero external C library dependencies.
  - First-run setup wizard offering `/home/lmb/documents/leeosvault/misc/dayplanner.db`, standard XDG data directory, custom path, or temporary session.

---

## Keybindings Quick Reference

| Key | Mode | Action |
| :--- | :--- | :--- |
| `h` / `l` | Normal | Navigate day left / right |
| `j` / `k` | Normal | Navigate time slot up / down |
| `Ctrl-d` / `Ctrl-u` | Normal | Jump 4 hours down / up |
| `gg` / `G` | Normal | Jump to 00:00 / 23:59 |
| `t` | Normal | Snap directly to current live time |
| `w` / `b` | Normal | Jump to next / previous scheduled block |
| `1` / `3` / `W` | Normal | Switch between Day, 3-Day, and Week view |
| `+` / `-` | Normal | Zoom time resolution in / out (15m $\leftrightarrow$ 30m $\leftrightarrow$ 60m) |
| `z` | Normal | Open quick granularity picker modal |
| `Tab` | Normal | Toggle Inspector sidebar pane |
| `i` / `a` | Normal | Insert new block at cursor slot |
| `c` / `cw` | Normal | Edit currently selected block |
| `dd` | Normal | Delete selected block |
| `J` / `K` | Normal | Nudge block start time forward/backward 15m |
| `>` / `<` | Normal | Extend / shrink block duration 15m |
| `y` / `p` | Normal | Yank (copy) & paste block |
| `v` | Normal | Enter Visual selection mode |
| `:` | Normal | Enter Command mode |
| `?` | Normal | Open Vim cheat sheet overlay |
| `q` | Normal | Quit Dayplanner |

---

## Running Dayplanner

### Standard Run
```bash
cargo run
```

### Run with Specific Database Path
```bash
cargo run -- --db /home/lmb/documents/leeosvault/misc/dayplanner.db
```

### Ephemeral / Temporary Session
```bash
cargo run -- --temp
```

### Build Optimized Binary
```bash
cargo build --release
./target/release/dayplanner
```
