use dioxus::prelude::*;
use api::{
    CreateTimecardEntry, LaborCode, HourType, Repository,
    TimecardEntryView, UpdateTimecardEntry,
};
use crate::utils::{utc_to_central_hhmm, start_now_hhmm, end_now_hhmm};

#[derive(Clone, PartialEq)]
enum EntryMode { TimeInputs, Duration }

#[component]
pub fn EntryFormModal(
    show: ReadSignal<bool>,
    editing: ReadSignal<Option<TimecardEntryView>>,
    date: String,
    on_close: EventHandler,
    on_saved: EventHandler,
) -> Element {
    let labor_codes = use_context::<Signal<Vec<LaborCode>>>();
    let hour_types  = use_context::<Signal<Vec<HourType>>>();

    let mut mode          = use_signal(|| EntryMode::TimeInputs);
    let mut labor_code_id = use_signal(String::new);
    let mut hour_type_id  = use_signal(String::new);
    let mut telework      = use_signal(|| false);
    let mut start_time    = use_signal(String::new);
    let mut end_time      = use_signal(String::new);
    let mut duration      = use_signal(|| 8.0f64);
    let mut error         = use_signal(|| Option::<String>::None);

    // Populate fields when the drawer opens or the editing target changes.
    // Both `show` and `editing` are ReadOnlySignal so the effect re-runs whenever
    // either changes — e.g. switching from "add" to "edit" or opening the form
    // for a different entry.
    use_effect(move || {
        let _ = show();  // track open/close
        match editing() {
            Some(e) => {
                *labor_code_id.write() = e.labor_code_id.to_string();
                *hour_type_id.write()  = e.hour_type_id.to_string();
                *telework.write()      = e.telework;
                *start_time.write()    = utc_to_central_hhmm(e.start_time);
                *end_time.write()      = e.end_time.map(utc_to_central_hhmm).unwrap_or_default();
                *duration.write()      = e.decimal_hours.unwrap_or(8.0);
            }
            None => {
                *labor_code_id.write() = String::new();
                // Default hour type to REG if available
                let reg_id = hour_types.read().iter()
                    .find(|ht| ht.code.eq_ignore_ascii_case("REG"))
                    .map(|ht| ht.id.to_string())
                    .unwrap_or_default();
                *hour_type_id.write()  = reg_id;
                *telework.write()      = false;
                *start_time.write()    = String::new();
                *end_time.write()      = String::new();
                *duration.write()      = 8.0;
            }
        }
        *mode.write()  = EntryMode::TimeInputs;
        *error.write() = None;
    });

    // All hooks above; early return below is safe
    if !show() {
        return rsx! { div {} };
    }

    let end_from_duration = move |start: &str, dur: f64| -> Option<String> {
        let (h, m): (u32, u32) = {
            let p: Vec<&str> = start.split(':').collect();
            (p.first()?.parse().ok()?, p.get(1)?.parse().ok()?)
        };
        let total = h * 60 + m + (dur * 60.0).round() as u32;
        if total >= 1440 { return None; }
        Some(format!("{:02}:{:02}", total / 60, total % 60))
    };

    let date_for_submit = date.clone();
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

        let editing_val = editing();
        let date_val    = date_for_submit.clone();
        let tw          = *telework.read();
        let mut err     = error;
        let on_saved    = on_saved;
        let on_close    = on_close;

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

    let is_editing = editing().is_some();

    rsx! {
        div { class: "fixed inset-0 z-50 flex",
            // Dim backdrop — click closes drawer
            div {
                class: "flex-1 bg-black/35",
                onclick: move |_| on_close.call(()),
            }

            // Drawer panel
            div { class: "w-80 bg-[#161b22] border-l border-[#30363d] flex flex-col",
                // ── Header ──────────────────────────────────────────────
                div { class: "flex-shrink-0 flex items-center justify-between px-5 py-4 border-b border-[#30363d]",
                    div { class: "flex flex-col gap-0.5",
                        span { class: "text-sm font-semibold text-[#e6edf3]",
                            if is_editing { "Edit Entry" } else { "New Entry" }
                        }
                        span { class: "text-xs text-[#8b949e]", "{date}" }
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-ghost btn-xs btn-square",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                // ── Body ────────────────────────────────────────────────
                div { class: "flex-1 overflow-y-auto px-5 py-[18px] flex flex-col gap-4",

                    // Mode toggle (segmented control)
                    div { class: "bg-[#0d1117] border border-[#30363d] rounded-[6px] overflow-hidden flex",
                        button {
                            r#type: "button",
                            class: if *mode.read() == EntryMode::TimeInputs {
                                "flex-1 btn btn-xs btn-primary rounded-none border-0"
                            } else {
                                "flex-1 btn btn-xs btn-ghost rounded-none border-0 text-[#8b949e]"
                            },
                            onclick: move |_| *mode.write() = EntryMode::TimeInputs,
                            "Time Inputs"
                        }
                        button {
                            r#type: "button",
                            class: if *mode.read() == EntryMode::Duration {
                                "flex-1 btn btn-xs btn-primary rounded-none border-0"
                            } else {
                                "flex-1 btn btn-xs btn-ghost rounded-none border-0 text-[#8b949e]"
                            },
                            onclick: move |_| *mode.write() = EntryMode::Duration,
                            "Duration"
                        }
                    }

                    // Labor Code
                    div { class: "flex flex-col gap-1",
                        label { class: "text-xs font-medium text-[#8b949e] uppercase tracking-wide",
                            "Labor Code"
                        }
                        select {
                            class: "pd-select input input-bordered w-full text-sm",
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
                    div { class: "flex flex-col gap-1",
                        label { class: "text-xs font-medium text-[#8b949e] uppercase tracking-wide",
                            "Hour Type"
                        }
                        select {
                            class: "pd-select input input-bordered w-full text-sm",
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

                    // Start Time
                    div { class: "flex flex-col gap-1",
                        label { class: "text-xs font-medium text-[#8b949e] uppercase tracking-wide",
                            "Start Time"
                        }
                        div { class: "relative",
                            input {
                                r#type: "time",
                                class: "input input-bordered w-full text-sm",
                                value: "{start_time}",
                                oninput: move |e| *start_time.write() = e.value(),
                            }
                            button {
                                r#type: "button",
                                class: "pd-now-btn",
                                onclick: move |_| *start_time.write() = start_now_hhmm(),
                                "NOW"
                            }
                        }
                    }

                    // End Time (Time Inputs mode) or Duration (Duration mode)
                    if *mode.read() == EntryMode::TimeInputs {
                        div { class: "flex flex-col gap-1",
                            label { class: "text-xs font-medium text-[#8b949e] uppercase tracking-wide",
                                "End Time (optional)"
                            }
                            div { class: "relative",
                                input {
                                    r#type: "time",
                                    class: "input input-bordered w-full text-sm",
                                    value: "{end_time}",
                                    oninput: move |e| *end_time.write() = e.value(),
                                }
                                button {
                                    r#type: "button",
                                    class: "pd-now-btn",
                                    onclick: move |_| *end_time.write() = end_now_hhmm(),
                                    "NOW"
                                }
                            }
                        }
                    } else {
                        div { class: "flex flex-col gap-1",
                            label { class: "text-xs font-medium text-[#8b949e] uppercase tracking-wide",
                                "Duration (hours)"
                            }
                            input {
                                r#type: "number",
                                class: "input input-bordered w-full text-sm",
                                min: "0.25", max: "24", step: "0.25",
                                value: "{duration}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<f64>() { *duration.write() = v; }
                                },
                            }
                        }
                    }

                    // Telework
                    div { class: "flex items-center justify-between",
                        label { class: "text-xs font-medium text-[#8b949e] uppercase tracking-wide",
                            "Telework"
                        }
                        input {
                            r#type: "checkbox",
                            class: "toggle toggle-primary",
                            checked: *telework.read(),
                            onchange: move |e| *telework.write() = e.checked(),
                        }
                    }
                }

                // ── Error banner ─────────────────────────────────────────
                if let Some(ref msg) = *error.read() {
                    div { class: "mx-5 mb-2 alert alert-error text-sm py-2",
                        span { "{msg}" }
                    }
                }

                // ── Footer ───────────────────────────────────────────────
                div { class: "flex-shrink-0 flex gap-2 px-5 py-4 border-t border-[#30363d]",
                    button {
                        r#type: "button",
                        class: "btn btn-ghost btn-sm",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-primary btn-sm flex-1",
                        onclick: handle_submit,
                        if is_editing { "Update" } else { "Create" }
                    }
                }
            }
        }
    }
}
