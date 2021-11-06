use std::path::PathBuf;

use crate::generator::stubs;
use crate::deserialiser::parse_file;
use crate::GeneratorError;

pub struct WritableFile {
    pub relative_path: PathBuf,
    pub content: String
}

/// Call this function from a build script. The expected directory structure is as below:
/// TODO
pub fn process_spec_path(src_path: &PathBuf) -> Result<(), GeneratorError> {

    if src_path.is_file() {
        if !is_json_file(src_path) {
            return Err(GeneratorError::NotADirectoryOrJsonFile(
                format!("Not a JSON file: {:?}", src_path.as_os_str())));
        }
        return process_file(src_path);
    }

    if !src_path.is_dir() {
        return Err(GeneratorError::NotADirectoryOrJsonFile(
            format!("Not a file or directory: {:?}", src_path.as_os_str())));
    }
    for entry in std::fs::read_dir(src_path).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }
        if !is_json_file(&path) {
            continue;
        }
        process_file(&path)?;
    }
    Ok(())
}

fn is_json_file(file_path: &PathBuf) -> bool {
    match file_path.extension() {
        Some(e) => e.to_str().unwrap() == "json",
        None => false
    }
}

fn process_file(file_path: &PathBuf) -> Result<(), GeneratorError> {
    let output_path = {
        let mut out = PathBuf::from(file_path);
        out.pop();
        out.push("out");
        out
    };
    let config = parse_file(file_path)?;
    let stub_files = stubs::generate_stubs(file_path, &config)?;
    for file in stub_files {
        let mut output_file = PathBuf::from(&output_path);
        output_file.push(&file.relative_path);
        output_file.pop();
        std::fs::create_dir_all(&output_file)
            .map_err(|_| GeneratorError::WriteError(output_file.clone()))?;
        output_file.push((&file.relative_path).file_name().unwrap());
        std::fs::write(&output_file, file.content)
            .map_err(|_| GeneratorError::WriteError(output_file.clone()))?;
    }
    Ok(())
}
