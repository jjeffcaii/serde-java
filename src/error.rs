use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] io::Error),

    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
