use dioxus::prelude::*;
use crate::routes::Route;

#[component]
pub fn Layout() -> Element {
    rsx! {
        div { class: "min-h-screen bg-base-100 text-base-content",
            // Top navbar
            div { class: "navbar bg-base-200 shadow-sm px-4",
                div { class: "flex-1",
                    span { class: "text-xl font-bold", "Timecard Calc" }
                }
                div { class: "flex-none",
                    ul { class: "menu menu-horizontal gap-1",
                        li {
                            Link {
                                to: Route::Dashboard {},
                                class: "btn btn-ghost btn-sm",
                                "Dashboard"
                            }
                        }
                        li {
                            Link {
                                to: Route::Settings {},
                                class: "btn btn-ghost btn-sm",
                                "Settings"
                            }
                        }
                    }
                }
            }
            // Page body
            main { class: "container mx-auto p-6 max-w-5xl",
                Outlet::<Route> {}
            }
        }
    }
}
