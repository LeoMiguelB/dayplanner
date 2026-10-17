# Dayplanner TUI: Screen Wireframes & UI Specification

This document presents the visual wireframes and layout blueprints for each pane, view, and modal in the **Dayplanner** TUI.

---

## 1. Global Screen Anatomy

Every view shares a consistent layout shell divided into 3 primary zones:
1. **Header Bar**: Active date range, current view mode, date switcher, and **Granularity/Zoom indicator** (`15m` / `30m` / `60m`).
2. **Main Workspace**: The scrollable 24-hour time grid (or split grid + Inspector pane).
3. **Footer Bar**: Vim Mode badge (`[NORMAL]`, `[VISUAL]`, etc.), selection details, active DB path, and command prompt.

---

## 2. Screen 1: Week Planner View (Microsoft Teams Style)

This is the primary dashboard view. It displays 7 days (or 5 workdays) side-by-side with a vertical 24-hour scrollable time ruler.

```text
┌─[ Dayplanner ]── Week 42 (Oct 12 – Oct 18, 2026) ───────────────────[? Help]─┐
│ View: [1]Day  [3]3-Day  [*W]Week │ Zoom: [-] [15m] [+] (z) │ < Prev  Today  Next > │
├───────┬────────────┬────────────┬────────────┬────────────┬────────────┬──────┤
│ Time  │ Mon 10/12  │ Tue 10/13  │ Wed 10/14  │ Thu 10/15  │ Fri 10/16  │ Sat..│
├───────┼────────────┼────────────┼────────────┼────────────┼────────────┼──────┤
│ 07:00 │            │            │            │            │            │      │
│ 07:30 │            │            │            │            │            │      │
│ 08:00 │┌──────────┐│            │┌──────────┐│            │┌──────────┐│      │
│ 08:30 ││Gym / Run ││            ││Sprint Pl.││            ││Standup   ││      │
│ 09:00 │└──────────┘│┌──────────┐│└──────────┘│            │└──────────┘│      │
│ 09:30 │            ││Client    ││            │┌──────────┐│            │      │
│ 10:00 │┌──────────┐││Review    ││            ││1:1 Sync  ││┌──────────┐│      │
│ 10:30 ││Deep Work ││└──────────┘│┌──────────┐│└──────────┘││Research  ││▓ Cursor
│ 11:00 ││(Ratatui) ││            ││Design Sys││            ││NVMe Boot ││      │
│ 11:30 ││          ││            ││Refactor  ││            │└──────────┘│      │
│ 12:00 │└──────────┘│┌──────────┐│└──────────┘│┌──────────┐│            │      │
│ 12:30 │            ││Lunch     ││            ││Team Lunch││            │      │
│ 13:00 │            │└──────────┘│            │└──────────┘│            │      │
│ 13:30 │            │            │            │            │            │      │
│ 14:00 │            │            │            │            │            │      │
│  ▼    │            │            │            │            │            │  ▼   │
├───────┴────────────┴────────────┴────────────┴────────────┴────────────┴──────┤
│ [NORMAL] Sat 10/17 10:30 - 11:30 │ "Research NVMe Boot" [Tag: Tech]           │
│ DB: ~/documents/leeosvault/misc/dayplanner.db                   │ Pos: 10:30 │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Granularity & Zoom Control
- **Quick Motion**: Press `+` or `=` to zoom in (finer resolution: `60m` $\rightarrow$ `30m` $\rightarrow$ `15m`).
- Press `-` or `_` to zoom out (broader overview: `15m` $\rightarrow$ `30m` $\rightarrow$ `60m`).
- Press `z` to open a quick resolution popup menu (`1: 15m`, `2: 30m`, `3: 60m`).
- The grid dynamically recalculates row heights and slot steps without losing cursor position.

---

## 3. Screen 2: Single Day View with Collapsible Inspector Pane

For focused daily execution, press `1` (or `<Tab>` to toggle the side inspector). Overlapping meetings are laid out side-by-side.

```text
┌─[ Dayplanner ]── Saturday, October 17, 2026 ────────────────────────[? Help]─┐
│ View: [*1]Day  [3]3-Day  [W]Week │ Zoom: [15m] (z) │ < [h] Prev   [t] Today   [l] Next > │
├───────┬──────────────────────────────────────────┬────────────────────────────┤
│ Time  │ Schedule Grid (Scroll: 24h ▲/▼)          │ Block Inspector  [Tab: Hide]│
├───────┼──────────────────────────────────────────┼────────────────────────────┤
│ 08:00 │                                          │ ▣ SELECTED BLOCK           │
│ 08:15 │                                          │ Title: Architecture Plan   │
│ 08:30 │ ┌──────────────────────────────────────┐ │ Category: [Deep Work]      │
│ 08:45 │ │ Morning Coffee & Review              │ │ Color: #00AFF (Cyan)       │
│ 09:00 │ └──────────────────────────────────────┘ │ Date: Sat, Oct 17, 2026    │
│ 09:15 │                                          │ Start: 10:00  End: 11:30   │
│ 09:30 │                                          │ Duration: 1h 30m           │
│ 09:45 │                                          │                            │
│ 10:00 │ ┌───────────────────┬──────────────────┐ │ Notes:                     │
│ 10:15 │ │ Architecture Plan │ Pair Review      │ │ - Wireframe review with    │
│ 10:30 │ │ [Deep Work]       │ [Collab]         │ │   stakeholders             │
│ 10:45 │ │                   │                  │ │ - Confirm SQLite schema    │
│ 11:00 │ │                   └──────────────────┘ │ - Setup Ratatui grid      │
│ 11:15 │ │                                      │ │                            │
│ 11:30 │ └──────────────────────────────────────┘ │ ────────────────────────── │
│ 11:45 │                                          │ 📊 DAY STATS               │
│ 12:00 │ ┌──────────────────────────────────────┐ │ Total Planned:  4h 30m     │
│ 12:15 │ │ Lunch Break                          │ │ Free Time:      7h 30m     │
│ 12:30 │ └──────────────────────────────────────┘ │ Blocks Count:   4          │
│  ▼    │                                          │                            │
├───────┴──────────────────────────────────────────┴────────────────────────────┤
│ [NORMAL] Selected: 10:00 - 11:30 "Architecture Plan"                          │
│ :                                                                             │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Modal 1: Block Creation & Edit Form (`i`, `a`, or `c`)

