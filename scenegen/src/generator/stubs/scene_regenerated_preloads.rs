
use crate::GeneratorError;
use crate::deserialiser::scene::*;

pub fn generate_preloads(_config: &Scene) -> Result<String, GeneratorError> {
    let content = String::from("fn make_preloads(&self) -> ResourcePreloads {{

    }}");
    Ok(content)
}
