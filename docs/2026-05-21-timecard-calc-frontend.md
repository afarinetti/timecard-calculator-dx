# Timecard Calculator — Frontend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `packages/ui` Dioxus frontend with dashboard views (day/week/pay period/history), entry form modal, settings page, and timezone-aware time display. All styling via TailwindCSS v4 + DaisyUI v5.

**Architecture:** Dioxus 0.7 desktop. Global state via Context API (`use_context_provider`). Async data via `use_resource`. No JavaScript, no TypeScript — all Rust.

**Tech Stack:** Dioxus 0.7 (desktop, router features), TailwindCSS v4, DaisyUI v5, chrono, chrono-tz

**Depends on:** Backend plan must be complete — `packages/api` must compile and `api::db::pool()` must be available.

---

### Task 1: Setup — TailwindCSS v4 + DaisyUI + Dioxus.toml

**Files:**
- Create: `packages/ui/assets/tailwind.css`
- Create: `Dioxus.toml` (workspace root or `packages/desktop/`)

- [ ] **Step 1: Install Tailwind v4 CLI and DaisyUI**

DaisyUI v5 is a CSS plugin for TailwindCSS v4. Install both as npm dev dependencies (used only at build time for CSS generation):

```bash
npm init -y
npm install --save-dev tailwindcss@next @tailwindcss/cli daisyui@latest
```

Or use the standalone Tailwind CLI binary. The key point: this is only for CSS compilation, not for the Rust build.

- [ ] **Step 2: Write `packages/ui/assets/tailwind.css`**

```css
@import "tailwindcss";
@plugin "daisyui" {
  themes: dark --default;
}
```

This sets the DaisyUI dark theme as the default. Adjust the `themes` list to include other DaisyUI themes if desired.

- [ ] **Step 3: Add a CSS build step**

Add a script to generate compiled CSS from the Tailwind input:

```bash
# Run manually during development, or wire into dx serve
npx @tailwindcss/cli -i packages/ui/assets/tailwind.css -o packages/ui/assets/app.css --watch
```

The output `app.css` is the file referenced in the Dioxus app via `asset!`.

- [ ] **Step 4: Add styling convention note to team**

**DaisyUI + Dioxus state rule:** Use DaisyUI semantic/structural classes for visual appearance (`btn`, `card`, `table`, `tabs`, `modal-box`, etc.) but **never use DaisyUI CSS-state classes** (`dropdown-open`, `drawer-toggle`, `collapse-open`, `modal-open` via checkbox, `tab-active` via sibling selector, etc.). These rely on CSS `:checked` pseudo-class and adjacent-sibling selectors that conflict with Dioxus's VDOM rendering.

All open/closed/active state must be driven by `use_signal` with conditional class binding:

```rust
// Correct pattern — signal drives the class
let mut show = use_signal(|| false);
rsx! {
    // Modal open/close
    dialog { class: if show() { "modal modal-open" } else { "modal" },
        div { class: "modal-box", ... }
    }

    // Tab active state
    button {
        class: if active_tab() == "day" { "tab tab-active" } else { "tab" },
        onclick: move |_| *active_tab.write() = "day".to_string(),
        "Day"
    }

    // Dropdown visibility
    div { class: "dropdown",
        button { class: "btn", onclick: move |_| *show.write() = !show(), "Menu" }
        if show() {
            ul { class: "dropdown-content menu ...", ... }
        }
    }
}
```

This means: avoid `<input type="checkbox" class="modal-toggle">`, `<input class="drawer-toggle">`, `collapse` with checkbox, and similar patterns from the DaisyUI docs. Use the equivalent `open`/signal approach for every interactive DaisyUI component.

- [ ] **Step 5: Create `Dioxus.toml`** (at workspace root)

```toml
[application]
name = "timecard-calc"
default_platform = "desktop"
out_dir = "dist"
asset_dir = "packages/ui/assets"

[desktop]
```

- [ ] **Step 6: Commit**

```bash
git add packages/ui/assets/ Dioxus.toml package.json
git commit -m "feat: add TailwindCSS v4 + DaisyUI v5 setup"
```

---

### Task 2: Routes + App shell

**Files:**
- Create: `packages/ui/src/routes.rs`
- Create: `packages/ui/src/app.rs`
- Modify: `packages/ui/src/lib.rs`

- [ ] **Step 1: Write `packages/ui/src/routes.rs`**

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

- [ ] **Step 2: Write `packages/ui/src/app.rs`**

