
use jsonschema::JSONSchema;
use std::path::PathBuf;

pub fn validate_file(json_value: &serde_json::Value) -> Result<(), String> {
    let schema = compile_schema();
    schema.validate(&json_value)
        .map_err(|error_iter| {
            let mut error_builder = String::new();
            for error in error_iter {
                error_builder.push_str(format!("{:?} ", error.kind).as_str());
            }
            error_builder
        })?;
    Ok(())
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
