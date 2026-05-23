# Timecard Calculator — Frontend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `packages/ui` — all Dioxus RSX components and pages for the timecard calculator desktop app, styled with TailwindCSS v4 + DaisyUI v5.

**Architecture:** Dioxus 0.7 desktop. Global lookup data shared via Context API (`use_context_provider`). Async DB calls via `use_resource` (re-runs on signal change). All open/closed/active UI state driven by `use_signal` — never by DaisyUI CSS-state classes (see styling convention below).

**Tech Stack:** Dioxus 0.7 (desktop, router), TailwindCSS v4, DaisyUI v5, chrono, chrono-tz, rfd (native file dialog), serde_json

**Depends on:** Backend plan (`packages/api`) must be complete — `api::pool()` and all repository types must be available.

---

## Styling Convention (MUST follow throughout)

Use DaisyUI semantic/structural classes for appearance (`btn`, `card`, `table`, `modal-box`, etc.) but **never use DaisyUI CSS-state selectors** (`modal-open` via `<input type="checkbox">`, `drawer-toggle`, `collapse-open`, `dropdown-open`, etc.). These rely on CSS `:checked` and adjacent-sibling tricks that conflict with Dioxus's VDOM.

**Always control state with `use_signal` and conditional class binding:**

```rust
// Correct
let mut open = use_signal(|| false);
rsx! {
    dialog { class: if open() { "modal modal-open" } else { "modal" }, ... }
    button { class: if tab() == "day" { "tab tab-active" } else { "tab" }, ... }
}

// Wrong — never do this
rsx! { input { r#type: "checkbox", class: "modal-toggle" } }
```

---

## File Map

| File | Role |
|---|---|
| `Dioxus.toml` | Build config: platform=desktop, asset_dir |
| `packages/ui/assets/tailwind.css` | TailwindCSS v4 input with DaisyUI plugin |
| `packages/ui/src/lib.rs` | `pub use App` |
| `packages/ui/src/app.rs` | Root `App` component — context providers + startup load |
| `packages/ui/src/routes.rs` | `Route` enum |
| `packages/ui/src/utils.rs` | Pure date/time utilities (tested) |
| `packages/ui/src/components/mod.rs` | Module declarations |
| `packages/ui/src/components/layout.rs` | DaisyUI navbar + `Outlet` |
| `packages/ui/src/components/entry_table.rs` | Entry list with edit/delete |
| `packages/ui/src/components/entry_form.rs` | Add/edit modal (time + duration modes) |
| `packages/ui/src/pages/mod.rs` | Module declarations |
| `packages/ui/src/pages/dashboard.rs` | Day / Week / Pay Period / History tabs |
| `packages/ui/src/pages/settings.rs` | Lookup CRUD + anchors + import/export |

---

### Task 1: TailwindCSS v4 + DaisyUI setup

**Files:**
- Create: `packages/ui/assets/tailwind.css`
- Create: `Dioxus.toml`

- [ ] **Step 1: Install Tailwind CLI and DaisyUI**

DaisyUI v5 is a CSS plugin for TailwindCSS v4. Run from workspace root:

```bash
npm init -y
npm install --save-dev @tailwindcss/cli daisyui@latest
```

- [ ] **Step 2: Create `packages/ui/assets/tailwind.css`**

```css
@import "tailwindcss";
@plugin "daisyui" {
  themes: dark --default;
}
```

- [ ] **Step 3: Generate compiled CSS**

```bash
npx @tailwindcss/cli -i packages/ui/assets/tailwind.css -o packages/ui/assets/app.css
```

Expected: `packages/ui/assets/app.css` created (~50–200 KB).

- [ ] **Step 4: Create `Dioxus.toml` at workspace root**

```toml
[application]
name = "timecard-calc"
default_platform = "desktop"
out_dir = "dist"
asset_dir = "packages/ui/assets"

[desktop]
```

- [ ] **Step 5: Add a `build:css` script to `package.json`** for development watching

```json
{
  "scripts": {
    "build:css": "npx @tailwindcss/cli -i packages/ui/assets/tailwind.css -o packages/ui/assets/app.css",
    "watch:css": "npx @tailwindcss/cli -i packages/ui/assets/tailwind.css -o packages/ui/assets/app.css --watch"
  }
}
```

- [ ] **Step 6: Add `app.css` to `.gitignore`** (generated file)

```bash
echo "packages/ui/assets/app.css" >> .gitignore
```

- [ ] **Step 7: Commit**

```bash
git add Dioxus.toml packages/ui/assets/tailwind.css .gitignore package.json package-lock.json
git commit -m "feat: TailwindCSS v4 + DaisyUI v5 setup"
```

---

### Task 2: Timezone and date utilities (TDD)

**Files:**
- Create: `packages/ui/src/utils.rs`

- [ ] **Step 1: Write the failing tests**

Create `packages/ui/src/utils.rs` with tests only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate_date_forward() {
        assert_eq!(navigate_date("2026-05-21", 1), "2026-05-22");
    }

    #[test]
    fn navigate_date_backward() {
        assert_eq!(navigate_date("2026-05-21", -1), "2026-05-20");
    }

    #[test]
    fn navigate_date_crosses_month() {
        assert_eq!(navigate_date("2026-05-31", 1), "2026-06-01");
    }

    #[test]
    fn navigate_week_forward() {
        assert_eq!(navigate_week("2026-05-18", 1), "2026-05-25");
    }

    #[test]
    fn navigate_week_backward() {
        assert_eq!(navigate_week("2026-05-25", -1), "2026-05-18");
    }

    #[test]
    fn week_start_for_wednesday() {
        // Wednesday 2026-05-20 → Monday 2026-05-18
        assert_eq!(week_start_for("2026-05-20"), "2026-05-18");
    }

    #[test]
    fn week_start_for_monday_is_itself() {
        assert_eq!(week_start_for("2026-05-18"), "2026-05-18");
    }

    #[test]
    fn utc_to_central_hhmm_converts_correctly() {
        // 14:00 UTC = 08:00 CDT (UTC-6 in May)
        assert_eq!(utc_to_central_hhmm("2026-05-21T14:00:00Z"), "08:00");
    }

    #[test]
    fn utc_to_central_hhmm_invalid_returns_placeholder() {
        assert_eq!(utc_to_central_hhmm("not-a-date"), "??:??");
    }
}
```

- [ ] **Step 2: Add the stubs so it compiles**

Add above the `#[cfg(test)]` block:

