use dioxus::prelude::*;
use crate::routes::Route;

#[component]
pub fn Layout() -> Element {
    let route = use_route::<Route>();
    let on_dash = matches!(route, Route::Dashboard {});
    let on_set  = matches!(route, Route::Settings {});

    rsx! {
        div { class: "flex flex-col h-screen bg-[#0d1117] text-[#e6edf3] overflow-hidden",

            // ── Top bar ──
            div { class: "h-11 bg-[#0d1117] border-b border-[#21262d] flex items-center px-4 gap-1 flex-shrink-0",

                // App title
                div { class: "flex items-center gap-1.5 mr-3 select-none",
                    span { class: "text-sm font-semibold text-[#e6edf3]", "Timecard Calculator" }
                    span { class: "text-[10px] font-medium text-[#6e7681]", "v{env!(\"CARGO_PKG_VERSION\")}" }
                }

                // Dashboard nav item
                Link {
                    to: Route::Dashboard {},
                    class: if on_dash {
                        "text-sm font-medium text-[#58a6ff] px-4 py-2 border-b-2 border-[#58a6ff] -mb-px transition-colors no-underline h-11 flex items-center"
                    } else {
                        "text-sm font-medium text-[#8b949e] hover:text-[#e6edf3] px-4 py-2 border-b-2 border-transparent -mb-px transition-colors no-underline h-11 flex items-center"
                    },
                    "Dashboard"
                }

                // Settings nav item
                Link {
                    to: Route::Settings {},
                    class: if on_set {
                        "text-sm font-medium text-[#58a6ff] px-4 py-2 border-b-2 border-[#58a6ff] -mb-px transition-colors no-underline h-11 flex items-center"
                    } else {
                        "text-sm font-medium text-[#8b949e] hover:text-[#e6edf3] px-4 py-2 border-b-2 border-transparent -mb-px transition-colors no-underline h-11 flex items-center"
                    },
                    "Settings"
                }
            }

            // ── Main content ──
            main { class: "flex-1 overflow-hidden flex flex-col",
                Outlet::<Route> {}
            }
        }
    }
}
