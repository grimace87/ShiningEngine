
mod deserialiser;
pub mod generator;

use std::path::PathBuf;

#[derive(Debug)]
pub enum GeneratorError {
    OpenError(PathBuf),
    NotADirectory(String),
    BadJson(PathBuf, String),
    InvalidSchema(PathBuf, String),
    WriteError(PathBuf),
    InvalidSpec(String)
}
