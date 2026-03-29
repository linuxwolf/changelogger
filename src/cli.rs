use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

use std::ffi::OsString;

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(version, about, long_about = None)]
pub struct Cli {
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
            }
        );
    }
}
