use log::{debug, error, info, warn};

use crate::{cli::Cli, errors::AppResult, logging::AppLogger};

mod cli;
mod errors;
mod logging;
mod settings;

fn main() -> AppResult<()> {
    let cli = Cli::open();
    AppLogger::<termcolor::StandardStream>::init(&cli);

    info!("changelogger starting ...");
    debug!("setup configuration");
    debug!(
        "read version from <default-branch>: `git cat-file --textconv <default-branch>:<version-file>"
    );
    error!("fails if <version-file> on <default-branch> does not exist!");
    debug!("find matching tag: `git tag | grep <version-prefix><current-version>`");
    debug!("find all commits from <current-version-tag> to <default-branch>");
    debug!("process commits");
    debug!(
        "read current changelog from <default-branch>: `git cat-file --textconv <default-branch>:<changelog-file>"
    );
    warn!("initialize with a template if <changelog-file> on <default-branch> does not exist!");
    debug!("write to <changelog-file>");
    debug!("write to <version-file>");

    Ok(())
}
