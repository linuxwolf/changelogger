use crate::{cli::Cli, errors::AppResult};

mod cli;
mod errors;

fn main() -> AppResult<()> {
    let _cli = Cli::open();

    Ok(())
}
