use dioxus::prelude::*;
use api::TimecardEntryView;

#[component]
pub fn EntryFormModal(
    show: bool,
    editing: Option<TimecardEntryView>,
    date: String,
    on_close: EventHandler,
    on_saved: EventHandler,
) -> Element {
    rsx! { div {} }
}
