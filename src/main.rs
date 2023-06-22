use uv::{
    Result,
    Loop,
    RunMode,
};

fn main() {
    if let Err(err) = try_main() {
        eprintln!("Uncaught Error: {}", err);
    };
}

fn try_main() -> Result<()> {
    let lp = Loop::try_new()?;
    lp.run(RunMode::Default)
}

