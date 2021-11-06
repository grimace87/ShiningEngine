
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

    fn get_test_dir() -> PathBuf {
        let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        src_path.push("resources");
        src_path.push("test");
        src_path.push("generator");
        src_path
    }

    #[test]
    #[serial]
    fn json_file_with_text_rejected() {
        let test_dir = get_test_dir();
        let process_result = process_spec_path(&test_dir, "json_file_with_text");
        assert!(matches!(process_result, Err(GeneratorError::BadJson(_, _))));
    }

    #[test]
    #[serial]
    fn invalid_spec_rejected() {
        let test_dir = get_test_dir();
        let process_result = process_spec_path(&test_dir, "features_bad");
        assert!(matches!(process_result, Err(GeneratorError::InvalidSchema(_, _))));
    }

    #[test]
    #[serial]
    fn valid_files_in_directory_processed() {
        let test_dir = get_test_dir();
        let process_result = process_spec_path(&test_dir, "full_featured_app");
        assert!(process_result.is_ok());
    }
}
