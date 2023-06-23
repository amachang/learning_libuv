use uv::{
    Result,
    block_on,
};

fn main() {
    if let Err(err) = try_main() {
        eprintln!("Uncaught Error: {}", err);
    };
}

fn try_main() -> Result<()> {
    block_on(async {
        println!("hello async block");
    })
}

