use std::path::PathBuf;

mod cses_connection;
mod database;
mod settings;
pub mod tui;

#[tokio::main]
pub async fn start(path: PathBuf) {
    if let Some(mut app) = tui::Tui::new(path) {
        app.main_loop().await;
    }
}
