use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    CliError(#[from] clap::Error),
    #[error("problem configuring: {0}")]
    ConfigError(#[from] config::ConfigError),
    #[error("I/O error {0:#}")]
    Io(#[from] io::Error),
    #[error("command {command} exited with code {code}")]
    CmdFailed { command: String, code: i32 },
}

pub type AppResult<T> = std::result::Result<T, AppError>;
