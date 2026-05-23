use dioxus::prelude::*;
use api::TimecardEntryView;

#[component]
pub fn EntryTable(
    entries: Vec<TimecardEntryView>,
    on_edit: EventHandler<TimecardEntryView>,
    on_delete: EventHandler<i64>,
) -> Element {
    rsx! { div { "Entry Table" } }
}