```rust
pub fn navigate_date(_date: &str, _delta: i64) -> String { todo!() }
pub fn navigate_week(_week_start: &str, _delta: i64) -> String { todo!() }
pub fn week_start_for(_date: &str) -> String { todo!() }
pub fn utc_to_central_hhmm(_utc_iso: &str) -> String { todo!() }
pub fn today() -> String { todo!() }
pub fn live_elapsed_hours(_utc_start: &str) -> f64 { todo!() }
```

- [ ] **Step 3: Add `packages/ui/src/lib.rs` stub and declare the utils module**

`packages/ui/src/lib.rs`:
```rust
pub mod utils;

use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! { div { "loading..." } }
}
```

- [ ] **Step 4: Run failing tests**

```bash
cargo test -p ui utils 2>&1 | head -20
```

Expected: all fail with `not yet implemented`.

- [ ] **Step 5: Implement the utility functions**

Replace the stub implementations in `utils.rs`:

```rust
use chrono::{Datelike, Duration, NaiveDate};
use chrono_tz::America::Chicago;
use chrono::TimeZone;

/// Navigate a YYYY-MM-DD date string by `delta` days (positive = forward).
pub fn navigate_date(date: &str, delta: i64) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(delta)).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|_| date.to_string())
}

/// Navigate a week start (Monday) by `delta` weeks.
pub fn navigate_week(week_start: &str, delta: i64) -> String {
    navigate_date(week_start, delta * 7)
}

/// Return the Monday of the week containing `date`.
pub fn week_start_for(date: &str) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| {
            let days_from_mon = d.weekday().num_days_from_monday() as i64;
            (d - Duration::days(days_from_mon)).format("%Y-%m-%d").to_string()
        })
        .unwrap_or_else(|_| date.to_string())
}

/// Today's date as YYYY-MM-DD in local system time.
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Convert UTC ISO 8601 timestamp to "HH:MM" in Central time.
/// Returns "??:??" on parse failure.
pub fn utc_to_central_hhmm(utc_iso: &str) -> String {
    let parsed = chrono::DateTime::parse_from_rfc3339(utc_iso)
        .or_else(|_| chrono::DateTime::parse_from_str(utc_iso, "%Y-%m-%dT%H:%M:%SZ"));
    match parsed {
        Ok(dt) => dt.with_timezone(&Chicago).format("%H:%M").to_string(),
        Err(_) => "??:??".to_string(),
    }
}

/// Elapsed decimal hours from `utc_start` to now, rounded to nearest 15 minutes.
pub fn live_elapsed_hours(utc_start: &str) -> f64 {
    let start = chrono::DateTime::parse_from_rfc3339(utc_start)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc));
    match start {
        Some(s) => {
            let mins = (chrono::Utc::now() - s).num_minutes().max(0) as f64;
            (mins / 15.0).round() * 15.0 / 60.0
        }
        None => 0.0,
    }
}
```

- [ ] **Step 6: Run tests — expect all pass**

```bash
cargo test -p ui utils
```

Expected:
```
test utils::tests::navigate_date_forward ... ok
test utils::tests::navigate_date_backward ... ok
test utils::tests::navigate_date_crosses_month ... ok
test utils::tests::navigate_week_forward ... ok
test utils::tests::navigate_week_backward ... ok
test utils::tests::week_start_for_wednesday ... ok
test utils::tests::week_start_for_monday_is_itself ... ok
test utils::tests::utc_to_central_hhmm_converts_correctly ... ok
test utils::tests::utc_to_central_hhmm_invalid_returns_placeholder ... ok
test result: ok. 9 passed; 0 failed
```

- [ ] **Step 7: Commit**

```bash
git add packages/ui/src/utils.rs packages/ui/src/lib.rs
git commit -m "feat: date/timezone utilities with full test suite"
```

---

### Task 3: Routes + App component + context providers

**Files:**
- Create: `packages/ui/src/routes.rs`
- Modify: `packages/ui/src/app.rs` (new file)
- Modify: `packages/ui/src/lib.rs`

- [ ] **Step 1: Create `packages/ui/src/routes.rs`**

```rust
use dioxus::prelude::*;
use crate::pages::{dashboard::Dashboard, settings::Settings};
use crate::components::layout::Layout;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(Layout)]
        #[route("/")]
        Dashboard {},
        #[route("/settings")]
        Settings {},
}
```

- [ ] **Step 2: Create `packages/ui/src/app.rs`**

```rust
use dioxus::prelude::*;
use api::{LaborCode, HourType, PayPeriodAnchor, Repository};
use crate::{routes::Route, utils::{today, week_start_for}};

#[component]
pub fn App() -> Element {
    // Global lookup data — populated once on startup
    let labor_codes   = use_context_provider(|| Signal::new(Vec::<LaborCode>::new()));
    let hour_types    = use_context_provider(|| Signal::new(Vec::<HourType>::new()));
    let anchors       = use_context_provider(|| Signal::new(Vec::<PayPeriodAnchor>::new()));

    // Navigation state
    let today_str = today();
    use_context_provider(|| Signal::new(today_str.clone()));          // current_date: Signal<String>
    use_context_provider(|| Signal::new(week_start_for(&today_str))); // current_week: Signal<String>

    // Load lookup data once on startup
    let _init = use_resource(move || async move {
        let pool = api::pool();
        let repo = Repository::new(pool);
        let mut lc  = labor_codes;
        let mut ht  = hour_types;
        let mut ppa = anchors;
        if let Ok(data) = repo.list_labor_codes().await          { *lc.write()  = data; }
        if let Ok(data) = repo.list_hour_types().await           { *ht.write()  = data; }
        if let Ok(data) = repo.list_pay_period_anchors().await   { *ppa.write() = data; }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/app.css") }
        Router::<Route> {}
    }
}
```

- [ ] **Step 3: Create module stubs needed for the routes to compile**

Create `packages/ui/src/components/mod.rs`:
```rust
pub mod layout;
pub mod entry_table;
pub mod entry_form;
```