Triggered by pressing `i` (insert at cursor slot), `c` (edit selected block), or after selecting a range in **VISUAL** mode (`v` + `j`/`k` $\rightarrow$ `c`).

```text
┌─────────────────────────── Edit Time Block ────────────────────────────┐
│                                                                        │
│  Title:       [ Architecture Planning & TUI Layout                   ] │
│                                                                        │
│  Category:    [ Deep Work         ▼ ] (c: Cycle / <Enter>: Pick)       │
│               [■ Deep Work]  [■ Meeting]  [■ Personal]  [■ Health]     │
│                                                                        │
│  Date:        [ 2026-10-17 ]                                           │
│  Start Time:  [ 10:00 ]                  End Time:   [ 11:30 ]         │
│  Duration:    1 hour 30 mins                                           │
│                                                                        │
│  Description / Notes:                                                  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Finalize wireframes for Dayplanner Ratatui interface.            │  │
│  │ Verify 15m/30m granularity toggle and 24h scroll handling.       │  │
│  │                                                                  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│  [ <Enter> Save Block ]       [ <Tab> Next Field ]        [ <Esc> Cancel ]  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Modal 2: First-Run Database Setup Wizard

Appears on first launch if no database or configuration file is found:

```text
┌────────────────────────── Welcome to Dayplanner ──────────────────────────┐
│                                                                           │
│  Dayplanner stores your schedule locally in an embedded SQLite database.  │
│  Please select where you would like your database file to be stored:      │
│                                                                           │
│  ( ) 1. Vault Misc Directory (Recommended)                                │
│         /home/lmb/documents/leeosvault/misc/dayplanner.db                 │
│                                                                           │
│  ( ) 2. Standard User Data Directory (XDG)                                │
│         /home/lmb/.local/share/dayplanner/planner.db                      │
│                                                                           │
│  ( ) 3. Custom Path:                                                      │
│         [ /custom/path/to/my_calendar.db                                ] │
│                                                                           │
│  ( ) 4. Ephemeral / Temporary Session (No persistence)                    │
│         /tmp/dayplanner/temp_session.db                                   │
│                                                                           │
│  [j/k]: Navigate options  |  [Space]: Select  |  [Enter]: Confirm & Launch │
└───────────────────────────────────────────────────────────────────────────┘
```
- If option 1 is selected, Dayplanner checks if `/home/lmb/documents/leeosvault/misc/` exists. If not, it safely creates parent directories.
- The choice is written to `~/.config/dayplanner/config.toml` so future launches open seamlessly.

---

## 6. Modal 3: Quick Granularity / Zoom Switcher (`z`)

Pressing `z` summons a lightweight quick-picker popup directly over the grid:

```text
       ┌─────── Time Granularity ───────┐
       │                                │
       │  (•) 1. 15 Minutes (Precise)   │
       │  ( ) 2. 30 Minutes (Standard)  │
       │  ( ) 3. 60 Minutes (Compact)   │
       │                                │
       │  Tip: Press +/- anytime to zoom│
       │  [1/2/3] Quick select  [Esc]   │
       └────────────────────────────────┘
