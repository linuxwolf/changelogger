use clap::{Args, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use std::ffi::OsString;

#[derive(Debug, Default, Parser, PartialEq, Eq)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub configuration: Configuration,
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,
}

impl Cli {
    pub fn open_with<I, T>(itr: I) -> Cli
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Cli::parse_from(itr)
    }

    pub fn open() -> Cli {
        Cli::open_with(std::env::args())
    }
}

#[derive(Args, Debug, Default, PartialEq, Eq)]
pub struct Configuration {
    #[arg(long = "config")]
    /// Path to explicit configuration file to use
    pub config_file: Option<String>,

    #[arg(long = "version-file")]
    /// Version file to use
    pub version_file: Option<String>,

    #[arg(long = "version-prefix")]
    /// Version prefix to use when finding tags or possibly parsing version files
    pub version_prefix: Option<String>,

    #[arg(long = "changelog-file")]
    /// Changelog file to use
    pub changelog_file: Option<String>,

    #[arg(long = "default-branch")]
    /// Default branch to use with `git`
    pub default_branch: Option<String>,
}

#[cfg(test)]
mod testing {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn validate_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn open_with_no_args() {
        let args: Vec<String> = Vec::new();
        let result = Cli::open_with(args);

        assert_eq!(
            result,
            Cli {
                verbosity: Verbosity::default(),
                configuration: Configuration::default(),
            }
        );
    }

    #[test]
    fn open_default() {
        let result = Cli::open();

        assert_eq!(
            result,
            Cli {
                verbosity: Verbosity::default(),
                configuration: Configuration::default(),
            }
        );
    }
}
