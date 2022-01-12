
use crate::GeneratorError;
use crate::deserialiser::types::{Resources, scene::*};
use crate::generator::stubs::scene_regenerated_top_level::generate_top_level;
use crate::generator::stubs::scene_regenerated_struct::{generate_struct_definition, generate_get_ubo_fn};
use crate::generator::stubs::scene_regenerated_preloads::generate_preloads;
use crate::generator::stubs::scene_regenerated_description::generate_description;
use heck::CamelCase;

pub fn generate_regenerated_scene_contents(
    scene_number_one_based: usize,
    shared_resources: &Resources,
    config: &Scene,
    resources_dir_name: &'static str
) -> Result<String, GeneratorError> {

    let top_level_content = generate_top_level(scene_number_one_based, config, resources_dir_name)?;
    let struct_definition = generate_struct_definition(config, resources_dir_name)?;
    let preloads = generate_preloads(config)?;
    let description = generate_description(shared_resources, config)?;
    let get_ubo_fn = generate_get_ubo_fn(config)?;

    let struct_name = format!("{}Scene", config.id.to_camel_case());

    let gen_content = format!("
{}
{}

impl SceneInfo for {} {{

    {}
    {}
    {}
}}
",
        top_level_content,
        struct_definition,
        struct_name,
        preloads,
        description,
        get_ubo_fn
    );

    Ok(gen_content)
}
