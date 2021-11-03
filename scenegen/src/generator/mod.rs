
pub mod writer;
mod stubs;

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
    use serial_test::serial;
    use crate::generator::writer::process_spec_path;
    use crate::GeneratorError;

    fn get_test_file(file_name: &'static str) -> PathBuf {
        let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        src_path.push("resources");
        src_path.push("test");
        src_path.push(file_name);
        src_path
    }

    #[test]
    #[serial]
    fn text_file_with_json_rejected() {
        let non_json_file = get_test_file("text_file_with_json.txt");
        let process_result = process_spec_path(&non_json_file);
        assert!(matches!(process_result, Err(GeneratorError::NotADirectoryOrJsonFile(_))));
    }

    #[test]
    #[serial]
    fn json_file_with_text_rejected() {
        let json_file_with_plain_text = get_test_file("json_file_with_text.json");
        let process_result = process_spec_path(&json_file_with_plain_text);
        assert!(matches!(process_result, Err(GeneratorError::BadJson(_, _))));
    }

    #[test]
    #[serial]
    fn invalid_spec_rejected() {
        let bad_spec_file = get_test_file("features_bad.json");
        let process_result = process_spec_path(&bad_spec_file);
        assert!(matches!(process_result, Err(GeneratorError::InvalidSchema(_, _))));
    }

    #[test]
    #[serial]
    fn valid_spec_processed() {
        let valid_spec_file = get_test_file("full_featured_app.json");
        let process_result = process_spec_path(&valid_spec_file);
        assert!(process_result.is_ok());
    }

    #[test]
    #[serial]
    fn valid_files_in_directory_processed() {
        let dir_with_valid_specs = get_test_file("valid");
        let process_result = process_spec_path(&dir_with_valid_specs);
        assert!(process_result.is_ok());
    }
}
