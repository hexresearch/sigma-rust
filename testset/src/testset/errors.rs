use rusqlite;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
enum SErr {
    /// IO error
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    /// SQLite error
    #[error("SQLite error: {0}")]
    SQLite(#[from] rusqlite::Error)
}