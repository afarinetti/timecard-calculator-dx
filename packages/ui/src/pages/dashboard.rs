use dioxus::prelude::*;
use api::{PayPeriodAnchor, Repository, TimecardEntryView};
    use crate::{
    components::{entry_form::EntryFormModal, entry_table::EntryTable, pivot_table::PivotTable},
    utils::{navigate_date, navigate_week, today, week_start_for, date_range, overlapping_ids, CurrentDateSig, CurrentWeekSig},
};

#[derive(Clone, PartialEq)]
enum DashTab { Day, Week, PayPeriod, History }

#[component]
pub fn Dashboard() -> Element {
    let mut current_date = use_context::<CurrentDateSig>().0;
    let mut current_week = use_context::<CurrentWeekSig>().0;
    let anchors          = use_context::<Signal<Vec<PayPeriodAnchor>>>();

    let mut tab           = use_signal(|| DashTab::Day);
    let mut show_form     = use_signal(|| false);
    let mut editing_entry = use_signal(|| Option::<TimecardEntryView>::None);
    let mut error         = use_signal(|| Option::<String>::None);
    let mut reload        = use_signal(|| 0u32);
    let mut pp_offset     = use_signal(|| 0i32);

    // Day data — re-fetches when current_date or reload changes
    let day_data = use_resource(move || async move {
        let date = current_date();
        let _r   = reload();
        Repository::new(api::pool()).get_day_summary(&date).await
    });

    // Week data
    let week_data = use_resource(move || async move {
        let week = current_week();
        let _r   = reload();
        Repository::new(api::pool()).get_week_summary(&week).await
    });

    // Pay period data — recomputes when anchors, current_date, pp_offset, or reload change
    let pp_data = use_resource(move || async move {
        let anchors_val = anchors();
        let date        = current_date();
        let offset      = pp_offset();
        let _r          = reload();
        if anchors_val.is_empty() { return None; }
        let periods = Repository::compute_pay_periods(&anchors_val, &date);
        if periods.is_empty() { return None; }
        let base_idx = periods.iter()
            .position(|p| p.start_date <= date && p.end_date >= date)
            .unwrap_or_else(|| periods.len().saturating_sub(1));
        let target_idx = ((base_idx as i32) + offset)
            .clamp(0, (periods.len() as i32) - 1) as usize;
        let can_prev = target_idx > 0;
        let can_next = target_idx < periods.len() - 1;
        let period = periods[target_idx].clone();
        let summary = Repository::new(api::pool())
            .get_pay_period_summary(&period.start_date, &period.end_date)
            .await.ok()?;
        Some((period, summary, can_prev, can_next))
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
    let (today_hrs, today_error, today_wip) = if let Some(Ok(s)) = day_data.read().as_ref() {
        (Some(s.total_hours.max(0.0)), !overlapping_ids(&s.entries).is_empty(), s.entries.iter().any(|e| e.end_time.is_none()))
    } else {
        (None, false, false)
    };
    let (week_hrs, week_error) = if let Some(Ok(s)) = week_data.read().as_ref() {
        (Some(s.total_hours.max(0.0)), !overlapping_ids(&s.entries).is_empty())
    } else {
        (None, false)
    };
    let (pp_hrs, pp_error) = if let Some((_, summary, _, _)) = pp_data.read().as_ref().and_then(|opt| opt.as_ref()) {
        (Some(summary.total_hours.max(0.0)), !overlapping_ids(&summary.entries).is_empty())
    } else {
        (None, false)
    };

    let open_add_entry = move |_| {
        *editing_entry.write() = None;
        *show_form.write() = true;
    };

    rsx! {
        div { class: "flex flex-col h-full p-4 gap-4 overflow-hidden",

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
                        p { class: if today_error { "pd-stat-value-error" } else if today_wip { "pd-stat-value-wip" } else if h > 8.0 { "pd-stat-value-ok" } else { "pd-stat-value" },
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
                        p { class: if week_error { "pd-stat-value-error" } else if h > 40.0 { "pd-stat-value-ok" } else { "pd-stat-value" },
                            "{h:.1}"
                            span { class: "text-[13px] text-[#8b949e] font-normal ml-0.5", "h" }
                        }
                    } else {
                        p { class: "pd-stat-value text-[#8b949e]", "—" }
                    }
                }
                // Pay Period card
                div { class: "bg-[#161b22] border border-[#21262d] rounded-lg px-4 py-3.5",
                    p { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] mb-1.5", "Pay Period" }
                    if let Some(h) = pp_hrs {
                        p { class: if pp_error { "pd-stat-value-error" } else if h > 80.0 { "pd-stat-value-ok" } else { "pd-stat-value" },
                            "{h:.1}"
                            span { class: "text-[13px] text-[#8b949e] font-normal ml-0.5", "h" }
                        }
                    } else {
                        p { class: "pd-stat-value text-[#8b949e]", "—" }
                    }
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
                        button {
                            class: "text-xs text-[#8b949e] px-2 py-1 border border-[#30363d] rounded-[4px] hover:border-[#58a6ff] hover:text-[#58a6ff] transition-colors",
                            onclick: move |_| *current_week.write() = week_start_for(&today()),
                            "This Week"
                        }
                    }
                    if *tab.read() == DashTab::PayPeriod {
                        match pp_data.read().as_ref().and_then(|o| o.as_ref()) {
                            Some((period, _, can_prev, can_next)) => {
                                let can_prev = *can_prev;
                                let can_next = *can_next;
                                let label = format!("{} — {}", period.start_date, period.end_date);
                                rsx! {
                                    button {
                                        class: "border border-[#30363d] text-[#8b949e] hover:border-[#58a6ff] hover:text-[#58a6ff] px-2.5 py-1 rounded-[5px] text-sm leading-none transition-colors disabled:opacity-30 disabled:cursor-not-allowed",
                                        disabled: !can_prev,
                                        onclick: move |_| *pp_offset.write() -= 1,
                                        "‹"
                                    }
                                    span { class: "text-sm font-semibold text-[#e6edf3] min-w-[220px] text-center", "{label}" }
                                    button {
                                        class: "border border-[#30363d] text-[#8b949e] hover:border-[#58a6ff] hover:text-[#58a6ff] px-2.5 py-1 rounded-[5px] text-sm leading-none transition-colors disabled:opacity-30 disabled:cursor-not-allowed",
                                        disabled: !can_next,
                                        onclick: move |_| *pp_offset.write() += 1,
                                        "›"
                                    }
                                    button {
                                       class: "text-xs text-[#8b949e] px-2 py-1 border border-[#30363d] rounded-[4px] hover:border-[#58a6ff] hover:text-[#58a6ff] transition-colors",
                                       onclick: move |_| *pp_offset.write() = 0,
                                       "This Pay Period"
                                    }
                                }
                            },
                            None => rsx! {},
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
            div { class: "flex-1 min-h-0 flex flex-col overflow-hidden",
            match *tab.read() {
                DashTab::Day => rsx! {
                    div { class: "flex-1 overflow-auto min-h-0",
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
                                show_date: false,
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
                        Some(Ok(summary)) => {
                            let week = current_week();
                            let days: Vec<String> = (0..7).map(|i| navigate_date(&week, i)).collect();
                            rsx! {
                                div { class: "flex-1 overflow-auto min-h-0",
                                    PivotTable {
                                        entries: summary.entries.clone(),
                                        days,
                                        on_day_click: move |(date, entry): (String, Option<_>)| {
                                            *current_date.write() = date;
                                            *tab.write() = DashTab::Day;
                                            if let Some(e) = entry {
                                                *editing_entry.write() = Some(e);
                                                *show_form.write() = true;
                                            }
                                        },
                                    }
                                    div { class: "flex justify-end mt-2 text-xs text-[#8b949e]",
                                        "Total: "
                                        span { class: "text-[#e6edf3] font-mono font-bold ml-1",
                                            "{summary.total_hours:.2} hrs"
                                        }
                                    }
                                }
                            }
                        },
                    }
                },
                DashTab::PayPeriod => rsx! {
                    if anchors.read().is_empty() {
                        div { class: "text-[#8b949e] py-10 text-center text-sm",
                            "Configure pay period anchors in Settings to enable this view."
                        }
                    } else {
                        match pp_data.read().as_ref() {
                            None => rsx! {
                                div { class: "flex justify-center py-8",
                                    span { class: "loading loading-spinner" }
                                }
                            },
                            Some(None) => rsx! {
                                div { class: "text-[#8b949e] py-10 text-center text-sm",
                                    "No pay period found for the current date."
                                }
                            },
                            Some(Some((period, summary, can_prev, can_next))) => {
                                let _can_prev = *can_prev;
                                let _can_next = *can_next;
                                rsx! {
                                    div { class: "flex-1 overflow-auto min-h-0",
                                        PivotTable {
                                            entries: summary.entries.clone(),
                                            days: date_range(&period.start_date, &period.end_date),
                                            on_day_click: move |(date, entry): (String, Option<_>)| {
                                                *current_date.write() = date;
                                                *tab.write() = DashTab::Day;
                                                if let Some(e) = entry {
                                                    *editing_entry.write() = Some(e);
                                                    *show_form.write() = true;
                                                }
                                            },
                                        }
                                        div { class: "flex justify-end mt-2 text-xs text-[#8b949e]",
                                            "Total: "
                                            span { class: "text-[#e6edf3] font-mono font-bold ml-1",
                                                "{summary.total_hours:.2} hrs"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                DashTab::History => rsx! {
                    div { class: "text-[#8b949e] py-10 text-center text-sm",
                        "Select a pay period to view historical entries."
                    }
                },
            }
            } // end tab panel container

            // Entry form modal — always in RSX, show controls visibility
            EntryFormModal {
                show: show_form,
                editing: editing_entry,
                date: current_date.read().clone(),
                on_close: move |_| *show_form.write() = false,
                on_saved: move |_| { *show_form.write() = false; refresh(); },
            }
        }
    }
}
