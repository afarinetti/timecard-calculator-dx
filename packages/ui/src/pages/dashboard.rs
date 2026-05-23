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
                            onclick: move |_| {
                                let d = current_date.read().clone();
                                *current_date.write() = navigate_date(&d, -1);
                            },
                            "‹"
                        }
                        span { class: "text-sm font-semibold px-1", "{current_date}" }
                        button {
                            class: "btn btn-sm btn-ghost",
                            onclick: move |_| {
                                let d = current_date.read().clone();
                                *current_date.write() = navigate_date(&d, 1);
                            },
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
                            onclick: move |_| {
                                let w = current_week.read().clone();
                                *current_week.write() = navigate_week(&w, -1);
                            },
                            "‹"
                        }
                        span { class: "text-sm font-semibold px-1", "Week of {current_week}" }
                        button {
                            class: "btn btn-sm btn-ghost",
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
