use rusqlite;
use std::io;
use thiserror::Error;
use ergotree_ir::serialization::SerializationError;

#[derive(Error, Debug)]
pub enum SErr {
    /// IO error
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    /// SQLite error
    #[error("SQLite error: {0}")]
    SQLite(#[from] rusqlite::Error),
    /// Serialization error for Sigma
    #[error("Parse error: {0}")]
    Parse(#[from] SerializationError)
}
