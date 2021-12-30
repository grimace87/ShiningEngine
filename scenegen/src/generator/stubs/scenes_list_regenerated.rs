
use crate::deserialiser::scene::*;
use crate::GeneratorError;
use heck::CamelCase;

pub fn generate_scenes_list_regenerated_content(configs: &Vec<Scene>) -> Result<String, GeneratorError> {
    let mut content = String::new();
    for scene in configs.iter() {
        content = format!("{}pub mod {};\n", content, scene.id);
    }
    Ok(content)
}
