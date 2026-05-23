use dioxus::prelude::*;
use crate::routes::Route;

#[component]
pub fn Layout() -> Element {
    let route = use_route::<Route>();
    let on_dash = matches!(route, Route::Dashboard {});
    let on_set  = matches!(route, Route::Settings {});

    rsx! {
        div { class: "flex h-screen bg-[#0d1117] text-[#e6edf3] overflow-hidden",

            // ── Sidebar ──
            div { class: "w-14 bg-[#161b22] border-r border-[#21262d] flex flex-col items-center py-3 gap-1 flex-shrink-0",

                // Logo mark
                div { class: "w-7 h-7 bg-[#58a6ff] rounded-[6px] flex items-center justify-center mb-3 font-black text-white text-sm select-none",
                    "T"
                }

                // Dashboard nav item
                Link {
                    to: Route::Dashboard {},
                    class: if on_dash { "pd-nav-active w-11 rounded-[6px] py-1.5 flex flex-col items-center gap-0.5 no-underline" }
                           else       { "pd-nav-inactive w-11 rounded-[6px] py-1.5 flex flex-col items-center gap-0.5 no-underline hover:bg-[#21262d]" },
                    span { class: "pd-nav-icon text-base leading-none", "⊞" }
                    span { class: "pd-nav-label text-[9px] font-semibold uppercase tracking-wider", "Dash" }
                }

                // Settings nav item
                Link {
                    to: Route::Settings {},
                    class: if on_set { "pd-nav-active w-11 rounded-[6px] py-1.5 flex flex-col items-center gap-0.5 no-underline" }
                           else      { "pd-nav-inactive w-11 rounded-[6px] py-1.5 flex flex-col items-center gap-0.5 no-underline hover:bg-[#21262d]" },
                    span { class: "pd-nav-icon text-base leading-none", "⚙" }
                    span { class: "pd-nav-label text-[9px] font-semibold uppercase tracking-wider", "Set" }
                }
            }

            // ── Main content ──
            main { class: "flex-1 overflow-auto",
                Outlet::<Route> {}
            }
        }
    }
}
