
use crate::GeneratorError;
use crate::deserialiser::scene::*;

pub fn generate_description(_config: &Scene) -> Result<String, GeneratorError> {
    let content = String::from("fn make_description(&self) -> DrawingDescription {{

    }}");
    Ok(content)
}
