fn main() {
    let db_path = dirs::data_dir()
        .expect("Cannot locate app data directory")
        .join("timecard-calc")
        .join("timecard.db");

    api::db::init(db_path);

    dioxus::launch(ui::App);
}
