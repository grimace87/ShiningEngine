
mod deserialiser;

use std::path::PathBuf;

#[derive(Debug)]
pub enum GeneratorError {
    OpenError(PathBuf),
    NotADirectoryOrJsonFile(String),
    BadJson(PathBuf, String),
    InvalidSchema(PathBuf, String),
}
