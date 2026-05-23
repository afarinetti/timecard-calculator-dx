# UI Rework Design Spec
**Date:** 2026-05-23  
**Status:** Approved

## Overview

Rework the Timecard Calculator UI from a generic DaisyUI dark-theme app into a polished, consistent "Precision Dark" design. The app runs as a Dioxus 0.7 desktop app using Tailwind CSS v4 + DaisyUI with the dark theme.

---

## 1. Design System

### Color Palette

| Role | Value | Usage |
|---|---|---|
| `bg-base` | `#0d1117` | App background |
| `bg-panel` | `#161b22` | Sidebar, cards, drawer, table headers |
| `border` | `#21262d` | Dividers, card borders, table row separators |
| `border-active` | `#30363d` | Input borders, button borders, hover borders |
| `text-primary` | `#e6edf3` | Main content, headings |
| `text-muted` | `#8b949e` | Labels, placeholders, secondary text |
| `accent-blue` | `#58a6ff` | Active nav, links, TW badge, focus rings |
| `accent-blue-bg` | `#1f6feb22` | Active nav background, TW badge background |
| `accent-green` | `#238636` | Primary action buttons (Add, Create, Save) |
| `accent-green-text` | `#3fb950` | REG hour type label, pay period ok state |
| `accent-amber` | `#d29922` | OT hour type label, In Progress badge |
| `accent-red` | `#f85149` | Delete button hover, error states |

### Typography

- **UI chrome** (labels, nav, buttons, headings): `system-ui, -apple-system, sans-serif`
- **Data values** (times, hour counts, WBS codes): `ui-monospace, 'SF Mono', Menlo, monospace`
- **Column headers**: 10px, uppercase, 0.07em letter-spacing, `text-muted`
- **Stat card values**: 28px monospace bold

### Spacing & Radii

- Card/panel border-radius: `8px`
- Button/input border-radius: `6px`
- Badge border-radius: `4px`
- Borders: `1px solid` throughout
- Table cell padding: `11px 16px`

---

## 2. Layout

### Shell

A **56px left sidebar** replaces the top navbar:

- **Logo mark**: 28×28px blue (`#58a6ff`) rounded square with white "T", 14px bold
- **Nav items**: 44×auto pill, icon (15px) + label (9px uppercase) stacked vertically
  - Active: `bg-panel` tinted blue (`#1f6feb22`), `1px solid #1f6feb44` border, blue text/icon
  - Inactive: muted color, transparent background; hover darkens slightly
- **Pages**: Dashboard (grid icon `⊞`) and Settings (gear icon `⚙`)

Main content area: `flex: 1`, `background: #0d1117`, `padding: 24px 28px`, scrollable.

---

## 3. Dashboard Page

### Stat Cards

Three equal-width cards in a CSS grid row, above all other content:

| Card | Content |
|---|---|
| Today | Hours logged today |
| This Week | Hours logged this week |
| Pay Period | Total hours this pay period |

Each card: `bg-panel`, `1px border`, `8px radius`, `14px 16px` padding. Label: 10px uppercase muted. Value: 28px monospace bold. Pay Period card value is always displayed in `accent-green-text` (`#3fb950`).

### Date Navigation Row

Below stats, one flex row:
- Left: `‹` nav button · `May 23, 2026` date label (14px bold) · `›` nav button · `Today` pill
- Right: `+ Add Entry` green button

Nav buttons: `1px solid #30363d` border, ghost background. Hover: blue border + blue text. Today pill: same border style.

Week tab replaces the date label with `Week of May 19` and the nav buttons advance by week.

### Tab Bar

Underline-style tabs: Day · Week · Pay Period · History

- Active tab: `#58a6ff` text, `2px solid #58a6ff` bottom border
- Inactive: `text-muted`, transparent border
- Row has a `1px solid #21262d` bottom border

### Entry Table

Columns: **Code** · **Type** · **Start → End** · **Hrs** · **Actions**

| Column | Detail |
|---|---|
| Code | Labor code *name* (not WBS). If `telework=true`, render a `TW` badge inline: blue tinted bg, blue border, 10px bold |
| Type | Hour type code (`REG`, `OT`, etc.) in monospace bold, color-coded: REG=`#3fb950`, OT=`#d29922` |
| Start → End | `08:00 → 17:00` in monospace muted. If no end time: show amber `In Progress` badge instead of the end time |
| Hrs | Decimal hours in monospace bold. `—` (muted) when no end time |
| Actions | Always-visible **Edit** and **Delete** bordered buttons. Edit hover: blue. Delete hover: red. |

Table: `1px solid #21262d` border, `8px` radius, collapsed. Header row uses `bg-panel`. Row hover: slightly lighter background. No zebra striping.