```

---

## 7. Modal 4: Vim Motions Cheat Sheet (`?`)

Pressing `?` displays an instant reference overlay for all Vim motions:

```text
┌────────────────────────── Dayplanner Keybindings ──────────────────────────┐
│ NAVIGATION                             BLOCK ACTIONS                       │
│  h / l       Move Day (Left/Right)      i / a       New block at slot      │
│  j / k       Move Time (Up/Down)        c / cw      Edit selected block    │
│  Ctrl-d / u  Jump 4 Hours Down/Up       dd          Delete block (u: undo) │
│  gg / G      Start / End of 24h Day     J / K       Nudge block time ±15m  │
│  t           Jump to Current Time       > / <       Resize duration ±15m   │
│  w / b       Jump Next / Prev Block     y / p       Yank (copy) & Paste    │
│                                                                            │
│ VIEW & DISPLAY                         COMMAND MODE (:)                    │
│  1 / 3 / w   Day / 3-Day / Week View    :w          Save / Sync to DB      │
│  + / -       Zoom In / Out (15/30/60m)  :q / :q!    Quit application       │
│  z           Granularity Picker         :db [path]  View/switch database   │
│  Tab         Toggle Inspector Pane      :goto [date]Jump to specific date  │
│  /           Search / Filter Blocks                                        │
│                                                     [ Press <Esc> to Close ]│
└────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Summary of How Your Requests Are Met

1. **Easily Accessible Granularity**:
   - **Header Indicator**: Always shows active resolution (`[15m]`, `[30m]`, or `[60m]`).
   - **Zero-Friction Keys**: Single keystroke `+` or `-` to zoom in/out immediately.
   - **Quick Menu**: Press `z` to pick from the popup list without opening settings.
2. **Full 24-Hour Timeline**:
   - Continuous `00:00` to `23:59` timeline.
   - Smooth vertical scrolling with `j`/`k`, `Ctrl-d`/`Ctrl-u`, and instant jump to current hour with `t`.
   - Scroll indicator `▲`/`▼` and scrollbar track on the right margin.
3. **Teams-Style Multi-Day Planner**:
   - Parallel column layout for days with column-splitting for overlapping events.
