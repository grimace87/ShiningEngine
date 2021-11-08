
use jsonschema::JSONSchema;
use std::path::PathBuf;
use crate::generator::AppSpec;

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

/// Verify that references are all valid
pub fn validate_app_spec(spec: &AppSpec) -> Result<(), String> {

    let initial_scene_id = &spec.app.start_scene_id;
    if let None = spec.scenes.iter().find(|scene| &scene.id == initial_scene_id) {
        return Err(format!("Initial scene doesn't exist: {}", initial_scene_id));
    }

    for scene in spec.scenes.iter() {
        for font in scene.resources.fonts.iter() {
            let texture_id = &font.texture_id;
            if let None = scene.resources.textures.iter().find(|texture| &texture.id == texture_id) {
                return Err(format!("(Scene {}) Font texture doesn't exist: {}", scene.id, texture_id));
            }
        }

        for pass in scene.passes.iter() {
            if let Some(target_texture_id) = &pass.target_texture_id {
                if let None = scene.resources.textures.iter().find(|texture| &texture.id == target_texture_id) {
                    return Err(format!("(Scene {}) Texture target doesn't exist: {}", scene.id, target_texture_id));
                }
            }

            for step in pass.steps.iter() {
                let model_id = &step.model_id;
                if let None = scene.resources.models.iter().find(|model| &model.id == model_id) {
                    return Err(format!("(Scene {}) Model doesn't exist: {}", scene.id, model_id));
                }

                for texture_id in step.texture_ids.iter() {
                    if let None = scene.resources.textures.iter().find(|texture| &texture.id == texture_id) {
                        return Err(format!("(Scene {}) Step texture doesn't exist: {}", scene.id, texture_id));
                    }
                }
            }
        }
    }

    Ok(())
}
