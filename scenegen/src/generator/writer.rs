use std::path::PathBuf;

use crate::generator::stubs;
use crate::deserialiser::parse_file;
use crate::GeneratorError;

pub struct WritableFile {
    pub relative_path: PathBuf,
    pub content: String
}

/// Call this function from a build script. The expected directory structure is as below.
/// The files marked with an asterisk will always be generated, while the files marked with two
/// asterisks will only be created if they do not exist, and hence can be modified safely.
///
/// <project_dir>
/// - <spec_dir_name>
///   - app.json
///   - somescene.json
///   - anotherscene.json
/// - src
///   - app.rs*
///   - scenes
///     - somescene
///       - mod.rs*
///       - details.rs**
///     - anotherscene
///       - mod.rs*
///       - details.rs**
///
/// TODO - Update Config struct and generator to reflect the above directory and data structures
pub fn process_spec_path(project_dir: &PathBuf, spec_dir_name: &'static str) -> Result<(), GeneratorError> {

    if !project_dir.is_dir() {
        return Err(GeneratorError::NotADirectory(
            format!("Not a project directory: {:?}", project_dir.as_os_str())));
    }
    let spec_dir = {
        let mut path = PathBuf::from(&project_dir);
        path.push(spec_dir_name);
        path
    };
    if !spec_dir.is_dir() {
        return Err(GeneratorError::NotADirectory(
            format!("Not a spec directory: {:?}", spec_dir.as_os_str())));
    }

    for entry in std::fs::read_dir(spec_dir).unwrap() {
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