Below the table, right-aligned: `Total: 10.50 hrs` (value in monospace bold).

Empty state: centered muted paragraph, no table chrome rendered.

---

## 4. Entry Form Drawer

A **320px right-side drawer** rendered alongside the dashboard content (no CSS animation — Dioxus conditional renders don't easily support transitions). When `show_form` is true, the main content dims to ~35% opacity via an overlay div, and the drawer panel appears fixed to the right edge. A left-side box-shadow (`-20px 0 60px rgba(0,0,0,0.47)`) visually separates it from the content.

### Header

`bg-panel`, `1px border-bottom`. Title: "New Entry" or "Edit Entry" (15px bold). × close button (ghost, 20px).

### Body (scrollable)

Fields in order, 16px gaps:

1. **Mode toggle** — segmented control: `Time Inputs` / `Duration`. Dark background, active segment gets `#21262d` fill.

2. **Labor Code** — DaisyUI `select select-bordered`. Dark background, chevron arrow via SVG background-image. Focus: `#58a6ff` border + blue glow ring. Placeholder option: muted text.

3. **Hour Type** — same DaisyUI select style.

4. **Start / End** — two-column grid:
   - Both inputs: `type="time"`, monospace font, dark background, `1px border`
   - **Start** has a `Now` button overlaid at the right edge: sets value to `floor((now - 15min) / 15min) * 15min` — i.e. current time minus 15 minutes, rounded to the nearest 15-minute mark
   - **End** has a `Now` button overlaid at the right edge: sets value to `ceil((now + 15min) / 15min) * 15min` — i.e. current time plus 15 minutes, rounded up to the next 15-minute boundary
   - `Now` button style: `#21262d` background, `1px solid #30363d` border, muted text. Hover: blue border + blue text.
   - In Duration mode: End field is replaced by a number input (hours, step 0.25).

5. **Telework** — toggle row: label left, DaisyUI `toggle toggle-primary` right.

Error banner (if any): appears above the footer, `alert alert-error` style.

### Footer

`1px border-top`, `14px 20px` padding. `Cancel` ghost button + `Create` / `Update` full-width green button.

---

## 5. Settings Page

### Layout

Two-column within the main content area:

- **Settings sub-nav** (180px, `#0d1117` bg, `1px right border`): vertical list of category items
- **Content pane** (flex: 1, `28px 32px` padding): shows the active category

### Sub-nav Items

Categories: Pay Period · Labor Codes · Hour Types · Import / Export

Item style: `7px 10px` padding, `6px` radius, 13px medium. Active: blue-tinted bg + border, blue text. Inactive: muted; hover darkens.

### Content Pane Structure

Each pane:
- **Title**: 18px bold
- **Description**: 13px muted, `1.5` line-height
- **Add form**: contained in a `bg-panel` card (`16px` padding, `8px` radius, `1px border`). Fields inline with an Add button.
- **Table**: same column-header style as dashboard table; `1px border-bottom` row separators; always-visible action buttons; no outer border/card wrapper.

### Pane Specifics

**Pay Period**: Date input + Add Anchor button. Table: Start Date · Actions (Remove only).

**Labor Codes**: WBS Number input (narrow) + Name input (flex-1) + Add/Update button + Cancel button (when editing). Table: WBS (monospace code chip) · Name · Actions (Edit + Delete).

**Hour Types**: Code input (narrow) + Name input + Add/Update + Cancel. Table: Code (colored type badge) · Name · Actions (Edit + Delete).

**Import / Export**: Description paragraph with JSON format. Two outlined buttons: Import JSON (file picker) + Export JSON (save dialog).

---

## 6. Unchanged

- Routing (`Route` enum, `Layout` wrapper replaced by sidebar shell)
- All data fetching logic (`use_resource`, `Repository` calls)
- API layer (`api` crate) — no changes
- Modal backdrop click-to-close → drawer click-outside-to-close
- `use_context` signals for `current_date`, `current_week`, lookup data

---

## 7. Implementation Notes

- `tailwind.css`: keep `@plugin "daisyui" { themes: dark --default; }`. Override DaisyUI CSS variables to match the Precision Dark palette (primary = `#58a6ff`, base-100 = `#0d1117`, base-200 = `#161b22`, base-300 = `#21262d`).
- Replace `dialog.modal` + `modal-box` with a flex-positioned drawer div (no DaisyUI modal primitives for the entry form).
- Settings page: replace stacked `card` divs with the two-column sub-nav layout.
- Entry table: remove `table-zebra`; use explicit `border-bottom` rows.
- All action buttons: remove hover-only opacity trick; buttons are always rendered.
