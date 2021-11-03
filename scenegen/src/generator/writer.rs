use std::path::PathBuf;

use crate::generator::stubs;
use crate::deserialiser::parse_file;
use crate::GeneratorError;

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
    let output_file = {
        let mut out = PathBuf::from(file_path);
        out.pop();
        out.push("out");
        if !out.is_dir() {
            std::fs::create_dir(&out)
                .map_err(|_| GeneratorError::WriteError(out.clone()))?;
        }
        out.push(file_path.file_name().unwrap());
        out.set_extension("rs");
        out
    };
    let config = parse_file(file_path)?;
    let stub_content = stubs::generate_stub(&config)?;
    std::fs::write(&output_file, stub_content)
        .map_err(|_| GeneratorError::WriteError(output_file.clone()))?;
    Ok(())
}
