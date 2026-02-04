use cses::start;
use std::path::PathBuf;


#[tokio::main]
async fn main() {
    let path = PathBuf::from(".");
    start(path).await;
}