```rust
use dioxus::prelude::*;
use api::{LaborCode, HourType, PayPeriodAnchor, Repository};
use chrono::Local;
use crate::routes::Route;

pub struct AppState {
    pub labor_codes: Signal<Vec<LaborCode>>,
    pub hour_types: Signal<Vec<HourType>>,
    pub pay_period_anchors: Signal<Vec<PayPeriodAnchor>>,
    pub current_date: Signal<String>,
    pub current_week_start: Signal<String>,
}

#[component]
pub fn App() -> Element {
    // Provide global state via context
    let labor_codes = use_context_provider(|| Signal::new(Vec::<LaborCode>::new()));
    let hour_types = use_context_provider(|| Signal::new(Vec::<HourType>::new()));
    let pay_period_anchors = use_context_provider(|| Signal::new(Vec::<PayPeriodAnchor>::new()));

    let today = Local::now().format("%Y-%m-%d").to_string();
    // Week start = most recent Monday
    let week_start = {
        use chrono::{Datelike, Duration, NaiveDate, Weekday};
        let d = NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();
        let days_from_mon = d.weekday().num_days_from_monday();
        (d - Duration::days(days_from_mon as i64)).format("%Y-%m-%d").to_string()
    };
    use_context_provider(|| Signal::new(today));
    use_context_provider(|| Signal::new(week_start));

    // Load lookup data on startup
    let _load = use_resource(move || async move {
        let pool = api::pool();
        let repo = Repository::new(pool);
        let mut lc = labor_codes;
        let mut ht = hour_types;
        let mut ppa = pay_period_anchors;
        if let Ok(data) = repo.list_labor_codes().await { *lc.write() = data; }
        if let Ok(data) = repo.list_hour_types().await { *ht.write() = data; }
        if let Ok(data) = repo.list_pay_period_anchors().await { *ppa.write() = data; }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/app.css") }
        Router::<Route> {}
    }
}
```

- [ ] **Step 3: Write `packages/ui/src/lib.rs`**

```rust
pub mod app;
pub mod routes;
pub mod components;
pub mod pages;
pub mod utils;

pub use app::App;
```

- [ ] **Step 4: Commit**

```bash
git add packages/ui/src/
git commit -m "feat: add Route enum and App component with context providers"
```

---

### Task 3: Layout component + navigation

**Files:**
- Create: `packages/ui/src/components/layout.rs`
- Create: `packages/ui/src/components/mod.rs`

- [ ] **Step 1: Write `packages/ui/src/components/layout.rs`**

```rust
use dioxus::prelude::*;
use crate::routes::Route;

#[component]
pub fn Layout() -> Element {
    rsx! {
        div { class: "min-h-screen bg-base-100",
            // Navbar
            div { class: "navbar bg-base-200 shadow-sm",
                div { class: "flex-1",
                    span { class: "text-xl font-bold px-4", "Timecard Calc" }
                }
                div { class: "flex-none",
                    ul { class: "menu menu-horizontal px-1",
                        li {
                            Link { to: Route::Dashboard {}, "Dashboard" }
                        }
                        li {
                            Link { to: Route::Settings {}, "Settings" }
                        }
                    }
                }
            }
            // Page content
            main { class: "container mx-auto p-6",
                Outlet::<Route> {}
            }
        }
    }
}
```

- [ ] **Step 2: Write `packages/ui/src/components/mod.rs`**

```rust
pub mod layout;
pub mod entry_form;
pub mod entry_table;
```

- [ ] **Step 3: Commit**

```bash
git add packages/ui/src/components/
git commit -m "feat: add layout component with DaisyUI navbar"
```

---

### Task 4: Timezone utilities

**Files:**
- Create: `packages/ui/src/utils.rs`

- [ ] **Step 1: Write `packages/ui/src/utils.rs`**

```rust
use chrono::{NaiveDate, NaiveDateTime, Duration};
use chrono_tz::America::Chicago;
use chrono::TimeZone;

pub const CENTRAL_TZ: chrono_tz::Tz = Chicago;

/// Convert UTC ISO 8601 timestamp to Central time display string (HH:MM).
pub fn utc_to_central_hhmm(utc_iso: &str) -> String {
    let utc = chrono::DateTime::parse_from_rfc3339(utc_iso)
        .or_else(|_| chrono::DateTime::parse_from_str(utc_iso, "%Y-%m-%dT%H:%M:%SZ"))
        .ok();
    match utc {
        Some(dt) => dt.with_timezone(&CENTRAL_TZ).format("%H:%M").to_string(),
        None => "??:??".to_string(),
    }
}

/// Navigate a YYYY-MM-DD date string by delta days.
pub fn navigate_date(date: &str, delta: i64) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(delta)).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|_| date.to_string())
}

/// Navigate a week start date by delta weeks.
pub fn navigate_week(week_start: &str, delta: i64) -> String {
    navigate_date(week_start, delta * 7)
}

/// Get today's date as YYYY-MM-DD in local time.
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Get the Monday of the week containing `date`.
pub fn week_start_for(date: &str) -> String {
    use chrono::Datelike;
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| {
            let days = d.weekday().num_days_from_monday();
            (d - Duration::days(days as i64)).format("%Y-%m-%d").to_string()
        })
        .unwrap_or_else(|_| date.to_string())
}

/// Compute live elapsed decimal hours (rounded to nearest 15m) from a UTC start time to now.
pub fn live_elapsed_hours(utc_start: &str) -> f64 {
    use chrono::Utc;
    let start = chrono::DateTime::parse_from_rfc3339(utc_start)
        .ok()
        .map(|dt| dt.with_timezone(&Utc));
    match start {
        Some(s) => {
            let mins = (Utc::now() - s).num_minutes().max(0) as f64;
            (mins / 15.0).round() * 15.0 / 60.0
        }
        None => 0.0,
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/ui/src/utils.rs
git commit -m "feat: add timezone and date navigation utilities"
```

