use std::path::PathBuf;

pub mod cli;
mod cses_connection;
mod database;
mod settings;

pub async fn start(path: PathBuf) {
    if let Some(mut app) = cli::Cli::new(path) {
        app.main_loop().await;
    }
}
