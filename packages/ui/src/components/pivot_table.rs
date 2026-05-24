use dioxus::prelude::*;
use api::TimecardEntryView;
use std::collections::HashMap;
use chrono::{NaiveDate, Datelike, Weekday};
use crate::utils::{format_day_col, overlapping_ids};

// ── Internal data structures ────────────────────────────────────────────────

struct PivotRow {
    labor_code_id:    i64,
    hour_type_id:     i64,
    wbs_number:       String,
    labor_code_name:  String,
    hour_type_code:   String,
    telework:         bool,
    /// Hours per day, indexed in the same order as the `days` slice.
    cells:            Vec<Option<f64>>,
    total:            f64,
}

/// Build the pivot matrix from a flat list of entries and an ordered day list.
fn build_pivot(entries: &[TimecardEntryView], days: &[String]) -> Vec<PivotRow> {
    // Unique row keys in insertion order
    type RowKey = (i64, i64, bool); // (labor_code_id, hour_type_id, telework)
    let mut row_order: Vec<RowKey> = Vec::new();
    let mut meta: HashMap<RowKey, (String, String, String)> = HashMap::new(); // → (wbs, name, ht_code)
    let mut cells: HashMap<(RowKey, String), f64> = HashMap::new();

    for entry in entries {
        if let Some(h) = entry.decimal_hours {
            let key: RowKey = (entry.labor_code_id, entry.hour_type_id, entry.telework);
            if !row_order.contains(&key) {
                row_order.push(key);
                meta.insert(key, (
                    entry.wbs_number.clone(),
                    entry.labor_code_name.clone(),
                    entry.hour_type_code.clone(),
                ));
            }
            *cells.entry((key, entry.date.to_string())).or_default() += h;
        }
    }

    // Sort by wbs_number → hour_type_code → telework for stable ordering
    row_order.sort_by(|a, b| {
        let ma = &meta[a];
        let mb = &meta[b];
        ma.0.cmp(&mb.0).then(ma.2.cmp(&mb.2)).then(a.2.cmp(&b.2))
    });

    row_order
        .iter()
        .map(|key| {
            let (wbs, name, ht) = meta[key].clone();
            let day_cells: Vec<Option<f64>> = days
                .iter()
                .map(|d| cells.get(&(*key, d.clone())).copied())
                .collect();
            let total: f64 = day_cells.iter().filter_map(|c| *c).sum();
            PivotRow {
                labor_code_id: key.0,
                hour_type_id:  key.1,
                wbs_number: wbs,
                labor_code_name: name,
                hour_type_code: ht,
                telework: key.2,
                cells: day_cells,
                total,
            }
        })
        .collect()
}

// ── Component ───────────────────────────────────────────────────────────────

