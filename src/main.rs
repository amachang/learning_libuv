use uv::{
    Error,
    Result,
    block_on,
    time,
};

use std::time::Duration;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("Uncaught Error: {}", err);
    };
}

fn try_main() -> Result<()> {
    block_on(async {
        println!("hello async block 1");
        time::sleep(Duration::from_secs(1))?.await?;
        println!("hello async block 2");
        Ok::<(), Error>(())
    })??;
    block_on(async {
        println!("hello async block 3");
        time::sleep(Duration::from_secs(1))?.await?;
        println!("hello async block 4");
        Ok::<(), Error>(())
    })??;
    Ok(())
}

