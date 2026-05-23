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