---

### Task 5: Entry table component

**Files:**
- Create: `packages/ui/src/components/entry_table.rs`

- [ ] **Step 1: Write `packages/ui/src/components/entry_table.rs`**

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
    rsx! {
        div { class: "overflow-x-auto",
            table { class: "table table-zebra w-full",
                thead {
                    tr {
                        th { "WBS" }
                        th { "Hour Type" }
                        th { "Start" }
                        th { "End" }
                        th { "Hours" }
                        th { "Telework" }
                        th { "Actions" }
                    }
                }
                tbody {
                    for entry in entries.iter() {
                        tr { key: "{entry.id}",
                            td { "{entry.wbs_number}" }
                            td { "{entry.hour_type_code}" }
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
                                    "{h:.2}"
                                } else {
                                    "—"
                                }
                            }
                            td {
                                if entry.telework {
                                    span { class: "badge badge-info badge-sm", "Yes" }
                                }
                            }
                            td { class: "flex gap-2",
                                button {
                                    class: "btn btn-xs btn-ghost",
                                    onclick: {
                                        let entry = entry.clone();
                                        move |_| on_edit.call(entry.clone())
                                    },
                                    "Edit"
                                }
                                button {
                                    class: "btn btn-xs btn-error btn-ghost",
                                    onclick: {
                                        let id = entry.id;
                                        move |_| on_delete.call(id)
                                    },
                                    "Delete"
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

- [ ] **Step 2: Commit**

```bash
git add packages/ui/src/components/entry_table.rs
git commit -m "feat: add entry table component"
```

---

### Task 6: Entry form modal

**Files:**
- Create: `packages/ui/src/components/entry_form.rs`

- [ ] **Step 1: Write `packages/ui/src/components/entry_form.rs`**

```rust
use dioxus::prelude::*;
use api::{LaborCode, HourType, TimecardEntryView, CreateTimecardEntry, UpdateTimecardEntry, Repository};
use crate::utils::utc_to_central_hhmm;

#[derive(Clone, PartialEq)]
enum EntryMode {
    TimeInputs,
    Duration,
}

#[component]
pub fn EntryFormModal(
    show: bool,
    editing: Option<TimecardEntryView>,
    date: String,
    on_close: EventHandler,
    on_saved: EventHandler,
) -> Element {
    let labor_codes = use_context::<Signal<Vec<LaborCode>>>();
    let hour_types = use_context::<Signal<Vec<HourType>>>();

    let mut mode = use_signal(|| EntryMode::TimeInputs);
    let mut labor_code_id = use_signal(|| String::new());
    let mut hour_type_id = use_signal(|| String::new());
    let mut telework = use_signal(|| false);
    let mut start_time = use_signal(|| "08:00".to_string());
    let mut end_time = use_signal(|| "17:00".to_string());
    let mut duration = use_signal(|| 8.0f64);
    let mut error = use_signal(|| Option::<String>::None);

    // Populate fields when editing an existing entry
    use_effect(move || {
        if let Some(ref e) = editing {
            *labor_code_id.write() = e.labor_code_id.to_string();
            *hour_type_id.write() = e.hour_type_id.to_string();
            *telework.write() = e.telework;
            *start_time.write() = utc_to_central_hhmm(&e.start_time);
            *end_time.write() = e.end_time.as_deref().map(utc_to_central_hhmm).unwrap_or_default();
            *duration.write() = e.decimal_hours.unwrap_or(8.0);
        } else {
            *labor_code_id.write() = String::new();
            *hour_type_id.write() = String::new();
            *telework.write() = false;
            *start_time.write() = "08:00".to_string();
            *end_time.write() = "17:00".to_string();
            *duration.write() = 8.0;
        }
        *error.write() = None;
    });

    let compute_end_from_duration = move |start: &str, dur: f64| -> Option<String> {
        let parts: Vec<&str> = start.split(':').collect();
        if parts.len() != 2 { return None; }
        let h: u32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        let total_min = h * 60 + m + (dur * 60.0).round() as u32;
        if total_min >= 1440 { return None; } // past midnight
        Some(format!("{:02}:{:02}", total_min / 60, total_min % 60))
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
            EntryMode::Duration => compute_end_from_duration(&start, *duration.read()),
        };

        let editing_clone = editing.clone();
        let date_clone = date.clone();
        let tw = *telework.read();
        let mut err = error;
        let on_saved = on_saved.clone();
        let on_close = on_close.clone();

        spawn(async move {
            let pool = api::pool();
            let repo = Repository::new(pool);
            let result = if let Some(ref e) = editing_clone {
                repo.update_timecard_entry(&UpdateTimecardEntry {
                    id: e.id, labor_code_id: lc_id, hour_type_id: ht_id,
                    telework: tw, date: date_clone, start_time: start, end_time: end,
                }).await.map(|_| ())
            } else {
                repo.create_timecard_entry(&CreateTimecardEntry {
                    labor_code_id: lc_id, hour_type_id: ht_id,
                    telework: tw, date: date_clone, start_time: start, end_time: end,
                }).await.map(|_| ())
            };
            match result {
                Ok(_) => { on_saved.call(()); on_close.call(()); }
                Err(e) => { *err.write() = Some(e.to_string()); }
            }
        });
    };

    rsx! {
        dialog {
            class: if show { "modal modal-open" } else { "modal" },
            div { class: "modal-box w-11/12 max-w-lg",
                h3 { class: "font-bold text-lg mb-4",
                    if editing.is_some() { "Edit Entry" } else { "New Entry" }
                }

                // Mode toggle
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
                        option { value: "", disabled: true, selected: labor_code_id.read().is_empty(), "Select labor code" }
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
                        option { value: "", disabled: true, selected: hour_type_id.read().is_empty(), "Select hour type" }
                        for ht in hour_types.read().iter() {
                            option { value: "{ht.id}", "{ht.code} — {ht.name}" }
                        }
                    }
                }

                // Telework toggle
                div { class: "form-control mb-3",
                    label { class: "label cursor-pointer",
                        span { class: "label-text", "Telework" }
                        input {
                            r#type: "checkbox",
                            class: "toggle toggle-primary",
                            checked: "{telework}",
                            onchange: move |e| *telework.write() = e.checked(),
                        }
                    }
                }

                // Time fields
                if *mode.read() == EntryMode::TimeInputs {
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
                        div { class: "form-control",
                            label { class: "label", span { class: "label-text", "End Time (optional)" } }
                            input {
                                r#type: "time",
                                class: "input input-bordered",
                                value: "{end_time}",
                                oninput: move |e| *end_time.write() = e.value(),
                            }
                        }
                    }
                } else {
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
                        div { class: "form-control",
                            label { class: "label", span { class: "label-text", "Duration (hours)" } }
                            input {
                                r#type: "number",
                                class: "input input-bordered",
                                min: "0.25",
                                max: "24",
                                step: "0.25",
                                value: "{duration}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<f64>() {
                                        *duration.write() = v;
                                    }
                                },
                            }
                        }
                    }
                }

                // Error message
                if let Some(ref msg) = *error.read() {
                    div { class: "alert alert-error mb-3",
                        span { "{msg}" }
                    }
                }

                div { class: "modal-action",
                    button { class: "btn btn-ghost", onclick: move |_| on_close.call(()), "Cancel" }
                    button { class: "btn btn-primary", onclick: handle_submit,
                        if editing.is_some() { "Update" } else { "Create" }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/ui/src/components/entry_form.rs
git commit -m "feat: add entry form modal with time inputs and duration mode"
```

---

### Task 7: Dashboard page

**Files:**
- Create: `packages/ui/src/pages/dashboard.rs`
- Create: `packages/ui/src/pages/mod.rs`

- [ ] **Step 1: Write `packages/ui/src/pages/dashboard.rs`**

```rust
use dioxus::prelude::*;
use api::{TimecardEntryView, Repository};
use crate::components::{entry_table::EntryTable, entry_form::EntryFormModal};
use crate::utils::{navigate_date, navigate_week, today};

#[derive(Clone, PartialEq)]
enum DashTab {
    Day,
    Week,
    PayPeriod,
    History,
}

#[component]
pub fn Dashboard() -> Element {
    let mut current_date = use_context::<Signal<String>>();
    let mut current_week = use_context::<Signal<String>>();

    let mut tab = use_signal(|| DashTab::Day);
    let mut show_form = use_signal(|| false);
    let mut editing_entry = use_signal(|| Option::<TimecardEntryView>::None);
    let mut error = use_signal(|| Option::<String>::None);

    // Reload trigger: bump this to force use_resource to re-run
    let mut reload = use_signal(|| 0u32);

    // Day data
    let day_data = use_resource(move || {
        let date = current_date.read().clone();
        let _ = reload(); // subscribe to reload trigger
        async move {
            let pool = api::pool();
            Repository::new(pool).get_day_summary(&date).await
        }
    });

    // Week data
    let week_data = use_resource(move || {
        let week = current_week.read().clone();
        let _ = reload();
        async move {
            let pool = api::pool();
            Repository::new(pool).get_week_summary(&week).await
        }
    });

    let refresh = move || *reload.write() += 1;

    let handle_delete = move |id: i64| {
        spawn(async move {
            let pool = api::pool();
            match Repository::new(pool).delete_timecard_entry(id).await {
                Ok(_) => *reload.write() += 1,
                Err(e) => *error.write() = Some(e.to_string()),
            }
        });
    };

    rsx! {
        div { class: "space-y-4",

            // Error banner
            if let Some(ref msg) = *error.read() {
                div { class: "alert alert-error",
                    span { "{msg}" }
                    button { class: "btn btn-xs btn-ghost ml-auto", onclick: move |_| *error.write() = None, "✕" }
                }
            }

            // Header row with navigation + Add button
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-2",
                    if *tab.read() == DashTab::Day {
                        button {
                            class: "btn btn-sm btn-ghost",
                            onclick: move |_| *current_date.write() = navigate_date(&current_date.read(), -1),
                            "‹"
                        }
                        span { class: "font-semibold", "{current_date}" }
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
                        span { class: "font-semibold", "Week of {current_week}" }
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

            // Tabs
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

            // Tab content
            match *tab.read() {
                DashTab::Day => {
                    match day_data.read().as_ref() {
                        None => rsx! { div { class: "loading loading-spinner" } },
                        Some(Err(e)) => rsx! { div { class: "alert alert-error", "{e}" } },
                        Some(Ok(summary)) => rsx! {
                            EntryTable {
                                entries: summary.entries.clone(),
                                on_edit: move |entry| {
                                    *editing_entry.write() = Some(entry);
                                    *show_form.write() = true;
                                },
                                on_delete: handle_delete,
                            }
                            div { class: "text-right font-semibold mt-2",
                                "Total: {summary.total_hours:.2} hrs"
                            }
                        },
                    }
                }
                DashTab::Week => {
                    match week_data.read().as_ref() {
                        None => rsx! { div { class: "loading loading-spinner" } },
                        Some(Err(e)) => rsx! { div { class: "alert alert-error", "{e}" } },
                        Some(Ok(summary)) => rsx! {
                            EntryTable {
                                entries: summary.entries.clone(),
                                on_edit: move |entry| {
                                    *editing_entry.write() = Some(entry);
                                    *show_form.write() = true;
                                },
                                on_delete: handle_delete,
                            }
                            div { class: "flex justify-between mt-2",
                                div { class: "flex gap-4",
                                    for day in summary.by_day.iter() {
                                        span { class: "text-sm", "{day.date}: {day.total_hours:.2}h" }
                                    }
                                }
                                div { class: "font-semibold", "Total: {summary.total_hours:.2} hrs" }
                            }
                        },
                    }
                }
                DashTab::PayPeriod => rsx! {
                    div { class: "text-base-content/50 py-8 text-center",
                        "Pay period view — configure anchors in Settings first."
                    }
                },
                DashTab::History => rsx! {
                    div { class: "text-base-content/50 py-8 text-center",
                        "Select a pay period to view historical data."
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

- [ ] **Step 2: Write `packages/ui/src/pages/mod.rs`**

```rust
pub mod dashboard;
pub mod settings;
```

- [ ] **Step 3: Commit**

```bash
git add packages/ui/src/pages/
git commit -m "feat: add dashboard page with day/week tabs and entry form"
```

---

### Task 8: Settings page

**Files:**
- Create: `packages/ui/src/pages/settings.rs`

- [ ] **Step 1: Write `packages/ui/src/pages/settings.rs`**

```rust
use dioxus::prelude::*;
use api::{LaborCode, HourType, PayPeriodAnchor, Repository,
          CreateLaborCode, UpdateLaborCode, CreateHourType, UpdateHourType};
use rfd;

#[component]
pub fn Settings() -> Element {
    let mut labor_codes = use_context::<Signal<Vec<LaborCode>>>();
    let mut hour_types = use_context::<Signal<Vec<HourType>>>();
    let mut anchors = use_context::<Signal<Vec<PayPeriodAnchor>>>();
    let mut error = use_signal(|| Option::<String>::None);

    // --- Labor Codes ---
    let mut lc_wbs = use_signal(|| String::new());
    let mut lc_name = use_signal(|| String::new());
    let mut editing_lc = use_signal(|| Option::<LaborCode>::None);

    let save_labor_code = move |_| {
        let wbs = lc_wbs.read().trim().to_string();
        let name = lc_name.read().trim().to_string();
        if wbs.is_empty() || name.is_empty() { return; }
        let editing = editing_lc.read().clone();
        let mut lc_sig = labor_codes;
        let mut edit_sig = editing_lc;
        let mut wbs_sig = lc_wbs;
        let mut name_sig = lc_name;
        let mut err = error;
        spawn(async move {
            let pool = api::pool();
            let repo = Repository::new(pool);
            let result = if let Some(ref e) = editing {
                repo.update_labor_code(&UpdateLaborCode { id: e.id, wbs_number: wbs, name }).await
            } else {
                repo.create_labor_code(&CreateLaborCode { wbs_number: wbs, name }).await
            };
            match result {
                Ok(_) => {
                    if let Ok(data) = repo.list_labor_codes().await { *lc_sig.write() = data; }
                    *edit_sig.write() = None;
                    *wbs_sig.write() = String::new();
                    *name_sig.write() = String::new();
                }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    let delete_labor_code = move |id: i64| {
        let mut lc_sig = labor_codes;
        let mut err = error;
        spawn(async move {
            let pool = api::pool();
            let repo = Repository::new(pool);
            match repo.delete_labor_code(id).await {
                Ok(_) => { if let Ok(data) = repo.list_labor_codes().await { *lc_sig.write() = data; } }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    // --- Hour Types ---
    let mut ht_code = use_signal(|| String::new());
    let mut ht_name = use_signal(|| String::new());
    let mut editing_ht = use_signal(|| Option::<HourType>::None);

    let save_hour_type = move |_| {
        let code = ht_code.read().trim().to_string();
        let name = ht_name.read().trim().to_string();
        if code.is_empty() || name.is_empty() { return; }
        let editing = editing_ht.read().clone();
        let mut ht_sig = hour_types;
        let mut edit_sig = editing_ht;
        let mut code_sig = ht_code;
        let mut name_sig = ht_name;
        let mut err = error;
        spawn(async move {
            let pool = api::pool();
            let repo = Repository::new(pool);
            let result = if let Some(ref e) = editing {
                repo.update_hour_type(&UpdateHourType { id: e.id, code, name }).await
            } else {
                repo.create_hour_type(&CreateHourType { code, name }).await
            };
            match result {
                Ok(_) => {
                    if let Ok(data) = repo.list_hour_types().await { *ht_sig.write() = data; }
                    *edit_sig.write() = None;
                    *code_sig.write() = String::new();
                    *name_sig.write() = String::new();
                }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    let delete_hour_type = move |id: i64| {
        let mut ht_sig = hour_types;
        let mut err = error;
        spawn(async move {
            let pool = api::pool();
            let repo = Repository::new(pool);
            match repo.delete_hour_type(id).await {
                Ok(_) => { if let Ok(data) = repo.list_hour_types().await { *ht_sig.write() = data; } }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    // --- Pay Period Anchors ---
    let mut anchor_date = use_signal(|| String::new());

    let add_anchor = move |_| {
        let date = anchor_date.read().trim().to_string();
        if date.is_empty() { return; }
        let mut anc_sig = anchors;
        let mut date_sig = anchor_date;
        let mut err = error;
        spawn(async move {
            let pool = api::pool();
            let repo = Repository::new(pool);
            match repo.add_pay_period_anchor(&date).await {
                Ok(_) => {
                    if let Ok(data) = repo.list_pay_period_anchors().await { *anc_sig.write() = data; }
                    *date_sig.write() = String::new();
                }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    let remove_anchor = move |id: i64| {
        let mut anc_sig = anchors;
        let mut err = error;
        spawn(async move {
            let pool = api::pool();
            let repo = Repository::new(pool);
            match repo.remove_pay_period_anchor(id).await {
                Ok(_) => { if let Ok(data) = repo.list_pay_period_anchors().await { *anc_sig.write() = data; } }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    rsx! {
        div { class: "space-y-8",
            h1 { class: "text-2xl font-bold", "Settings" }

            // Error banner
            if let Some(ref msg) = *error.read() {
                div { class: "alert alert-error",
                    span { "{msg}" }
                    button { class: "btn btn-xs btn-ghost ml-auto", onclick: move |_| *error.write() = None, "✕" }
                }
            }

            // --- Pay Period Anchors ---
            div { class: "card bg-base-200",
                div { class: "card-body",
                    h2 { class: "card-title", "Pay Period Anchors" }
                    p { class: "text-sm text-base-content/60 mb-3", "Each anchor starts a 2-week pay period cycle." }
                    div { class: "flex gap-2 mb-4",
                        input {
                            r#type: "date",
                            class: "input input-bordered input-sm",
                            value: "{anchor_date}",
                            oninput: move |e| *anchor_date.write() = e.value(),
                        }
                        button { class: "btn btn-sm btn-primary", onclick: add_anchor, "Add" }
                    }
                    table { class: "table table-sm",
                        thead { tr { th { "Start Date" } th { } } }
                        tbody {
                            for anchor in anchors.read().iter() {
                                tr { key: "{anchor.id}",
                                    td { "{anchor.start_date}" }
                                    td {
                                        button {
                                            class: "btn btn-xs btn-error btn-ghost",
                                            onclick: { let id = anchor.id; move |_| remove_anchor(id) },
                                            "Remove"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- Labor Codes ---
            div { class: "card bg-base-200",
                div { class: "card-body",
                    h2 { class: "card-title", "Labor Codes" }
                    div { class: "flex gap-2 mb-4",
                        input {
                            class: "input input-bordered input-sm",
                            placeholder: "WBS Number",
                            value: "{lc_wbs}",
                            oninput: move |e| *lc_wbs.write() = e.value(),
                        }
                        input {
                            class: "input input-bordered input-sm flex-1",
                            placeholder: "Name",
                            value: "{lc_name}",
                            oninput: move |e| *lc_name.write() = e.value(),
                        }
                        button { class: "btn btn-sm btn-primary", onclick: save_labor_code,
                            if editing_lc.read().is_some() { "Update" } else { "Add" }
                        }
                        if editing_lc.read().is_some() {
                            button {
                                class: "btn btn-sm btn-ghost",
                                onclick: move |_| {
                                    *editing_lc.write() = None;
                                    *lc_wbs.write() = String::new();
                                    *lc_name.write() = String::new();
                                },
                                "Cancel"
                            }
                        }
                    }
                    table { class: "table table-sm",
                        thead { tr { th { "WBS" } th { "Name" } th { } } }
                        tbody {
                            for lc in labor_codes.read().iter() {
                                tr { key: "{lc.id}",
                                    td { "{lc.wbs_number}" }
                                    td { "{lc.name}" }
                                    td { class: "flex gap-1",
                                        button {
                                            class: "btn btn-xs btn-ghost",
                                            onclick: {
                                                let lc = lc.clone();
                                                move |_| {
                                                    *lc_wbs.write() = lc.wbs_number.clone();
                                                    *lc_name.write() = lc.name.clone();
                                                    *editing_lc.write() = Some(lc.clone());
                                                }
                                            },
                                            "Edit"
                                        }
                                        button {
                                            class: "btn btn-xs btn-error btn-ghost",
                                            onclick: { let id = lc.id; move |_| delete_labor_code(id) },
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- Hour Types ---
            div { class: "card bg-base-200",
                div { class: "card-body",
                    h2 { class: "card-title", "Hour Types" }
                    div { class: "flex gap-2 mb-4",
                        input {
                            class: "input input-bordered input-sm w-24",
                            placeholder: "Code (REG)",
                            value: "{ht_code}",
                            oninput: move |e| *ht_code.write() = e.value(),
                        }
                        input {
                            class: "input input-bordered input-sm flex-1",
                            placeholder: "Name",
                            value: "{ht_name}",
                            oninput: move |e| *ht_name.write() = e.value(),
                        }
                        button { class: "btn btn-sm btn-primary", onclick: save_hour_type,
                            if editing_ht.read().is_some() { "Update" } else { "Add" }
                        }
                        if editing_ht.read().is_some() {
                            button {
                                class: "btn btn-sm btn-ghost",
                                onclick: move |_| {
                                    *editing_ht.write() = None;
                                    *ht_code.write() = String::new();
                                    *ht_name.write() = String::new();
                                },
                                "Cancel"
                            }
                        }
                    }
                    table { class: "table table-sm",
                        thead { tr { th { "Code" } th { "Name" } th { } } }
                        tbody {
                            for ht in hour_types.read().iter() {
                                tr { key: "{ht.id}",
                                    td { code { "{ht.code}" } }
                                    td { "{ht.name}" }
                                    td { class: "flex gap-1",
                                        button {
                                            class: "btn btn-xs btn-ghost",
                                            onclick: {
                                                let ht = ht.clone();
                                                move |_| {
                                                    *ht_code.write() = ht.code.clone();
                                                    *ht_name.write() = ht.name.clone();
                                                    *editing_ht.write() = Some(ht.clone());
                                                }
                                            },
                                            "Edit"
                                        }
                                        button {
                                            class: "btn btn-xs btn-error btn-ghost",
                                            onclick: { let id = ht.id; move |_| delete_hour_type(id) },
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- JSON Import ---
            div { class: "card bg-base-200",
                div { class: "card-body",
                    h2 { class: "card-title", "Import JSON" }
                    p { class: "text-sm text-base-content/60 mb-3",
                        r#"Expects: { "labor_codes": [{"wbs_number":"...","name":"..."}], "hour_types": [{"code":"...","name":"..."}] }"#
                    }
                    input {
                        r#type: "file",
                        class: "file-input file-input-bordered file-input-sm",
                        accept: ".json",
                        onchange: move |e| {
                            // File reading in Dioxus desktop uses the file engine
                            let files = e.files();
                            let mut lc_sig = labor_codes;
                            let mut ht_sig = hour_types;
                            let mut err = error;
                            spawn(async move {
                                if let Some(engine) = files {
                                    let file_names = engine.files();
                                    if let Some(name) = file_names.first() {
                                        if let Some(content) = engine.read_file_to_string(name).await {
                                            match serde_json::from_str::<api::ImportPayload>(&content) {
                                                Ok(payload) => {
                                                    let pool = api::pool();
                                                    let repo = Repository::new(pool);
                                                    repo.import_lookup_data(&payload.labor_codes, &payload.hour_types).await;
                                                    if let Ok(data) = repo.list_labor_codes().await { *lc_sig.write() = data; }
                                                    if let Ok(data) = repo.list_hour_types().await { *ht_sig.write() = data; }
                                                }
                                                Err(e) => *err.write() = Some(format!("Invalid JSON: {e}")),
                                            }
                                        }
                                    }
                                }
                            });
                        },
                    }
                }
            }

            // --- JSON Export ---
            div { class: "card bg-base-200",
                div { class: "card-body",
                    h2 { class: "card-title", "Export JSON" }
                    p { class: "text-sm text-base-content/60 mb-3",
                        "Export all labor codes and hour types to a JSON file (same format as import)."
                    }
                    button {
                        class: "btn btn-sm btn-outline",
                        onclick: move |_| {
                            let mut err = error;
                            spawn(async move {
                                let pool = api::pool();
                                let repo = Repository::new(pool);
                                match repo.export_lookup_data().await {
                                    Err(e) => { *err.write() = Some(e.to_string()); return; }
                                    Ok(payload) => {
                                        let json = match serde_json::to_string_pretty(&payload) {
                                            Ok(s) => s,
                                            Err(e) => { *err.write() = Some(e.to_string()); return; }
                                        };
                                        // Open native save-file dialog
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
                        },
                        "Export"
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/ui/src/pages/settings.rs
git commit -m "feat: add settings page with lookup CRUD, pay period anchors, JSON import"
```

---

### Task 9: Final verification

- [ ] **Step 1: Build CSS**

```bash
npx @tailwindcss/cli -i packages/ui/assets/tailwind.css -o packages/ui/assets/app.css
```

Expected: `app.css` generated with Tailwind base + DaisyUI dark theme.

- [ ] **Step 2: Run `cargo check`**

```bash
cargo check
```

- [ ] **Step 3: Build and run desktop app**

```bash
dx serve --platform desktop
```

Expected: native desktop window opens, DB initialized, dashboard loads, all tabs navigate correctly, entry form opens as modal, settings CRUD works.

- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "feat: complete frontend — Dioxus RSX, DaisyUI, dashboard, settings"
```

---

## Plan Summary

| Task | File(s) | Status |
|------|---------|--------|
| 1. Setup TailwindCSS + DaisyUI | `assets/tailwind.css`, `Dioxus.toml` | - [ ] |
| 2. Routes + App component | `src/routes.rs`, `src/app.rs`, `src/lib.rs` | - [ ] |
| 3. Layout component | `src/components/layout.rs` | - [ ] |
| 4. Timezone utilities | `src/utils.rs` | - [ ] |
| 5. Entry table component | `src/components/entry_table.rs` | - [ ] |
| 6. Entry form modal | `src/components/entry_form.rs` | - [ ] |
| 7. Dashboard page | `src/pages/dashboard.rs` | - [ ] |
| 8. Settings page | `src/pages/settings.rs` | - [ ] |
| 9. Final verification | dx serve, cargo check | - [ ] |

**Frontend depends on the backend plan being complete first.**