Create `packages/ui/src/components/layout.rs` (stub):
```rust
use dioxus::prelude::*;
use crate::routes::Route;

#[component]
pub fn Layout() -> Element {
    rsx! {
        div { class: "min-h-screen bg-base-100",
            Outlet::<Route> {}
        }
    }
}
```

Create `packages/ui/src/pages/mod.rs`:
```rust
pub mod dashboard;
pub mod settings;
```

Create `packages/ui/src/pages/dashboard.rs` (stub):
```rust
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    rsx! { div { "Dashboard" } }
}
```

Create `packages/ui/src/pages/settings.rs` (stub):
```rust
use dioxus::prelude::*;

#[component]
pub fn Settings() -> Element {
    rsx! { div { "Settings" } }
}
```

Create `packages/ui/src/components/entry_table.rs` (stub):
```rust
use dioxus::prelude::*;
use api::TimecardEntryView;

#[component]
pub fn EntryTable(
    entries: Vec<TimecardEntryView>,
    on_edit: EventHandler<TimecardEntryView>,
    on_delete: EventHandler<i64>,
) -> Element {
    rsx! { div { "Entry Table" } }
}
```

Create `packages/ui/src/components/entry_form.rs` (stub):
```rust
use dioxus::prelude::*;
use api::TimecardEntryView;

#[component]
pub fn EntryFormModal(
    show: bool,
    editing: Option<TimecardEntryView>,
    date: String,
    on_close: EventHandler,
    on_saved: EventHandler,
) -> Element {
    rsx! { div {} }
}
```

- [ ] **Step 4: Update `packages/ui/src/lib.rs`**

```rust
pub mod app;
pub mod routes;
pub mod utils;
pub mod components;
pub mod pages;

pub use app::App;
```

- [ ] **Step 5: Verify the workspace compiles**

```bash
cargo check
```

Expected: exits 0.

- [ ] **Step 6: Commit**

```bash
git add packages/ui/src/
git commit -m "feat: routes, App with context providers, all component stubs"
```

---

### Task 4: Layout component

**Files:**
- Modify: `packages/ui/src/components/layout.rs`

- [ ] **Step 1: Replace the layout stub with the full DaisyUI navbar layout**

```rust
use dioxus::prelude::*;
use crate::routes::Route;

#[component]
pub fn Layout() -> Element {
    rsx! {
        div { class: "min-h-screen bg-base-100 text-base-content",
            // Top navbar
            div { class: "navbar bg-base-200 shadow-sm px-4",
                div { class: "flex-1",
                    span { class: "text-xl font-bold", "Timecard Calc" }
                }
                div { class: "flex-none",
                    ul { class: "menu menu-horizontal gap-1",
                        li {
                            Link {
                                to: Route::Dashboard {},
                                class: "btn btn-ghost btn-sm",
                                "Dashboard"
                            }
                        }
                        li {
                            Link {
                                to: Route::Settings {},
                                class: "btn btn-ghost btn-sm",
                                "Settings"
                            }
                        }
                    }
                }
            }
            // Page body
            main { class: "container mx-auto p-6 max-w-5xl",
                Outlet::<Route> {}
            }
        }
    }
}
```

- [ ] **Step 2: Verify**

```bash
cargo check -p ui
```

Expected: exits 0.

- [ ] **Step 3: Commit**

```bash
git add packages/ui/src/components/layout.rs
git commit -m "feat: DaisyUI navbar layout"
```

---

### Task 5: Entry table component

**Files:**
- Modify: `packages/ui/src/components/entry_table.rs`

- [ ] **Step 1: Replace the stub with the full component**

```rust
use dioxus::prelude::*;
use api::TimecardEntryView;
use crate::utils::utc_to_central_hhmm;

#[component]
pub fn EntryTable(
    entries: Vec<TimecardEntryView>,
    on_edit: EventHandler<TimecardEntryView>,
    on_delete: EventHandler<i64>,
) -> Element {
    if entries.is_empty() {
        return rsx! {
            p { class: "text-base-content/50 py-6 text-center text-sm", "No entries." }
        };
    }

    rsx! {
        div { class: "overflow-x-auto",
            table { class: "table table-zebra table-sm w-full",
                thead {
                    tr {
                        th { "WBS" }
                        th { "Type" }
                        th { "Start" }
                        th { "End" }
                        th { "Hours" }
                        th { "TW" }
                        th { }
                    }
                }
                tbody {
                    for entry in entries.iter() {
                        tr { key: "{entry.id}",
                            td { class: "font-mono text-xs", "{entry.wbs_number}" }
                            td { code { class: "badge badge-ghost badge-sm", "{entry.hour_type_code}" } }
                            td { "{utc_to_central_hhmm(&entry.start_time)}" }
                            td {
                                if let Some(ref end) = entry.end_time {
                                    "{utc_to_central_hhmm(end)}"
                                } else {
                                    span { class: "badge badge-warning badge-sm", "In Progress" }
                                }
                            }
                            td {
                                if let Some(h) = entry.decimal_hours {
                                    span { class: "font-semibold", "{h:.2}" }
                                } else {
                                    span { class: "text-base-content/40", "—" }
                                }
                            }
                            td {
                                if entry.telework {
                                    span { class: "badge badge-info badge-xs", "TW" }
                                }
                            }
                            td { class: "flex gap-1",
                                button {
                                    class: "btn btn-xs btn-ghost",
                                    onclick: {
                                        let entry = entry.clone();
                                        move |_| on_edit.call(entry.clone())
                                    },
                                    "Edit"
                                }
                                button {
                                    class: "btn btn-xs btn-ghost text-error",
                                    onclick: {
                                        let id = entry.id;
                                        move |_| on_delete.call(id)
                                    },
                                    "✕"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Verify**

```bash
cargo check -p ui
```

Expected: exits 0.

- [ ] **Step 3: Commit**

```bash
git add packages/ui/src/components/entry_table.rs
git commit -m "feat: entry table component with edit/delete actions"
```

---

### Task 6: Entry form modal

**Files:**
- Modify: `packages/ui/src/components/entry_form.rs`

- [ ] **Step 1: Replace the stub with the full modal component**

```rust
use dioxus::prelude::*;
use api::{
    CreateTimecardEntry, LaborCode, HourType, Repository,
    TimecardEntryView, UpdateTimecardEntry,
};
use crate::utils::utc_to_central_hhmm;

