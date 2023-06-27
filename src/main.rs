use uv::{
    block_on,
    time,
};

use std::time::Duration;

fn main() {
    block_on(async {
        println!("hello async block 1");
        time::sleep(Duration::from_secs(1)).await;
        println!("hello async block 2");
    });
    block_on(async {
        println!("hello async block 3");
        time::sleep(Duration::from_secs(1)).await;
        println!("hello async block 4");
    });
}

