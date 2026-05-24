use dioxus::prelude::*;
use api::{LaborCode, HourType, PayPeriodAnchor, Repository};
use crate::{routes::Route, utils::{today, week_start_for, CurrentDateSig, CurrentWeekSig}};

#[component]
pub fn App() -> Element {
    // Global lookup data — populated once on startup
    let labor_codes   = use_context_provider(|| Signal::new(Vec::<LaborCode>::new()));
    let hour_types    = use_context_provider(|| Signal::new(Vec::<HourType>::new()));
    let anchors       = use_context_provider(|| Signal::new(Vec::<PayPeriodAnchor>::new()));

    // Navigation state — wrapped in distinct newtypes so both signals can
    // coexist in the context store (both are Signal<String> but different TypeIds).
    let today_str = today();
    use_context_provider(|| CurrentDateSig(Signal::new(today_str.clone())));
    use_context_provider(|| CurrentWeekSig(Signal::new(week_start_for(&today_str))));

    // Load lookup data once on startup
    let _init = use_resource(move || async move {
        let pool = api::pool();
        let repo = Repository::new(pool);
        let mut lc  = labor_codes;
        let mut ht  = hour_types;
        let mut ppa = anchors;
        if let Ok(data) = repo.list_labor_codes().await          { *lc.write()  = data; }
        if let Ok(data) = repo.list_hour_types().await           { *ht.write()  = data; }
        if let Ok(data) = repo.list_pay_period_anchors().await   { *ppa.write() = data; }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/app.css") }
        document::Stylesheet { href: asset!("/assets/precision-dark.css") }
        Router::<Route> {}
    }
}
