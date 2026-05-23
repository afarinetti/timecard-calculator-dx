pub mod utils;

use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! { div { "Timecard Calc — loading..." } }
}
