use cses_automation::start;
use std::path::PathBuf;


#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let path = PathBuf::from(".");
    start(path).await;
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        tokio::signal::ctrl_c().await.ok();
    });
}
