use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("problem with command-line: {0}")]
    CliError(#[from] clap::Error),
}

pub type AppResult<T> = std::result::Result<T, AppError>;
