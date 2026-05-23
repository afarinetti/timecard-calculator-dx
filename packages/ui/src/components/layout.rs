use dioxus::prelude::*;
use crate::routes::Route;

#[component]
pub fn Layout() -> Element {
    rsx! {
        div { class: "min-h-screen bg-base-100",
            Outlet::<Route> {}
        }
    }
}
