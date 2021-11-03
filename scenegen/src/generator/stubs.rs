use crate::deserialiser::structures::Config;
use crate::GeneratorError;

/// Generate stubs for a scene as a String.
/// Content should typically be written to a file.
pub fn generate_stub(config: &Config) -> Result<String, GeneratorError> {
    Ok("Hello".to_string())
}
