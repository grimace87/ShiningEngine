
#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use jsonschema::JSONSchema;

    fn get_test_file(file_name: &'static str) -> serde_json::Value {
        let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        src_path.push("resources");
        src_path.push("test");
        src_path.push(file_name);
        let src = std::fs::read_to_string(src_path)
            .expect(format!("Failed to open {}", file_name).as_str());
        serde_json::from_str(src.as_str())
            .expect("Failed to parse test JSON")
    }

    fn compile_schema() -> JSONSchema {
        let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        src_path.push("resources");
        src_path.push("scenegen.schema.json");
        let src_string = std::fs::read_to_string(src_path)
            .expect("Failed to open schema file");
        let src_json = serde_json::from_str(src_string.as_str())
            .expect("Invalid JSON schema");
        let compiled_schema = JSONSchema::compile(&src_json)
            .expect("Invalid JSON schema");
        compiled_schema
    }

    #[test]
    fn app_with_known_features_is_accepted() {
        let src_json = get_test_file("features_good.json");
        let schema = compile_schema();
        let validation_result = schema.validate(&src_json);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn app_with_unknown_features_is_rejected() {
        let src_json = get_test_file("features_bad.json");
        let schema = compile_schema();
        let validation_result = schema.validate(&src_json);
        assert!(validation_result.is_err());
    }
}
