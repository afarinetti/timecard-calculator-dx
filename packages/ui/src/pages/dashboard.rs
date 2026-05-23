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

    let mut refresh = move || *reload.write() += 1;

    let handle_delete = move |id: i64| {
        spawn(async move {
            match Repository::new(api::pool()).delete_timecard_entry(id).await {
                Ok(_)  => *reload.write() += 1,
                Err(e) => *error.write() = Some(e.to_string()),
            }
        });
    };

    // Derive stat values before rsx! so borrows drop
    let today_hrs = day_data.read().as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|s| s.total_hours);
    let week_hrs = week_data.read().as_ref()
        .and_then(|r| r.as_ref().ok())
        .map(|s| s.total_hours);

    let open_add_entry = move |_| {
        *editing_entry.write() = None;
        *show_form.write() = true;
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

            // Stats cards
            div { class: "grid grid-cols-3 gap-3",
                // Today card
                div { class: "bg-[#161b22] border border-[#21262d] rounded-lg px-4 py-3.5",
                    p { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] mb-1.5", "Today" }
                    if let Some(h) = today_hrs {
                        p { class: "pd-stat-value",
                            "{h:.1}"
                            span { class: "text-[13px] text-[#8b949e] font-normal ml-0.5", "h" }
                        }
                    } else {
                        p { class: "pd-stat-value text-[#8b949e]", "—" }
                    }
                }
                // This Week card
                div { class: "bg-[#161b22] border border-[#21262d] rounded-lg px-4 py-3.5",
                    p { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] mb-1.5", "This Week" }
                    if let Some(h) = week_hrs {
                        p { class: "pd-stat-value",
                            "{h:.1}"
                            span { class: "text-[13px] text-[#8b949e] font-normal ml-0.5", "h" }
                        }
                    } else {
                        p { class: "pd-stat-value text-[#8b949e]", "—" }
                    }
                }
                // Pay Period card (stub — no API yet)
                div { class: "bg-[#161b22] border border-[#21262d] rounded-lg px-4 py-3.5",
                    p { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] mb-1.5", "Pay Period" }
                    p { class: "pd-stat-value-ok", "—" }
                }
            }

            // Date nav row
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-2",
                    if *tab.read() == DashTab::Day {
                        button {
                            class: "border border-[#30363d] text-[#8b949e] hover:border-[#58a6ff] hover:text-[#58a6ff] px-2.5 py-1 rounded-[5px] text-sm leading-none transition-colors",
                            onclick: move |_| {
                                let d = current_date.read().clone();
                                *current_date.write() = navigate_date(&d, -1);
                            },
                            "‹"
                        }
                        span { class: "text-sm font-semibold text-[#e6edf3] min-w-[120px] text-center", "{current_date}" }
                        button {
                            class: "border border-[#30363d] text-[#8b949e] hover:border-[#58a6ff] hover:text-[#58a6ff] px-2.5 py-1 rounded-[5px] text-sm leading-none transition-colors",
                            onclick: move |_| {
                                let d = current_date.read().clone();
                                *current_date.write() = navigate_date(&d, 1);
                            },
                            "›"
                        }
                        button {
                            class: "text-xs text-[#8b949e] px-2 py-1 border border-[#30363d] rounded-[4px] hover:border-[#58a6ff] hover:text-[#58a6ff] transition-colors",
                            onclick: move |_| *current_date.write() = today(),
                            "Today"
                        }
                    }
                    if *tab.read() == DashTab::Week {
                        button {
                            class: "border border-[#30363d] text-[#8b949e] hover:border-[#58a6ff] hover:text-[#58a6ff] px-2.5 py-1 rounded-[5px] text-sm leading-none transition-colors",
                            onclick: move |_| {
                                let w = current_week.read().clone();
                                *current_week.write() = navigate_week(&w, -1);
                            },
                            "‹"
                        }
                        span { class: "text-sm font-semibold text-[#e6edf3] min-w-[120px] text-center", "Week of {current_week}" }
                        button {
                            class: "border border-[#30363d] text-[#8b949e] hover:border-[#58a6ff] hover:text-[#58a6ff] px-2.5 py-1 rounded-[5px] text-sm leading-none transition-colors",
                            onclick: move |_| {
                                let w = current_week.read().clone();
                                *current_week.write() = navigate_week(&w, 1);
                            },
                            "›"
                        }
                    }
                }
                button {
                    class: "btn btn-primary btn-sm",
                    onclick: open_add_entry,
                    "+ Add Entry"
                }
            }

            // Tab bar — underline style
            div { class: "flex border-b border-[#21262d]",
                for (label, this_tab) in [
                    ("Day", DashTab::Day),
                    ("Week", DashTab::Week),
                    ("Pay Period", DashTab::PayPeriod),
                    ("History", DashTab::History),
                ] {
                    {
                        let is_active = *tab.read() == this_tab;
                        rsx! {
                            button {
                                class: if is_active {
                                    "text-sm font-medium text-[#58a6ff] px-4 py-2 border-b-2 border-[#58a6ff] -mb-px transition-colors"
                                } else {
                                    "text-sm font-medium text-[#8b949e] hover:text-[#e6edf3] px-4 py-2 border-b-2 border-transparent -mb-px transition-colors"
                                },
                                onclick: move |_| *tab.write() = this_tab.clone(),
                                "{label}"
                            }
                        }
                    }
                }
            }

            // Tab panels
            match *tab.read() {
                DashTab::Day => rsx! {
                    match day_data.read().as_ref() {
                        None => rsx! {
                            div { class: "flex justify-center py-8",
                                span { class: "loading loading-spinner" }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            div { class: "alert alert-error", "{e}" }
                        },
                        Some(Ok(summary)) => rsx! {
                            EntryTable {
                                entries: summary.entries.clone(),
                                on_edit: move |entry: TimecardEntryView| {
                                    *editing_entry.write() = Some(entry);
                                    *show_form.write() = true;
                                },
                                on_delete: handle_delete,
                            }
                            div { class: "flex justify-end mt-2 text-xs text-[#8b949e]",
                                "Total: "
                                span { class: "text-[#e6edf3] font-mono font-bold ml-1",
                                    "{summary.total_hours:.2} hrs"
                                }
                            }
                        },
                    }
                },
                DashTab::Week => rsx! {
                    match week_data.read().as_ref() {
                        None => rsx! {
                            div { class: "flex justify-center py-8",
                                span { class: "loading loading-spinner" }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            div { class: "alert alert-error", "{e}" }
                        },
                        Some(Ok(summary)) => rsx! {
                            EntryTable {
                                entries: summary.entries.clone(),
                                on_edit: move |entry: TimecardEntryView| {
                                    *editing_entry.write() = Some(entry);
                                    *show_form.write() = true;
                                },
                                on_delete: handle_delete,
                            }
                            div { class: "flex justify-end mt-2 text-xs text-[#8b949e]",
                                "Total: "
                                span { class: "text-[#e6edf3] font-mono font-bold ml-1",
                                    "{summary.total_hours:.2} hrs"
                                }
                            }
                        },
                    }
                },
                DashTab::PayPeriod => rsx! {
                    div { class: "text-[#8b949e] py-10 text-center text-sm",
                        "Configure pay period anchors in Settings to enable this view."
                    }
                },
                DashTab::History => rsx! {
                    div { class: "text-[#8b949e] py-10 text-center text-sm",
                        "Select a pay period to view historical entries."
                    }
                },
            }

            // Entry form modal — always in RSX, show controls visibility
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
