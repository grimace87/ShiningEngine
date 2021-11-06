
use jsonschema::JSONSchema;
use std::path::PathBuf;

pub fn validate_app_file(json_value: &serde_json::Value) -> Result<(), String> {
    validate_file(json_value, "app.schema.json")
}

pub fn validate_scene_file(json_value: &serde_json::Value) -> Result<(), String> {
    validate_file(json_value, "scene.schema.json")
}

fn validate_file(json_value: &serde_json::Value, schema_file: &'static str) -> Result<(), String> {
    let schema = compile_schema(schema_file);
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

fn compile_schema(schema_file: &'static str) -> JSONSchema {
    let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    src_path.push("resources");
    src_path.push(schema_file);
    let src_string = std::fs::read_to_string(src_path)
        .expect("Failed to open schema file");
    let src_json = serde_json::from_str(src_string.as_str())
        .expect("Invalid JSON schema");
    let compiled_schema = JSONSchema::compile(&src_json)
        .expect("Invalid JSON schema");
    compiled_schema
}
