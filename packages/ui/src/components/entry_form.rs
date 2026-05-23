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
    let editing_for_effect = editing.clone();
    use_effect(move || {
        if let Some(ref e) = editing_for_effect {
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

    let editing_for_submit = editing.clone();
    let date_for_submit    = date.clone();
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

        let editing_val = editing_for_submit.clone();
        let date_val    = date_for_submit.clone();
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
