use log::{debug, info, warn};

use crate::{cli::Cli, errors::AppResult, git::GitOps, logging::AppLogger, settings::Settings};

mod cli;
mod errors;
mod git;
mod logging;
mod settings;

fn main() -> AppResult<()> {
    let cli = Cli::open();
    AppLogger::<termcolor::StandardStream>::init(&cli);

    info!("changelogger starting ...");
    debug!("setup configuration");
    let settings = Settings::new(&cli)?;

    debug!(
        "read version info from branch {}",
        settings.default_branch(),
    );
    let git = GitOps::new(settings.default_branch());
    let ver = git.cat_file(settings.version_file())?.trim().to_string();
    info!("current version is {ver}");

    debug!("search for tag matching {ver}");
    let _tags = git.tags()?;

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
