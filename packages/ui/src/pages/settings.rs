use dioxus::prelude::*;
use api::{
    CreateHourType, CreateLaborCode, HourType, LaborCode,
    PayPeriodAnchor, Repository, UpdateHourType, UpdateLaborCode,
};

#[component]
pub fn Settings() -> Element {
    let labor_codes = use_context::<Signal<Vec<LaborCode>>>();
    let hour_types  = use_context::<Signal<Vec<HourType>>>();
    let anchors     = use_context::<Signal<Vec<PayPeriodAnchor>>>();
    let mut error       = use_signal(|| Option::<String>::None);

    // --- Labor Codes form state ---
    let mut lc_wbs      = use_signal(|| String::new());
    let mut lc_name     = use_signal(|| String::new());
    let mut editing_lc  = use_signal(|| Option::<LaborCode>::None);

    // --- Hour Types form state ---
    let mut ht_code     = use_signal(|| String::new());
    let mut ht_name     = use_signal(|| String::new());
    let mut editing_ht  = use_signal(|| Option::<HourType>::None);

    // --- Pay Period Anchor form state ---
    let mut anchor_date = use_signal(|| String::new());

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
        if code.is_empty() || name.is_empty() { return; }
        let editing = editing_ht.read().clone();
        let mut ht_sig  = hour_types;
        let mut edit    = editing_ht;
        let mut code_s  = ht_code;
        let mut name_s  = ht_name;
        let mut err     = error;
        spawn(async move {
            let repo = Repository::new(api::pool());
            let result = if let Some(ref e) = editing {
                repo.update_hour_type(&UpdateHourType { id: e.id, code, name }).await
            } else {
                repo.create_hour_type(&CreateHourType { code, name }).await
            };
            match result {
                Ok(_) => {
                    if let Ok(d) = repo.list_hour_types().await { *ht_sig.write() = d; }
                    *edit.write() = None;
                    *code_s.write() = String::new();
                    *name_s.write() = String::new();
                }
                Err(e) => *err.write() = Some(e.to_string()),
            }
        });
    };

    let cancel_ht = move |_| {
        *editing_ht.write() = None;
        *ht_code.write() = String::new();
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
                                repo.import_lookup_data(&payload.labor_codes, &payload.hour_types).await;
                                if let Ok(d) = repo.list_labor_codes().await { *lc_sig.write() = d; }
                                if let Ok(d) = repo.list_hour_types().await  { *ht_sig.write() = d; }
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
                Err(e) => { *err.write() = Some(e.to_string()); return; }
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

    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold", "Settings" }

            // Error banner
            if let Some(ref msg) = *error.read() {
                div { class: "alert alert-error",
                    span { "{msg}" }
                    button { class: "btn btn-xs btn-ghost ml-auto", onclick: move |_| *error.write() = None, "✕" }
                }
            }

            // --- Pay Period Anchors ---
            div { class: "card bg-base-200 shadow",
                div { class: "card-body",
                    h2 { class: "card-title text-lg", "Pay Period Anchors" }
                    p { class: "text-sm text-base-content/60 mb-3",
                        "Each anchor starts a 2-week pay period cycle that repeats forward indefinitely."
                    }
                    div { class: "flex gap-2 mb-3",
                        input {
                            r#type: "date",
                            class: "input input-bordered input-sm",
                            value: "{anchor_date}",
                            oninput: move |e| *anchor_date.write() = e.value(),
                        }
                        button { class: "btn btn-sm btn-primary", onclick: add_anchor, "Add" }
                    }
                    if !anchors.read().is_empty() {
                        table { class: "table table-sm",
                            thead { tr { th { "Start Date" } th { } } }
                            tbody {
                                for a in anchors.read().iter() {
                                    tr { key: "{a.id}",
                                        td { code { "{a.start_date}" } }
                                        td {
                                            button {
                                                class: "btn btn-xs btn-ghost text-error",
                                                onclick: { let id = a.id; move |_| remove_anchor(id) },
                                                "Remove"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // --- Labor Codes ---
            div { class: "card bg-base-200 shadow",
                div { class: "card-body",
                    h2 { class: "card-title text-lg", "Labor Codes" }
                    div { class: "flex gap-2 mb-3 flex-wrap",
                        input {
                            class: "input input-bordered input-sm w-32",
                            placeholder: "WBS Number",
                            value: "{lc_wbs}",
                            oninput: move |e| *lc_wbs.write() = e.value(),
                        }
                        input {
                            class: "input input-bordered input-sm flex-1 min-w-32",
                            placeholder: "Name",
                            value: "{lc_name}",
                            oninput: move |e| *lc_name.write() = e.value(),
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            onclick: save_lc,
                            if editing_lc.read().is_some() { "Update" } else { "Add" }
                        }
                        if editing_lc.read().is_some() {
                            button { class: "btn btn-sm btn-ghost", onclick: cancel_lc, "Cancel" }
                        }
                    }
                    if !labor_codes.read().is_empty() {
                        table { class: "table table-sm",
                            thead { tr { th { "WBS" } th { "Name" } th { } } }
                            tbody {
                                for lc in labor_codes.read().iter() {
                                    tr { key: "{lc.id}",
                                        td { code { class: "text-xs", "{lc.wbs_number}" } }
                                        td { "{lc.name}" }
                                        td { class: "flex gap-1",
                                            button {
                                                class: "btn btn-xs btn-ghost",
                                                onclick: {
                                                    let lc = lc.clone();
                                                    move |_| {
                                                        *lc_wbs.write()    = lc.wbs_number.clone();
                                                        *lc_name.write()   = lc.name.clone();
                                                        *editing_lc.write() = Some(lc.clone());
                                                    }
                                                },
                                                "Edit"
                                            }
                                            button {
                                                class: "btn btn-xs btn-ghost text-error",
                                                onclick: { let id = lc.id; move |_| delete_lc(id) },
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

            // --- Hour Types ---
            div { class: "card bg-base-200 shadow",
                div { class: "card-body",
                    h2 { class: "card-title text-lg", "Hour Types" }
                    div { class: "flex gap-2 mb-3 flex-wrap",
                        input {
                            class: "input input-bordered input-sm w-20",
                            placeholder: "REG",
                            value: "{ht_code}",
                            oninput: move |e| *ht_code.write() = e.value(),
                        }
                        input {
                            class: "input input-bordered input-sm flex-1 min-w-32",
                            placeholder: "Name",
                            value: "{ht_name}",
                            oninput: move |e| *ht_name.write() = e.value(),
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            onclick: save_ht,
                            if editing_ht.read().is_some() { "Update" } else { "Add" }
                        }
                        if editing_ht.read().is_some() {
                            button { class: "btn btn-sm btn-ghost", onclick: cancel_ht, "Cancel" }
                        }
                    }
                    if !hour_types.read().is_empty() {
                        table { class: "table table-sm",
                            thead { tr { th { "Code" } th { "Name" } th { } } }
                            tbody {
                                for ht in hour_types.read().iter() {
                                    tr { key: "{ht.id}",
                                        td { code { class: "badge badge-ghost badge-sm", "{ht.code}" } }
                                        td { "{ht.name}" }
                                        td { class: "flex gap-1",
                                            button {
                                                class: "btn btn-xs btn-ghost",
                                                onclick: {
                                                    let ht = ht.clone();
                                                    move |_| {
                                                        *ht_code.write()   = ht.code.clone();
                                                        *ht_name.write()   = ht.name.clone();
                                                        *editing_ht.write() = Some(ht.clone());
                                                    }
                                                },
                                                "Edit"
                                            }
                                            button {
                                                class: "btn btn-xs btn-ghost text-error",
                                                onclick: { let id = ht.id; move |_| delete_ht(id) },
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

            // --- Import / Export ---
            div { class: "card bg-base-200 shadow",
                div { class: "card-body",
                    h2 { class: "card-title text-lg", "Import / Export" }
                    p { class: "text-sm text-base-content/60 mb-4",
                        {"JSON format: { \"labor_codes\": [{\"wbs_number\":\"…\",\"name\":\"…\"}], \"hour_types\": [{\"code\":\"…\",\"name\":\"…\"}] }"}
                    }
                    div { class: "flex gap-3 flex-wrap items-center",
                        // Import
                        label { class: "btn btn-sm btn-outline",
                            "Import JSON"
                            input {
                                r#type: "file",
                                class: "hidden",
                                accept: ".json",
                                onchange: handle_import,
                            }
                        }
                        // Export
                        button { class: "btn btn-sm btn-outline", onclick: handle_export, "Export JSON" }
                    }
                }
            }
        }
    }
}
