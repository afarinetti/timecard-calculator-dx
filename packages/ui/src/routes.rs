use dioxus::prelude::*;
use crate::pages::{dashboard::Dashboard, settings::Settings};
use crate::components::layout::Layout;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(Layout)]
        #[route("/")]
        Dashboard {},
        #[route("/settings")]
        Settings {},
}
