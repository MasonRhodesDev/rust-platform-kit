use std::time::Duration;

use rust_platform_kit::permissions::{self, Permission};
use rust_platform_kit::presence;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match presence::current(Duration::from_secs(60)).await {
        Ok(p) => println!("presence: {p:?}"),
        Err(e) => println!("presence: error: {e}"),
    }
    for p in Permission::ALL {
        println!("{p:?}: {:?}", permissions::access(p));
    }
}