/// Pivot table for Week and Pay Period tabs.
///
/// Rows are grouped by (labor code, hour type, telework).
/// Columns are the days in `days` (YYYY-MM-DD). Clicking a cell with hours
/// fires `on_day_click` with `(date, Some(entry))` when exactly one entry
/// matches, or `(date, None)` when there are zero or multiple matches.
#[component]
pub fn PivotTable(
    entries:      Vec<TimecardEntryView>,
    days:         Vec<String>,
    on_day_click: EventHandler<(String, Option<TimecardEntryView>)>,
) -> Element {
    if entries.is_empty() {
        return rsx! {
            p { class: "text-[#8b949e] py-8 text-center text-sm",
                "No entries for this period."
            }
        };
    }

    let rows      = build_pivot(&entries, &days);
    let n_days    = days.len();
    let overlap_ids = overlapping_ids(&entries);

    // Per-column totals
    let col_totals: Vec<f64> = (0..n_days)
        .map(|i| rows.iter().filter_map(|r| r.cells[i]).sum())
        .collect();
    let grand_total: f64 = col_totals.iter().sum();

    // Visible column indices: always show weekdays; show weekends only if they have hours
    let is_weekend = |date: &str| -> bool {
        NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map(|d| matches!(d.weekday(), Weekday::Sat | Weekday::Sun))
            .unwrap_or(false)
    };    let visible: Vec<usize> = (0..n_days)
        .filter(|&i| !is_weekend(&days[i]) || col_totals[i] > 0.0)
        .collect();

    // Shared TH class
    let th = "text-xs text-[#8b949e] uppercase tracking-[0.07em] font-semibold \
              px-2 py-3 border-b border-[#21262d] text-center whitespace-nowrap";
    let th_left = "text-xs text-[#8b949e] uppercase tracking-[0.07em] font-semibold \
                   px-3 py-3 border-b border-[#21262d] text-left whitespace-nowrap max-w-[180px]";

    rsx! {
        div { class: "w-full overflow-auto",
            table { class: "w-full border border-[#21262d] rounded-lg overflow-hidden border-collapse",
                // ── Header ─────────────────────────────────────────────
                thead { class: "sticky top-0 z-10",
                    tr { class: "bg-[#161b22]",
                        th { class: "{th_left}", "Code" }
                        {visible.iter().map(|&i| {
                            let day = &days[i];
                            let day_parsed = NaiveDate::parse_from_str(day, "%Y-%m-%d")
                                .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1970,1,1).unwrap());
                            let (wday, mdate) = format_day_col(day_parsed);
                            rsx! {
                                th { key: "{day}", class: "{th}",
                                    div { class: "flex flex-col items-center leading-tight gap-0.5",
                                        span { "{wday}" }
                                        span { class: "text-[11px] font-normal text-[#6e7681]", "{mdate}" }
                                    }
                                }
                            }
                        })}
                        th { class: "{th}", "Total" }
                    }
                }

                // ── Body ───────────────────────────────────────────────
                tbody {
                    {rows.iter().enumerate().map(|(row_idx, row)| {
                        let lc_id  = row.labor_code_id;
                        let ht_id  = row.hour_type_id;
                        let tw     = row.telework;
                        let day_cells = visible.iter().map(|&col_idx| {
                            let day = days[col_idx].clone();
                            let hours = row.cells[col_idx];
                            // Find matching entries for this cell
                            let matched: Vec<TimecardEntryView> = entries
                                .iter()
                                .filter(|e| {
                                    e.date.to_string() == day
                                        && e.labor_code_id == lc_id
                                        && e.hour_type_id  == ht_id
                                        && e.telework      == tw
                                })
                                .cloned()
                                .collect();
                            let cell_overlaps = matched.iter().any(|e| overlap_ids.contains(&e.id));
                            let single = if matched.len() == 1 { matched.into_iter().next() } else { None };
                            rsx! {
                                td { key: "{day}",
                                    class: "px-2 py-2 text-center border-b border-[#21262d]",
                                    class: if cell_overlaps { "bg-red-950/30" } else { "" },
                                    if let Some(h) = hours {
                                        button {
                                            class: if cell_overlaps {
                                                "font-mono text-sm font-bold text-red-400 \
                                                 hover:text-red-300 transition-colors cursor-pointer \
                                                 px-1.5 rounded"
                                            } else {
                                                "font-mono text-sm font-bold text-[#e6edf3] \
                                                 hover:text-[#58a6ff] transition-colors cursor-pointer \
                                                 px-1.5 rounded"
                                            },
                                            onclick: move |_| on_day_click.call((day.clone(), single.clone())),
                                            "{h:.1}"
                                        }
                                    } else {
                                        button {
                                            class: "text-[#30363d] hover:text-[#58a6ff] transition-colors \
                                                    cursor-pointer px-1.5 rounded text-xs",
                                            onclick: move |_| on_day_click.call((day.clone(), None)),
                                            "·"
                                        }
                                    }
                                }
                            }
                        });
                        let ht_class = if row.hour_type_code.to_uppercase() == "OT" {
                            "pd-type-ot font-mono text-xs"
                        } else {
                            "pd-type-reg font-mono text-xs"
                        };
                        let row_total = row.total;
                        rsx! {
                            tr { key: "{row_idx}",
                                class: "hover:bg-[#161b2280] transition-colors",
                                // Code cell with inline type + TW badges
                                td { class: "px-3 py-2 border-b border-[#21262d]",
                                    div { class: "flex flex-col gap-0.5",
                                        span { class: "text-[#e6edf3] text-sm", "{row.labor_code_name}" }
                                        div { class: "flex items-center gap-1",
                                            span { class: "{ht_class} text-[10px]", "{row.hour_type_code}" }
                                            if row.telework {
                                                span { class: "pd-tw-badge text-[10px] px-1 py-px", "TW" }
                                            }
                                        }
                                    }
                                }
                                // Day cells
                                {day_cells}
                                // Row total
                                td { class: "px-3 py-2 text-right font-mono text-sm font-bold \
                                             text-[#e6edf3] border-b border-[#21262d]",
                                    "{row_total:.1}"
                                }
                            }
                        }
                    })}
                }

                // ── Footer totals ───────────────────────────────────────
                tfoot {
                    tr { class: "bg-[#161b22] border-t-2 border-[#21262d]",
                        td { class: "px-3 py-2.5 text-[11px] font-semibold text-[#8b949e] \
                                     uppercase tracking-wide",
                            "Total"
                        }
                        {visible.iter().map(|&i| {
                            let day = days[i].clone();
                            let total = col_totals[i];
                            rsx! {
                                td { key: "{day}",
                                    class: "px-2 py-2.5 text-center",
                                    if total > 0.0 {
                                        button {
                                            class: "font-mono text-sm font-bold text-[#e6edf3] \
                                                    hover:text-[#58a6ff] transition-colors cursor-pointer \
                                                    px-1.5 rounded",
                                            onclick: move |_| on_day_click.call((day.clone(), None)),
                                            "{total:.1}"
                                        }
                                    } else {
                                        button {
                                            class: "text-[#30363d] hover:text-[#58a6ff] transition-colors \
                                                    cursor-pointer px-1.5 rounded text-xs",
                                            onclick: move |_| on_day_click.call((day.clone(), None)),
                                            "·"
                                        }
                                    }
                                }
                            }
                        })}
                        td { class: "px-3 py-2.5 text-right font-mono text-sm font-bold text-[#e6edf3]",
                            "{grand_total:.1}"
                        }
                    }
                }
            }
        }
    }
}
