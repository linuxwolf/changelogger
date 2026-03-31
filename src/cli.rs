use clap::{Args, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use std::ffi::OsString;

#[derive(Debug, Default, Parser, PartialEq, Eq)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub configuring: Configuring,
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
pub struct Configuring {
    #[arg(long = "config")]
    /// Path to explicit configuration file to use
    pub config_file: Option<String>,
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
                configuring: Configuring::default(),
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
                configuring: Configuring::default(),
            }
        );
    }
}
