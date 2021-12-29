
pub mod writer;
mod stubs;

use std::path::PathBuf;
use crate::deserialiser::app::App;
use crate::deserialiser::scene::Scene;
use crate::deserialiser::parse_directory;
use crate::generator::writer::write_app_files;
use crate::GeneratorError;

pub struct AppSpec {
    pub app: App,
    pub scenes: Vec<Scene>
}

/// Call this function from a build script. The expected directory structure is as below.
/// The files marked with an asterisk will always be generated, while the files marked with two
/// asterisks will only be created if they do not exist, and hence can be modified safely.
///
/// <project_dir>
/// - <spec_dir_name>
///   - app.json
///   - some_scene.json
///   - another_scene.json
/// - <resources_dir_name>
///   - textures
///     - some_image.jpg
///     - another_image.png
/// - src
///   - app.rs*
///   - scenes
///     - some_scene
///       - mod.rs*
///       - details.rs**
///     - another_scene
///       - mod.rs*
///       - details.rs**
pub fn process_spec_path(
    project_dir: &PathBuf,
    spec_dir_name: &'static str,
    resources_dir_name: &'static str
) -> Result<(), GeneratorError> {
    let app_spec = parse_directory(project_dir, spec_dir_name)?;
    write_app_files(project_dir, &app_spec, resources_dir_name)?;
    Ok(())
}

/// Test suite
/// Test that processing an individual file succeeds if, and only if, it is a JSON file, while
/// processing directories performs work on JSON files while ignoring others.
///
/// Note that if these tests run in parallel (which is the default), they race at creating output
/// directories which can cause hard-to-explain test failures, but only if the output directories
/// do not yet exist when the tests are run.
#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use crate::generator::process_spec_path;
    use crate::GeneratorError;

    fn get_test_dir(test_dir: &'static str) -> PathBuf {
        let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        src_path.push("resources");
        src_path.push("test");
        src_path.push("generator");
        src_path.push(test_dir);
        src_path
    }

    #[test]
    fn json_file_with_text_rejected() {
        let test_dir = get_test_dir("json_file_with_text");
        let process_result = process_spec_path(&test_dir, "spec");
        assert!(matches!(process_result, Err(GeneratorError::BadJson(_, _))));
    }

    #[test]
    fn invalid_spec_rejected() {
        let test_dir = get_test_dir("features_bad");
        let process_result = process_spec_path(&test_dir, "spec");
        assert!(matches!(process_result, Err(GeneratorError::InvalidSchema(_, _))));
    }

    #[test]
    fn missing_initial_scene_fails_validation() {
        let test_dir = get_test_dir("missing_initial_scene");
        let process_result = process_spec_path(&test_dir, "spec");
        assert!(matches!(process_result, Err(GeneratorError::InvalidSpec(_))));
    }

    #[test]
    fn bad_generated_texture_format_fails_validation() {
        let test_dir = get_test_dir("bad_generator_format");
        let process_result = process_spec_path(&test_dir, "spec");
        assert!(matches!(process_result, Err(GeneratorError::InvalidSpec(_))));
    }

    #[test]
    fn valid_files_in_directory_processed() {
        let test_dir = get_test_dir("full_featured_app");
        let process_result = process_spec_path(&test_dir, "spec");
        assert!(process_result.is_ok());
    }
}
