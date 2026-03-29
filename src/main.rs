use crate::{cli::Cli, errors::AppResult, logging::init_logging};

mod cli;
mod errors;
mod logging;

fn main() -> AppResult<()> {
    init_logging();
    let _cli = Cli::open();

    Ok(())
}
