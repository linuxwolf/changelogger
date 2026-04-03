use anyhow::Result;
use log::{debug, info, warn};

use crate::{
    app::{App, AppOps},
    cli::Cli,
    git::GitOps,
    logging::AppLogger,
    settings::Settings,
};

mod app;
mod cli;
mod git;
mod logging;
mod settings;

fn main() -> Result<()> {
    let cli = Cli::open();
    AppLogger::<termcolor::StandardStream>::init(&cli);

    info!("changelogger starting ...");
    debug!("setup configuration");
    let settings = Settings::new(&cli)?;

    let app = AppOps::<GitOps>::new(settings);

    let version = app.get_version()?;
    let tag = app.get_version_tag(&version)?;

    let _commits = app.list_commits(tag)?;
    debug!("process commits");
    debug!(
        "read current changelog from <default-branch>: `git cat-file --textconv <default-branch>:<changelog-file>"
    );
    warn!("initialize with a template if <changelog-file> on <default-branch> does not exist!");
    debug!("write to <changelog-file>");
    debug!("write to <version-file>");

    Ok(())
}
