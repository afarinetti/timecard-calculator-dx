use dioxus::prelude::*;
use api::TimecardEntryView;
use crate::utils::{format_day_label, overlapping_ids, utc_to_central_hhmm};

#[component]
pub fn EntryTable(
    entries: Vec<TimecardEntryView>,
    show_date: bool,
    on_edit: EventHandler<TimecardEntryView>,
    on_delete: EventHandler<i64>,
) -> Element {
    if entries.is_empty() {
        return rsx! {
            p { class: "text-[#8b949e] py-8 text-center text-sm", "No entries for this period." }
        };
    }

    let overlap_ids = overlapping_ids(&entries);

    rsx! {
        div { class: "overflow-x-auto",
            table { class: "w-full border border-[#21262d] rounded-lg overflow-hidden border-collapse",
                thead {
                    tr { class: "bg-[#161b22]",
                        if show_date {
                            th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left px-4 py-2.5 border-b border-[#21262d]", "Day" }
                        }
                        th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left px-4 py-2.5 border-b border-[#21262d]", "Code" }
                        th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left px-4 py-2.5 border-b border-[#21262d]", "Type" }
                        th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left px-4 py-2.5 border-b border-[#21262d]", "Start → End" }
                        th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left px-4 py-2.5 border-b border-[#21262d]", "Hrs" }
                        th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-right px-4 py-2.5 border-b border-[#21262d]", "Actions" }
                    }
                }
                tbody {
                    for entry in entries.iter() {
                        {
                            let is_overlap = overlap_ids.contains(&entry.id);
                            let row_class = "border-b border-[#21262d] last:border-b-0 hover:bg-[#161b2280] transition-colors";
                            let time_class = if is_overlap { "text-red-400 font-semibold" } else { "text-[#8b949e]" };
                            rsx! {
                                tr { key: "{entry.id}", class: "{row_class}",

                                    // Optional day column
                                    if show_date {
                                        td { class: "px-4 py-[11px] font-mono text-xs text-[#8b949e] whitespace-nowrap",
                                            "{format_day_label(entry.date)}"
                                        }
                                    }

                                    // Code name + optional TW badge
                                    td { class: "px-4 py-[11px]",
                                        div { class: "flex items-center gap-2",
                                            span { class: "text-[#e6edf3] font-medium text-sm", "{entry.labor_code_name}" }
                                            if entry.telework {
                                                span { class: "pd-tw-badge", "TW" }
                                            }
                                        }
                                    }

                                    // Hour type — color coded
                                    td { class: "px-4 py-[11px]",
                                        span {
                                            class: "{entry.hour_type_badge_class} font-mono text-xs",
                                            "{entry.hour_type_code}"
                                        }
                                    }

                                    // Start → End — red if overlapping
                                    td { class: "px-4 py-[11px] font-mono text-xs whitespace-nowrap {time_class}",
                                        if let Some(end) = entry.end_time {
                                            "{utc_to_central_hhmm(entry.start_time)} → {utc_to_central_hhmm(end)}"
                                        } else {
                                            div { class: "flex items-center gap-2",
                                                span { "{utc_to_central_hhmm(entry.start_time)} →" }
                                                span { class: "pd-in-progress", "In Progress" }
                                            }
                                        }
                                    }

                                    // Hours
                                    td { class: "px-4 py-[11px] font-mono text-sm font-bold",
                                        if let Some(h) = entry.decimal_hours {
                                            span { class: "text-[#e6edf3]", "{h:.2}" }
                                        } else {
                                            span { class: "text-[#8b949e]", "—" }
                                        }
                                    }

                                    // Actions — always visible
                                    td { class: "px-4 py-[11px]",
                                        div { class: "flex gap-1.5 justify-end",
                                            button {
                                                class: "pd-action-edit",
                                                onclick: {
                                                    let entry = entry.clone();
                                                    move |_| on_edit.call(entry.clone())
                                                },
                                                "Edit"
                                            }
                                            button {
                                                class: "pd-action-delete",
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
        }
    }
}