#[derive(Clone, PartialEq)]
enum EntryMode { TimeInputs, Duration }

#[component]
pub fn EntryFormModal(
    show: bool,
    editing: Option<TimecardEntryView>,
    date: String,
    on_close: EventHandler,
    on_saved: EventHandler,
) -> Element {
    let labor_codes = use_context::<Signal<Vec<LaborCode>>>();
    let hour_types  = use_context::<Signal<Vec<HourType>>>();

    let mut mode          = use_signal(|| EntryMode::TimeInputs);
    let mut labor_code_id = use_signal(|| String::new());
    let mut hour_type_id  = use_signal(|| String::new());
    let mut telework      = use_signal(|| false);
    let mut start_time    = use_signal(|| "08:00".to_string());
    let mut end_time      = use_signal(|| "17:00".to_string());
    let mut duration      = use_signal(|| 8.0f64);
    let mut error         = use_signal(|| Option::<String>::None);

    // Populate fields when the modal opens or the editing target changes
    use_effect(move || {
        if let Some(ref e) = editing {
            *labor_code_id.write() = e.labor_code_id.to_string();
            *hour_type_id.write()  = e.hour_type_id.to_string();
            *telework.write()      = e.telework;
            *start_time.write()    = utc_to_central_hhmm(&e.start_time);
            *end_time.write()      = e.end_time.as_deref().map(utc_to_central_hhmm).unwrap_or_default();
            *duration.write()      = e.decimal_hours.unwrap_or(8.0);
        } else {
            *labor_code_id.write() = String::new();
            *hour_type_id.write()  = String::new();
            *telework.write()      = false;
            *start_time.write()    = "08:00".to_string();
            *end_time.write()      = "17:00".to_string();
            *duration.write()      = 8.0;
        }
        *mode.write()  = EntryMode::TimeInputs;
        *error.write() = None;
    });

    let end_from_duration = move |start: &str, dur: f64| -> Option<String> {
        let (h, m): (u32, u32) = {
            let p: Vec<&str> = start.split(':').collect();
            (p.first()?.parse().ok()?, p.get(1)?.parse().ok()?)
        };
        let total = h * 60 + m + (dur * 60.0).round() as u32;
        if total >= 1440 { return None; }
        Some(format!("{:02}:{:02}", total / 60, total % 60))
    };

    let handle_submit = move |_| {
        let lc_id: i64 = match labor_code_id.read().parse() {
            Ok(v) => v,
            Err(_) => { *error.write() = Some("Select a labor code".into()); return; }
        };
        let ht_id: i64 = match hour_type_id.read().parse() {
            Ok(v) => v,
            Err(_) => { *error.write() = Some("Select an hour type".into()); return; }
        };
        let start = start_time.read().clone();
        if start.is_empty() { *error.write() = Some("Start time is required".into()); return; }

        let end = match *mode.read() {
            EntryMode::TimeInputs => {
                let e = end_time.read().clone();
                if e.is_empty() { None } else { Some(e) }
            }
            EntryMode::Duration => end_from_duration(&start, *duration.read()),
        };

        let editing_val = editing.clone();
        let date_val    = date.clone();
        let tw          = *telework.read();
        let mut err     = error;
        let on_saved    = on_saved.clone();
        let on_close    = on_close.clone();

        spawn(async move {
            let pool = api::pool();
            let repo = Repository::new(pool);
            let result = if let Some(ref e) = editing_val {
                repo.update_timecard_entry(&UpdateTimecardEntry {
                    id: e.id, labor_code_id: lc_id, hour_type_id: ht_id,
                    telework: tw, date: date_val,
                    start_time: start, end_time: end,
                }).await.map(|_| ())
            } else {
                repo.create_timecard_entry(&CreateTimecardEntry {
                    labor_code_id: lc_id, hour_type_id: ht_id,
                    telework: tw, date: date_val,
                    start_time: start, end_time: end,
                }).await.map(|_| ())
            };
            match result {
                Ok(_) => { on_saved.call(()); on_close.call(()); }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    rsx! {
        dialog {
            class: if show { "modal modal-open" } else { "modal" },
            div { class: "modal-box w-full max-w-lg",
                h3 { class: "font-bold text-lg mb-4",
                    if editing.is_some() { "Edit Entry" } else { "New Entry" }
                }

                // Mode toggle — use_signal, not DaisyUI checkbox trick
                div { class: "join mb-4",
                    button {
                        class: if *mode.read() == EntryMode::TimeInputs { "btn btn-sm join-item btn-active" } else { "btn btn-sm join-item" },
                        onclick: move |_| *mode.write() = EntryMode::TimeInputs,
                        "Time Inputs"
                    }
                    button {
                        class: if *mode.read() == EntryMode::Duration { "btn btn-sm join-item btn-active" } else { "btn btn-sm join-item" },
                        onclick: move |_| *mode.write() = EntryMode::Duration,
                        "Duration"
                    }
                }

                // Labor Code
                div { class: "form-control mb-3",
                    label { class: "label", span { class: "label-text", "Labor Code" } }
                    select {
                        class: "select select-bordered w-full",
                        value: "{labor_code_id}",
                        onchange: move |e| *labor_code_id.write() = e.value(),
                        option { value: "", disabled: true,
                            selected: labor_code_id.read().is_empty(),
                            "Select labor code"
                        }
                        for lc in labor_codes.read().iter() {
                            option { value: "{lc.id}", "{lc.wbs_number} — {lc.name}" }
                        }
                    }
                }

                // Hour Type
                div { class: "form-control mb-3",
                    label { class: "label", span { class: "label-text", "Hour Type" } }
                    select {
                        class: "select select-bordered w-full",
                        value: "{hour_type_id}",
                        onchange: move |e| *hour_type_id.write() = e.value(),
                        option { value: "", disabled: true,
                            selected: hour_type_id.read().is_empty(),
                            "Select hour type"
                        }
                        for ht in hour_types.read().iter() {
                            option { value: "{ht.id}", "{ht.code} — {ht.name}" }
                        }
                    }
                }

                // Telework
                div { class: "form-control mb-3",
                    label { class: "label cursor-pointer",
                        span { class: "label-text", "Telework" }
                        input {
                            r#type: "checkbox",
                            class: "toggle toggle-primary",
                            checked: *telework.read(),
                            onchange: move |e| *telework.write() = e.checked(),
                        }
                    }
                }

                // Time or Duration inputs
                div { class: "grid grid-cols-2 gap-3 mb-3",
                    div { class: "form-control",
                        label { class: "label", span { class: "label-text", "Start Time" } }
                        input {
                            r#type: "time",
                            class: "input input-bordered",
                            value: "{start_time}",
                            oninput: move |e| *start_time.write() = e.value(),
                        }
                    }
                    if *mode.read() == EntryMode::TimeInputs {
                        div { class: "form-control",
                            label { class: "label", span { class: "label-text", "End Time (optional)" } }
                            input {
                                r#type: "time",
                                class: "input input-bordered",
                                value: "{end_time}",
                                oninput: move |e| *end_time.write() = e.value(),
                            }
                        }
                    } else {
                        div { class: "form-control",
                            label { class: "label", span { class: "label-text", "Duration (hours)" } }
                            input {
                                r#type: "number",
                                class: "input input-bordered",
                                min: "0.25", max: "24", step: "0.25",
                                value: "{duration}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<f64>() { *duration.write() = v; }
                                },
                            }
                        }
                    }
                }

                // Error
                if let Some(ref msg) = *error.read() {
                    div { class: "alert alert-error mb-3", span { "{msg}" } }
                }

                div { class: "modal-action",
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: handle_submit,
                        if editing.is_some() { "Update" } else { "Create" }
                    }
                }
            }
            // Click outside to close
            div {
                class: "modal-backdrop",
                onclick: move |_| on_close.call(()),
            }
        }
    }
}
```

- [ ] **Step 2: Verify**

```bash
cargo check -p ui
```

Expected: exits 0.

- [ ] **Step 3: Commit**

```bash
git add packages/ui/src/components/entry_form.rs
git commit -m "feat: entry form modal with time-inputs and duration modes"
```

---

### Task 7: Dashboard page

**Files:**
- Modify: `packages/ui/src/pages/dashboard.rs`

- [ ] **Step 1: Replace the stub with the full dashboard**

```rust
use dioxus::prelude::*;
use api::{Repository, TimecardEntryView};
use crate::{
    components::{entry_form::EntryFormModal, entry_table::EntryTable},
    utils::{navigate_date, navigate_week, today},
};

#[derive(Clone, PartialEq)]
enum DashTab { Day, Week, PayPeriod, History }

#[component]
pub fn Dashboard() -> Element {
    let mut current_date = use_context::<Signal<String>>();
    let mut current_week = use_context::<Signal<String>>();

    let mut tab           = use_signal(|| DashTab::Day);
    let mut show_form     = use_signal(|| false);
    let mut editing_entry = use_signal(|| Option::<TimecardEntryView>::None);
    let mut error         = use_signal(|| Option::<String>::None);
    let mut reload        = use_signal(|| 0u32);

    // Day data — re-fetches when current_date or reload changes
    let day_data = use_resource(move || {
        let date = current_date.read().clone();
        let _r   = reload();
        async move {
            Repository::new(api::pool()).get_day_summary(&date).await
        }
    });

    // Week data
    let week_data = use_resource(move || {
        let week = current_week.read().clone();
        let _r   = reload();
        async move {
            Repository::new(api::pool()).get_week_summary(&week).await
        }
    });

    let refresh = move || *reload.write() += 1;

    let handle_delete = move |id: i64| {
        spawn(async move {
            match Repository::new(api::pool()).delete_timecard_entry(id).await {
                Ok(_)  => *reload.write() += 1,
                Err(e) => *error.write() = Some(e.to_string()),
            }
        });
    };

    rsx! {
        div { class: "space-y-4",

            // Error banner
            if let Some(ref msg) = *error.read() {
                div { class: "alert alert-error shadow",
                    span { "{msg}" }
                    button {
                        class: "btn btn-xs btn-ghost ml-auto",
                        onclick: move |_| *error.write() = None,
                        "✕"
                    }
                }
            }

            // Navigation + Add button
            div { class: "flex items-center justify-between flex-wrap gap-2",
                div { class: "flex items-center gap-1",
                    if *tab.read() == DashTab::Day {
                        button {
                            class: "btn btn-sm btn-ghost",
                            onclick: move |_| *current_date.write() = navigate_date(&current_date.read(), -1),
                            "‹"
                        }
                        span { class: "text-sm font-semibold px-1", "{current_date}" }
                        button {
                            class: "btn btn-sm btn-ghost",
                            onclick: move |_| *current_date.write() = navigate_date(&current_date.read(), 1),
                            "›"
                        }
                        button {
                            class: "btn btn-xs btn-outline ml-2",
                            onclick: move |_| *current_date.write() = today(),
                            "Today"
                        }
                    }
                    if *tab.read() == DashTab::Week {
                        button {
                            class: "btn btn-sm btn-ghost",
                            onclick: move |_| *current_week.write() = navigate_week(&current_week.read(), -1),
                            "‹"
                        }
                        span { class: "text-sm font-semibold px-1", "Week of {current_week}" }
                        button {
                            class: "btn btn-sm btn-ghost",
                            onclick: move |_| *current_week.write() = navigate_week(&current_week.read(), 1),
                            "›"
                        }
                    }
                }
                button {
                    class: "btn btn-primary btn-sm",
                    onclick: move |_| {
                        *editing_entry.write() = None;
                        *show_form.write() = true;
                    },
                    "+ Add Entry"
                }
            }

            // Tab bar — use_signal drives tab-active, not DaisyUI checkbox trick
            div { class: "tabs tabs-lifted",
                button {
                    class: if *tab.read() == DashTab::Day { "tab tab-active" } else { "tab" },
                    onclick: move |_| *tab.write() = DashTab::Day,
                    "Day"
                }
                button {
                    class: if *tab.read() == DashTab::Week { "tab tab-active" } else { "tab" },
                    onclick: move |_| *tab.write() = DashTab::Week,
                    "Week"
                }
                button {
                    class: if *tab.read() == DashTab::PayPeriod { "tab tab-active" } else { "tab" },
                    onclick: move |_| *tab.write() = DashTab::PayPeriod,
                    "Pay Period"
                }
                button {
                    class: if *tab.read() == DashTab::History { "tab tab-active" } else { "tab" },
                    onclick: move |_| *tab.write() = DashTab::History,
                    "History"
                }
            }

            // Tab panels
            match *tab.read() {
                DashTab::Day => rsx! {
                    match day_data.read().as_ref() {
                        None => rsx! { div { class: "flex justify-center py-8", span { class: "loading loading-spinner" } } },
                        Some(Err(e)) => rsx! { div { class: "alert alert-error", "{e}" } },
                        Some(Ok(summary)) => rsx! {
                            EntryTable {
                                entries: summary.entries.clone(),
                                on_edit: move |entry: TimecardEntryView| {
                                    *editing_entry.write() = Some(entry);
                                    *show_form.write() = true;
                                },
                                on_delete: handle_delete,
                            }
                            div { class: "flex justify-end mt-2 text-sm font-semibold",
                                "Total: {summary.total_hours:.2} hrs"
                            }
                        },
                    }
                },
                DashTab::Week => rsx! {
                    match week_data.read().as_ref() {
                        None => rsx! { div { class: "flex justify-center py-8", span { class: "loading loading-spinner" } } },
                        Some(Err(e)) => rsx! { div { class: "alert alert-error", "{e}" } },
                        Some(Ok(summary)) => rsx! {
                            EntryTable {
                                entries: summary.entries.clone(),
                                on_edit: move |entry: TimecardEntryView| {
                                    *editing_entry.write() = Some(entry);
                                    *show_form.write() = true;
                                },
                                on_delete: handle_delete,
                            }
                            div { class: "flex flex-wrap gap-4 justify-between mt-2 text-sm",
                                div { class: "flex gap-3 text-base-content/60",
                                    for day in summary.by_day.iter() {
                                        span { "{day.date}: {day.total_hours:.2}h" }
                                    }
                                }
                                span { class: "font-semibold", "Total: {summary.total_hours:.2} hrs" }
                            }
                        },
                    }
                },
                DashTab::PayPeriod => rsx! {
                    div { class: "text-base-content/50 py-10 text-center text-sm",
                        "Configure pay period anchors in Settings to enable this view."
                    }
                },
                DashTab::History => rsx! {
                    div { class: "text-base-content/50 py-10 text-center text-sm",
                        "Select a pay period to view historical entries."
                    }
                },
            }

            // Entry form modal
            EntryFormModal {
                show: *show_form.read(),
                editing: editing_entry.read().clone(),
                date: current_date.read().clone(),
                on_close: move |_| *show_form.write() = false,
                on_saved: move |_| { *show_form.write() = false; refresh(); },
            }
        }
    }
}
```

- [ ] **Step 2: Verify**

```bash
cargo check -p ui
```

Expected: exits 0.

- [ ] **Step 3: Smoke test the running app**

In one terminal, watch CSS:
```bash
npm run watch:css
```

In another:
```bash
dx serve --platform desktop
```

Expected: app window opens, Dashboard page visible, tab switching works, "Add Entry" button opens the modal.

- [ ] **Step 4: Commit**

```bash
git add packages/ui/src/pages/dashboard.rs
git commit -m "feat: dashboard page with day/week tabs, navigation, and entry modal"
```

---

### Task 8: Settings page

**Files:**
- Modify: `packages/ui/src/pages/settings.rs`

- [ ] **Step 1: Replace the settings stub with the full page**

```rust
use dioxus::prelude::*;
use api::{
    CreateHourType, CreateLaborCode, HourType, LaborCode,
    PayPeriodAnchor, Repository, UpdateHourType, UpdateLaborCode,
};

#[component]
pub fn Settings() -> Element {
    let mut labor_codes = use_context::<Signal<Vec<LaborCode>>>();
    let mut hour_types  = use_context::<Signal<Vec<HourType>>>();
    let mut anchors     = use_context::<Signal<Vec<PayPeriodAnchor>>>();
    let mut error       = use_signal(|| Option::<String>::None);

    // --- Labor Codes form state ---
    let mut lc_wbs      = use_signal(|| String::new());
    let mut lc_name     = use_signal(|| String::new());
    let mut editing_lc  = use_signal(|| Option::<LaborCode>::None);

    // --- Hour Types form state ---
    let mut ht_code     = use_signal(|| String::new());
    let mut ht_name     = use_signal(|| String::new());
    let mut editing_ht  = use_signal(|| Option::<HourType>::None);

    // --- Pay Period Anchor form state ---
    let mut anchor_date = use_signal(|| String::new());

    // ---- Labor Code handlers ----

    let save_lc = move |_| {
        let wbs  = lc_wbs.read().trim().to_string();
        let name = lc_name.read().trim().to_string();
        if wbs.is_empty() || name.is_empty() { return; }
        let editing = editing_lc.read().clone();
        let mut lc_sig  = labor_codes;
        let mut edit    = editing_lc;
        let mut wbs_s   = lc_wbs;
        let mut name_s  = lc_name;
        let mut err     = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            let result = if let Some(ref e) = editing {
                repo.update_labor_code(&UpdateLaborCode { id: e.id, wbs_number: wbs, name }).await
            } else {
                repo.create_labor_code(&CreateLaborCode { wbs_number: wbs, name }).await
            };
            match result {
                Ok(_) => {
                    if let Ok(d) = repo.list_labor_codes().await { *lc_sig.write() = d; }
                    *edit.write() = None;
                    *wbs_s.write() = String::new();
                    *name_s.write() = String::new();
                }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    let cancel_lc = move |_| {
        *editing_lc.write() = None;
        *lc_wbs.write() = String::new();
        *lc_name.write() = String::new();
    };

    let delete_lc = move |id: i64| {
        let mut lc_sig = labor_codes;
        let mut err    = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            match repo.delete_labor_code(id).await {
                Ok(_)  => { if let Ok(d) = repo.list_labor_codes().await { *lc_sig.write() = d; } }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    // ---- Hour Type handlers ----

    let save_ht = move |_| {
        let code = ht_code.read().trim().to_string();
        let name = ht_name.read().trim().to_string();
        if code.is_empty() || name.is_empty() { return; }
        let editing = editing_ht.read().clone();
        let mut ht_sig  = hour_types;
        let mut edit    = editing_ht;
        let mut code_s  = ht_code;
        let mut name_s  = ht_name;
        let mut err     = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            let result = if let Some(ref e) = editing {
                repo.update_hour_type(&UpdateHourType { id: e.id, code, name }).await
            } else {
                repo.create_hour_type(&CreateHourType { code, name }).await
            };
            match result {
                Ok(_) => {
                    if let Ok(d) = repo.list_hour_types().await { *ht_sig.write() = d; }
                    *edit.write() = None;
                    *code_s.write() = String::new();
                    *name_s.write() = String::new();
                }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    let cancel_ht = move |_| {
        *editing_ht.write() = None;
        *ht_code.write() = String::new();
        *ht_name.write() = String::new();
    };

    let delete_ht = move |id: i64| {
        let mut ht_sig = hour_types;
        let mut err    = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            match repo.delete_hour_type(id).await {
                Ok(_)  => { if let Ok(d) = repo.list_hour_types().await { *ht_sig.write() = d; } }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    // ---- Pay period anchor handlers ----

    let add_anchor = move |_| {
        let date = anchor_date.read().trim().to_string();
        if date.is_empty() { return; }
        let mut anc_sig = anchors;
        let mut date_s  = anchor_date;
        let mut err     = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            match repo.add_pay_period_anchor(&date).await {
                Ok(_) => {
                    if let Ok(d) = repo.list_pay_period_anchors().await { *anc_sig.write() = d; }
                    *date_s.write() = String::new();
                }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    let remove_anchor = move |id: i64| {
        let mut anc_sig = anchors;
        let mut err     = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            match repo.remove_pay_period_anchor(id).await {
                Ok(_)  => { if let Ok(d) = repo.list_pay_period_anchors().await { *anc_sig.write() = d; } }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    // ---- Import handler ----

    let handle_import = move |e: Event<FormData>| {
        let files = e.files();
        let mut lc_sig = labor_codes;
        let mut ht_sig = hour_types;
        let mut err    = error;
        spawn(async move {
            if let Some(engine) = files {
                let names = engine.files();
                if let Some(name) = names.first() {
                    if let Some(text) = engine.read_file_to_string(name).await {
                        match serde_json::from_str::<api::ImportPayload>(&text) {
                            Ok(payload) => {
                                let repo = Repository::new(api::pool());
                                repo.import_lookup_data(&payload.labor_codes, &payload.hour_types).await;
                                if let Ok(d) = repo.list_labor_codes().await { *lc_sig.write() = d; }
                                if let Ok(d) = repo.list_hour_types().await  { *ht_sig.write() = d; }
                            }
                            Err(e) => *err.write() = Some(format!("Invalid JSON: {e}")),
                        }
                    }
                }
            }
        });
    };

    // ---- Export handler ----

    let handle_export = move |_| {
        let mut err = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            match repo.export_lookup_data().await {
                Err(e) => { *err.write() = Some(e.to_string()); return; }
                Ok(payload) => {
                    let json = match serde_json::to_string_pretty(&payload) {
                        Ok(s) => s,
                        Err(e) => { *err.write() = Some(e.to_string()); return; }
                    };
                    if let Some(path) = rfd::AsyncFileDialog::new()
                        .set_title("Export lookup data")
                        .set_file_name("timecard-lookup.json")
                        .add_filter("JSON", &["json"])
                        .save_file()
                        .await
                    {
                        if let Err(e) = std::fs::write(path.path(), json.as_bytes()) {
                            *err.write() = Some(e.to_string());
                        }
                    }
                }
            }
        });
    };

    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold", "Settings" }

            // Error banner
            if let Some(ref msg) = *error.read() {
                div { class: "alert alert-error",
                    span { "{msg}" }
                    button { class: "btn btn-xs btn-ghost ml-auto", onclick: move |_| *error.write() = None, "✕" }
                }
            }

            // --- Pay Period Anchors ---
            div { class: "card bg-base-200 shadow",
                div { class: "card-body",
                    h2 { class: "card-title text-lg", "Pay Period Anchors" }
                    p { class: "text-sm text-base-content/60 mb-3",
                        "Each anchor starts a 2-week pay period cycle that repeats forward indefinitely."
                    }
                    div { class: "flex gap-2 mb-3",
                        input {
                            r#type: "date",
                            class: "input input-bordered input-sm",
                            value: "{anchor_date}",
                            oninput: move |e| *anchor_date.write() = e.value(),
                        }
                        button { class: "btn btn-sm btn-primary", onclick: add_anchor, "Add" }
                    }
                    if !anchors.read().is_empty() {
                        table { class: "table table-sm",
                            thead { tr { th { "Start Date" } th { } } }
                            tbody {
                                for a in anchors.read().iter() {
                                    tr { key: "{a.id}",
                                        td { code { "{a.start_date}" } }
                                        td {
                                            button {
                                                class: "btn btn-xs btn-ghost text-error",
                                                onclick: { let id = a.id; move |_| remove_anchor(id) },
                                                "Remove"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- Labor Codes ---
            div { class: "card bg-base-200 shadow",
                div { class: "card-body",
                    h2 { class: "card-title text-lg", "Labor Codes" }
                    div { class: "flex gap-2 mb-3 flex-wrap",
                        input {
                            class: "input input-bordered input-sm w-32",
                            placeholder: "WBS Number",
                            value: "{lc_wbs}",
                            oninput: move |e| *lc_wbs.write() = e.value(),
                        }
                        input {
                            class: "input input-bordered input-sm flex-1 min-w-32",
                            placeholder: "Name",
                            value: "{lc_name}",
                            oninput: move |e| *lc_name.write() = e.value(),
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            onclick: save_lc,
                            if editing_lc.read().is_some() { "Update" } else { "Add" }
                        }
                        if editing_lc.read().is_some() {
                            button { class: "btn btn-sm btn-ghost", onclick: cancel_lc, "Cancel" }
                        }
                    }
                    if !labor_codes.read().is_empty() {
                        table { class: "table table-sm",
                            thead { tr { th { "WBS" } th { "Name" } th { } } }
                            tbody {
                                for lc in labor_codes.read().iter() {
                                    tr { key: "{lc.id}",
                                        td { code { class: "text-xs", "{lc.wbs_number}" } }
                                        td { "{lc.name}" }
                                        td { class: "flex gap-1",
                                            button {
                                                class: "btn btn-xs btn-ghost",
                                                onclick: {
                                                    let lc = lc.clone();
                                                    move |_| {
                                                        *lc_wbs.write()    = lc.wbs_number.clone();
                                                        *lc_name.write()   = lc.name.clone();
                                                        *editing_lc.write() = Some(lc.clone());
                                                    }
                                                },
                                                "Edit"
                                            }
                                            button {
                                                class: "btn btn-xs btn-ghost text-error",
                                                onclick: { let id = lc.id; move |_| delete_lc(id) },
                                                "✕"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- Hour Types ---
            div { class: "card bg-base-200 shadow",
                div { class: "card-body",
                    h2 { class: "card-title text-lg", "Hour Types" }
                    div { class: "flex gap-2 mb-3 flex-wrap",
                        input {
                            class: "input input-bordered input-sm w-20",
                            placeholder: "REG",
                            value: "{ht_code}",
                            oninput: move |e| *ht_code.write() = e.value(),
                        }
                        input {
                            class: "input input-bordered input-sm flex-1 min-w-32",
                            placeholder: "Name",
                            value: "{ht_name}",
                            oninput: move |e| *ht_name.write() = e.value(),
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            onclick: save_ht,
                            if editing_ht.read().is_some() { "Update" } else { "Add" }
                        }
                        if editing_ht.read().is_some() {
                            button { class: "btn btn-sm btn-ghost", onclick: cancel_ht, "Cancel" }
                        }
                    }
                    if !hour_types.read().is_empty() {
                        table { class: "table table-sm",
                            thead { tr { th { "Code" } th { "Name" } th { } } }
                            tbody {
                                for ht in hour_types.read().iter() {
                                    tr { key: "{ht.id}",
                                        td { code { class: "badge badge-ghost badge-sm", "{ht.code}" } }
                                        td { "{ht.name}" }
                                        td { class: "flex gap-1",
                                            button {
                                                class: "btn btn-xs btn-ghost",
                                                onclick: {
                                                    let ht = ht.clone();
                                                    move |_| {
                                                        *ht_code.write()   = ht.code.clone();
                                                        *ht_name.write()   = ht.name.clone();
                                                        *editing_ht.write() = Some(ht.clone());
                                                    }
                                                },
                                                "Edit"
                                            }
                                            button {
                                                class: "btn btn-xs btn-ghost text-error",
                                                onclick: { let id = ht.id; move |_| delete_ht(id) },
                                                "✕"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- Import / Export ---
            div { class: "card bg-base-200 shadow",
                div { class: "card-body",
                    h2 { class: "card-title text-lg", "Import / Export" }
                    p { class: "text-sm text-base-content/60 mb-4",
                        r#"JSON format: { "labor_codes": [{"wbs_number":"…","name":"…"}], "hour_types": [{"code":"…","name":"…"}] }"#
                    }
                    div { class: "flex gap-3 flex-wrap items-center",
                        // Import
                        label { class: "btn btn-sm btn-outline",
                            "Import JSON"
                            input {
                                r#type: "file",
                                class: "hidden",
                                accept: ".json",
                                onchange: handle_import,
                            }
                        }
                        // Export
                        button { class: "btn btn-sm btn-outline", onclick: handle_export, "Export JSON" }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Verify**

```bash
cargo check -p ui
```

Expected: exits 0.

- [ ] **Step 3: Commit**

```bash
git add packages/ui/src/pages/settings.rs
git commit -m "feat: settings page — lookup CRUD, pay period anchors, import/export"
```

---

### Task 9: Final verification

- [ ] **Step 1: Run all tests**

```bash
cargo test
```

Expected:
```
test result: ok. N passed; 0 failed
```

- [ ] **Step 2: Full workspace check**

```bash
cargo check
```

Expected: exits 0.

- [ ] **Step 3: Build CSS**

```bash
npm run build:css
```

Expected: `packages/ui/assets/app.css` generated.

- [ ] **Step 4: Run the desktop app**

Open terminal A:
```bash
npm run watch:css
```

Open terminal B:
```bash
dx serve --platform desktop
```

Expected: window opens with dark DaisyUI theme.

- [ ] **Step 5: Verify golden path manually**

Work through this checklist in the running app:

- [ ] Settings → Add a labor code (e.g. WBS-001 / "Software Dev")
- [ ] Settings → Add an hour type (e.g. REG / "Regular")
- [ ] Settings → Edit the labor code name → confirm it updates in the table
- [ ] Settings → Add a pay period anchor (today's date)
- [ ] Settings → Export JSON → confirm file saved and contains the labor code + hour type
- [ ] Settings → Delete the labor code → confirm it disappears
- [ ] Settings → Import the exported JSON → labor code reappears
- [ ] Dashboard → Add Entry → select labor code and hour type, set times → Create
- [ ] Dashboard → confirm entry appears in Day tab with correct hours
- [ ] Dashboard → Edit the entry → change end time → Update → confirm hours change
- [ ] Dashboard → Navigate to next day → entry not visible
- [ ] Dashboard → Navigate back → entry visible again
- [ ] Dashboard → Week tab → entry appears → total shown
- [ ] Dashboard → Delete the entry → table empties

- [ ] **Step 6: Final commit**

```bash
git add .
git commit -m "feat: frontend complete — dashboard, settings, import/export"
```

---

## Summary

| Task | Key output |
|---|---|
| 1 | TailwindCSS v4 + DaisyUI v5 build pipeline |
| 2 | Date/timezone utilities + 9 unit tests |
| 3 | Routes, App with context providers, all stubs |
| 4 | DaisyUI navbar layout |
| 5 | Entry table component |
| 6 | Entry form modal (time + duration modes) |
| 7 | Dashboard page (Day + Week tabs) |
| 8 | Settings page (CRUD + import/export) |
| 9 | Full verification + golden path checklist |
