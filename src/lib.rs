use std::path::PathBuf;

mod cses_connection;
mod database;
mod settings;
pub mod cli;

pub async fn start(path: PathBuf) {
    if let Some(mut app) = cli::Cli::new(path) {
        app.main_loop().await;
    }
}
