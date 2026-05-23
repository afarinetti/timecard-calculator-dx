use once_cell::sync::OnceCell;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;

pub mod models;
pub mod repo;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

static POOL: OnceCell<SqlitePool> = OnceCell::new();

/// Returns the global pool. Panics if `init()` has not been called.
pub fn pool() -> &'static SqlitePool {
    POOL.get().expect("DB pool not initialized — call api::db::init() first")
}

/// Synchronously initialize the pool and run pending migrations.
/// Must be called once from `main()` before `dioxus::launch`.
pub fn init(db_path: PathBuf) {
    std::fs::create_dir_all(db_path.parent().unwrap())
        .expect("Failed to create app data directory");

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        let opts = SqliteConnectOptions::from_str(
            &format!("sqlite://{}", db_path.display()),
        )
        .expect("Invalid DB path")
        .create_if_missing(true)
        .pragma("foreign_keys", "ON")
        .pragma("journal_mode", "WAL");

        let pool = SqlitePool::connect_with(opts)
            .await
            .expect("Failed to connect to SQLite");

        MIGRATOR.run(&pool).await.expect("Failed to run migrations");

        POOL.set(pool).ok();
    });
}
