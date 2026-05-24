use dioxus::prelude::*;
use api::{
    CreateHourType, CreateLaborCode, HourType, LaborCode,
    PayPeriodAnchor, Repository, UpdateHourType, UpdateLaborCode,
    ExportEntriesPayload,
};

#[component]
pub fn Settings() -> Element {
    let labor_codes = use_context::<Signal<Vec<LaborCode>>>();
    let hour_types  = use_context::<Signal<Vec<HourType>>>();
    let anchors     = use_context::<Signal<Vec<PayPeriodAnchor>>>();
    let mut error       = use_signal(|| Option::<String>::None);

    // --- Labor Codes form state ---
    let mut lc_wbs      = use_signal(String::new);
    let mut lc_name     = use_signal(String::new);
    let mut editing_lc  = use_signal(|| Option::<LaborCode>::None);

    // --- Hour Types form state ---
    let mut ht_code     = use_signal(String::new);
    let mut ht_name     = use_signal(String::new);
    let mut editing_ht  = use_signal(|| Option::<HourType>::None);
    let mut ht_badge_class = use_signal(String::new);

    // --- Pay Period Anchor form state ---
    let mut anchor_date = use_signal(String::new);

    let mut active_pane = use_signal(|| 0u8); // 0=PayPeriod, 1=LaborCodes, 2=HourTypes, 3=ImportExport

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
        let badge = ht_badge_class.read().trim().to_string();
        if code.is_empty() || name.is_empty() { return; }
        let editing = editing_ht.read().clone();
        let mut ht_sig  = hour_types;
        let mut edit    = editing_ht;
        let mut code_s  = ht_code;
        let mut name_s  = ht_name;
        let mut badge_s = ht_badge_class;
        let mut err     = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            let result = if let Some(ref e) = editing {
                repo.update_hour_type(&UpdateHourType { id: e.id, code, name, badge_class: badge }).await
            } else {
                repo.create_hour_type(&CreateHourType { code, name, badge_class: badge }).await
            };
            match result {
                Ok(_) => {
                    if let Ok(d) = repo.list_hour_types().await { *ht_sig.write() = d; }
                    *edit.write() = None;
                    *code_s.write() = String::new();
                    *name_s.write() = String::new();
                    *badge_s.write() = String::new();
                }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    let cancel_ht = move |_| {
        *editing_ht.write() = None;
        *ht_code.write() = String::new();
        *ht_badge_class.write() = String::new();
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
            if let Some(file) = files.into_iter().next() {
                match file.read_string().await {
                    Ok(text) => {
                        match serde_json::from_str::<api::ImportPayload>(&text) {
                            Ok(payload) => {
                                let repo = Repository::new(api::pool());
                                match repo.import_lookup_data(&payload.labor_codes, &payload.hour_types).await {
                                    Ok(_) => {
                                        if let Ok(d) = repo.list_labor_codes().await { *lc_sig.write() = d; }
                                        if let Ok(d) = repo.list_hour_types().await  { *ht_sig.write() = d; }
                                    }
                                    Err(e) => *err.write() = Some(e.to_string()),
                                }
                            }
                            Err(e) => *err.write() = Some(format!("Invalid JSON: {e}")),
                        }
                    }
                    Err(e) => *err.write() = Some(format!("Failed to read file: {e}")),
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
                Err(e) => { *err.write() = Some(e.to_string());}
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

    // ---- Entry Export handler ----

    let handle_export_entries = move |_| {
        let mut err = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            match repo.export_entries().await {
                Err(e) => { *err.write() = Some(e.to_string()); }
                Ok(payload) => {
                    let json = match serde_json::to_string_pretty(&payload) {
                        Ok(s) => s,
                        Err(e) => { *err.write() = Some(e.to_string()); return; }
                    };
                    if let Some(path) = rfd::AsyncFileDialog::new()
                        .set_title("Export timecard entries")
                        .set_file_name("timecard-entries.json")
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

    // ---- Entry Import handler ----

    let handle_import_entries = move |e: Event<FormData>| {
        let files = e.files();
        let mut err = error;
        spawn(async move {
            if let Some(file) = files.into_iter().next() {
                match file.read_string().await {
                    Ok(text) => {
                        match serde_json::from_str::<ExportEntriesPayload>(&text) {
                            Ok(payload) => {
                                let repo = Repository::new(api::pool());
                                match repo.import_entries(&payload.entries).await {
                                    Ok(count) => {
                                        *err.write() = Some(format!("Imported {} entries.", count));
                                    }
                                    Err(e) => *err.write() = Some(e.to_string()),
                                }
                            }
                            Err(e) => *err.write() = Some(format!("Invalid JSON: {e}")),
                        }
                    }
                    Err(e) => *err.write() = Some(format!("Failed to read file: {e}")),
                }
            }
        });
    };

    rsx! {
        div { class: "flex h-full",

            // ── Sub-nav (180px) ──
            div { class: "w-[180px] bg-[#0d1117] border-r border-[#21262d] flex flex-col p-3 gap-0.5 flex-shrink-0",
                p { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.08em] font-semibold px-2 mb-1.5 mt-1",
                    "Settings"
                }
                for (i, label) in [(0u8, "Pay Period"), (1u8, "Labor Codes"), (2u8, "Hour Types"), (3u8, "Import / Export")] {
                    button {
                        class: if *active_pane.read() == i { "pd-subnav-active w-full text-left" }
                               else { "pd-subnav-item w-full text-left" },
                        onclick: move |_| { *active_pane.write() = i; *error.write() = None; },
                        "{label}"
                    }
                }
            }

            // ── Content pane ──
            div { class: "flex-1 overflow-y-auto px-8 py-7",

                // Error banner
                if let Some(ref msg) = *error.read() {
                    div { class: "alert alert-error mb-6",
                        span { "{msg}" }
                        button { class: "btn btn-xs btn-ghost ml-auto",
                            onclick: move |_| *error.write() = None, "✕"
                        }
                    }
                }

                match *active_pane.read() {

                    // 0: Pay Period Anchors
                    0 => rsx! {
                        h1 { class: "text-lg font-bold text-[#e6edf3] mb-1", "Pay Period Anchors" }
                        p { class: "text-sm text-[#8b949e] mb-6 leading-relaxed",
                            "Each anchor date starts a 2-week pay period cycle that repeats forward indefinitely."
                        }
                        // Add form card
                        div { class: "bg-[#161b22] border border-[#21262d] rounded-lg p-4 mb-5 flex gap-3 items-end flex-wrap",
                            div { class: "flex flex-col gap-1",
                                label { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold", "Start Date" }
                                input {
                                    r#type: "date",
                                    class: "bg-[#0d1117] border border-[#30363d] rounded-[6px] px-3 py-1.5 \
                                            text-sm text-[#e6edf3] outline-none focus:border-[#58a6ff] \
                                            transition-colors cursor-pointer [color-scheme:dark]",
                                    value: "{anchor_date}",
                                    oninput: move |e| *anchor_date.write() = e.value(),
                                }
                            }
                            button { class: "btn btn-sm btn-primary", onclick: add_anchor, "Add Anchor" }
                        }
                        // Table (if not empty)
                        if !anchors.read().is_empty() {
                            table { class: "w-full border-collapse",
                                thead { tr {
                                    th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left pb-2 px-4 border-b border-[#21262d]", "Start Date" }
                                    th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-right pb-2 px-4 border-b border-[#21262d]", "Actions" }
                                } }
                                tbody {
                                    for a in anchors.read().iter() {
                                        tr { key: "{a.id}", class: "border-b border-[#21262d] last:border-b-0 hover:bg-[#161b2260]",
                                            td { class: "px-4 py-3",
                                                code { class: "font-mono text-xs text-[#8b949e] bg-[#161b22] border border-[#21262d] px-2 py-0.5 rounded", "{a.start_date}" }
                                            }
                                            td { class: "px-4 py-3 text-right",
                                                button { class: "pd-action-delete", onclick: { let id = a.id; move |_| remove_anchor(id) }, "Remove" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },

                    // 1: Labor Codes
                    1 => rsx! {
                        h1 { class: "text-lg font-bold text-[#e6edf3] mb-1", "Labor Codes" }
                        p { class: "text-sm text-[#8b949e] mb-6 leading-relaxed", "WBS codes used to categorize time entries." }
                        div { class: "bg-[#161b22] border border-[#21262d] rounded-lg p-4 mb-5 flex gap-3 items-end flex-wrap",
                            div { class: "flex flex-col gap-1",
                                label { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold", "WBS Number" }
                                input { class: "input input-bordered input-sm w-32", placeholder: "WBS-1234",
                                    value: "{lc_wbs}", oninput: move |e| *lc_wbs.write() = e.value() }
                            }
                            div { class: "flex flex-col gap-1 flex-1 min-w-32",
                                label { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold", "Name" }
                                input { class: "input input-bordered input-sm w-full", placeholder: "Overhead Admin",
                                    value: "{lc_name}", oninput: move |e| *lc_name.write() = e.value() }
                            }
                            button { class: "btn btn-sm btn-primary", onclick: save_lc,
                                if editing_lc.read().is_some() { "Update" } else { "Add" }
                            }
                            if editing_lc.read().is_some() {
                                button { class: "btn btn-sm btn-ghost", onclick: cancel_lc, "Cancel" }
                            }
                        }
                        if !labor_codes.read().is_empty() {
                            table { class: "w-full border-collapse",
                                thead { tr {
                                    th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left pb-2 px-4 border-b border-[#21262d]", "WBS" }
                                    th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left pb-2 px-4 border-b border-[#21262d]", "Name" }
                                    th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-right pb-2 px-4 border-b border-[#21262d]", "Actions" }
                                } }
                                tbody {
                                    for lc in labor_codes.read().iter() {
                                        tr { key: "{lc.id}", class: "border-b border-[#21262d] last:border-b-0 hover:bg-[#161b2260]",
                                            td { class: "px-4 py-3",
                                                code { class: "font-mono text-xs text-[#8b949e] bg-[#161b22] border border-[#21262d] px-2 py-0.5 rounded", "{lc.wbs_number}" }
                                            }
                                            td { class: "px-4 py-3 text-sm text-[#e6edf3] font-medium", "{lc.name}" }
                                            td { class: "px-4 py-3 flex gap-1.5 justify-end",
                                                button { class: "pd-action-edit", onclick: { let lc = lc.clone(); move |_| { *lc_wbs.write() = lc.wbs_number.clone(); *lc_name.write() = lc.name.clone(); *editing_lc.write() = Some(lc.clone()); } }, "Edit" }
                                                button { class: "pd-action-delete", onclick: { let id = lc.id; move |_| delete_lc(id) }, "Delete" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },

                    // 2: Hour Types
                    2 => rsx! {
                        h1 { class: "text-lg font-bold text-[#e6edf3] mb-1", "Hour Types" }
                        p { class: "text-sm text-[#8b949e] mb-6 leading-relaxed", "Classification codes for time entries (e.g. regular, overtime, holiday)." }
                        div { class: "bg-[#161b22] border border-[#21262d] rounded-lg p-4 mb-5 flex gap-3 items-end flex-wrap",
                            div { class: "flex flex-col gap-1",
                                label { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold", "Code" }
                                input { class: "input input-bordered input-sm w-20", placeholder: "REG",
                                    value: "{ht_code}", oninput: move |e| *ht_code.write() = e.value() }
                            }
                            div { class: "flex flex-col gap-1 flex-1 min-w-32",
                                label { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold", "Name" }
                                input { class: "input input-bordered input-sm w-full", placeholder: "Regular",
                                    value: "{ht_name}", oninput: move |e| *ht_name.write() = e.value() }
                            }
                            div { class: "flex flex-col gap-1 flex-1 min-w-32",
                                label { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold", "Badge Class" }
                                input { class: "input input-bordered input-sm w-full", placeholder: "pd-type-reg",
                                    value: "{ht_badge_class}", oninput: move |e| *ht_badge_class.write() = e.value() }
                            }
                            button { class: "btn btn-sm btn-primary", onclick: save_ht,
                                if editing_ht.read().is_some() { "Update" } else { "Add" }
                            }
                            if editing_ht.read().is_some() {
                                button { class: "btn btn-sm btn-ghost", onclick: cancel_ht, "Cancel" }
                            }
                        }
                        if !hour_types.read().is_empty() {
                            table { class: "w-full border-collapse",
                                thead { tr {
                                    th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left pb-2 px-4 border-b border-[#21262d]", "Code" }
                                    th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left pb-2 px-4 border-b border-[#21262d]", "Name" }
                                    th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-right pb-2 px-4 border-b border-[#21262d]", "Actions" }
                                    th { class: "text-[10px] text-[#8b949e] uppercase tracking-[0.07em] font-semibold text-left pb-2 px-4 border-b border-[#21262d]", "Badge Class" }
                                } }
                                tbody {
                                    for ht in hour_types.read().iter() {
                                        tr { key: "{ht.id}", class: "border-b border-[#21262d] last:border-b-0 hover:bg-[#161b2260]",
                                            td { class: "px-4 py-3",
                                                span { class: "{ht.badge_class} font-mono text-xs", "{ht.code}" }
                                            }
                                            td { class: "px-4 py-3 text-sm text-[#e6edf3] font-medium", "{ht.name}" }
                                            td { class: "px-4 py-3",
                                                code { class: "font-mono text-xs text-[#8b949e] bg-[#161b22] border border-[#21262d] px-2 py-0.5 rounded", "{ht.badge_class}" }
                                            }
                                            td { class: "px-4 py-3 flex gap-1.5 justify-end",
                                                button { class: "pd-action-edit", onclick: { let ht = ht.clone(); move |_| { *ht_code.write() = ht.code.clone(); *ht_name.write() = ht.name.clone(); *ht_badge_class.write() = ht.badge_class.clone(); *editing_ht.write() = Some(ht.clone()); } }, "Edit" }
                                                button { class: "pd-action-delete", onclick: { let id = ht.id; move |_| delete_ht(id) }, "Delete" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },

                    // 3: Import / Export
                    _ => rsx! {
                        h1 { class: "text-lg font-bold text-[#e6edf3] mb-1", "Import / Export" }

                        // ── Lookup Data ──
                        div { class: "bg-[#161b22] border border-[#21262d] rounded-lg p-5 mb-5",
                            h2 { class: "text-sm font-semibold text-[#e6edf3] mb-1", "Lookup Data" }
                            p { class: "text-sm text-[#8b949e] mb-4 leading-relaxed",
                                "Back up or restore labor codes and hour types as a JSON file."
                            }
                            p { class: "text-xs text-[#8b949e] font-mono mb-4",
                                {"{ \"labor_codes\": [...], \"hour_types\": [...] }"}
                            }
                            div { class: "flex gap-3 flex-wrap",
                                label { class: "btn btn-sm btn-outline",
                                    "Import JSON"
                                    input { r#type: "file", class: "hidden", accept: ".json", onchange: handle_import }
                                }
                                button { class: "btn btn-sm btn-outline", onclick: handle_export, "Export JSON" }
                            }
                        }

                        // ── Time Entries ──
                        div { class: "bg-[#161b22] border border-[#21262d] rounded-lg p-5",
                            h2 { class: "text-sm font-semibold text-[#e6edf3] mb-1", "Time Entries" }
                            p { class: "text-sm text-[#8b949e] mb-4 leading-relaxed",
                                "Back up or restore timecard entries as a JSON file."
                            }
                            p { class: "text-xs text-[#8b949e] font-mono mb-4",
                                {"{ \"entries\": [{ \"wbs_number\": \"...\", \"hour_type_code\": \"...\", \"telework\": false, \"date\": \"YYYY-MM-DD\", \"start_time\": \"HH:MM\", \"end_time\": \"HH:MM\" }] }"}
                            }
                            div { class: "flex gap-3 flex-wrap",
                                label { class: "btn btn-sm btn-outline",
                                    "Import JSON"
                                    input { r#type: "file", class: "hidden", accept: ".json", onchange: handle_import_entries }
                                }
                                button { class: "btn btn-sm btn-outline", onclick: handle_export_entries, "Export JSON" }
                            }
                        }
                    },
                }
            }
        }
    }
}
