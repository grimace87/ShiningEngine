
use crate::GeneratorError;
use crate::deserialiser::scene::*;

pub fn generate_starter_scene_contents(_config: &Scene) -> Result<String, GeneratorError> {
    let content = "Hello!".to_string();
    Ok(content)
}
