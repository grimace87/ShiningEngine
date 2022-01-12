use std::path::PathBuf;

use crate::generator::{CompleteSpec, stubs};
use crate::GeneratorError;

pub fn write_app_files(
    project_dir: &PathBuf,
    complete_spec: &CompleteSpec,
    resources_dir_name: &'static str
) -> Result<(), GeneratorError> {

    // App files - src/app/mod.rs and src/app/struct.gen.rs
    let (app_module_file_contents, app_regenerated_file_contents) =
        stubs::generate_app_stubs(&complete_spec.app)?;
    let app_module_file = make_project_file(
        project_dir,
        vec!["src", "app"],
        "mod.rs")?;
    if !app_module_file.is_file() {
        std::fs::write(&app_module_file, app_module_file_contents)
            .map_err(|_| GeneratorError::WriteError(app_module_file.clone()))?;
    }
    let app_regenerated_file = make_project_file(
        project_dir,
        vec!["src", "app"],
        "struct.gen.rs")?;
    std::fs::write(&app_regenerated_file, app_regenerated_file_contents)
        .map_err(|_| GeneratorError::WriteError(app_regenerated_file.clone()))?;

    // Scenes module - src/scenes/mod.rs and src/scenes/list.gen.rs
    let (scenes_list_module_file_contents, scenes_list_regenerated_file_contents) =
        stubs::generate_scene_listing_stubs(&complete_spec.scenes)?;
    let scenes_list_module_file = make_project_file(
        project_dir,
        vec!["src", "scenes"],
        "mod.rs")?;
    if !scenes_list_module_file.is_file() {
        std::fs::write(&scenes_list_module_file, scenes_list_module_file_contents)
            .map_err(|_| GeneratorError::WriteError(scenes_list_module_file.clone()))?;
    }
    let scenes_list_regenerated_file = make_project_file(
        project_dir,
        vec!["src", "scenes"],
        "list.gen.rs")?;
    std::fs::write(&scenes_list_regenerated_file, scenes_list_regenerated_file_contents)
        .map_err(|_| GeneratorError::WriteError(scenes_list_regenerated_file.clone()))?;

    for (scene_index, scene) in complete_spec.scenes.iter().enumerate() {
        let (scene_module_file_contents, scene_core_file_contents) =
            stubs::generate_scene_stubs(scene_index + 1, &complete_spec.app.shared_resources, &scene, resources_dir_name)?;

        let scene_module_file = make_project_file(
            project_dir,
            vec!["src", "scenes", scene.id.as_str()],
            "mod.rs")?;
        if !scene_module_file.is_file() {
            std::fs::write(&scene_module_file, scene_module_file_contents)
                .map_err(|_| GeneratorError::WriteError(scene_module_file.clone()))?;
        }

        let scene_core_file = make_project_file(
            project_dir,
            vec!["src", "scenes", scene.id.as_str()],
            "descriptions.gen.rs")?;
        std::fs::write(&scene_core_file, scene_core_file_contents)
            .map_err(|_| GeneratorError::WriteError(scene_core_file.clone()))?;
    }

    Ok(())
}

fn make_project_file(
    project_dir: &PathBuf,
    subdirectories: Vec<&str>,
    file_name: &str
) -> Result<PathBuf, GeneratorError> {
    let mut path = PathBuf::from(project_dir);
    for subdirectory in subdirectories {
        path.push(subdirectory);
    }
    std::fs::create_dir_all(&path)
        .map_err(|_| GeneratorError::WriteError(path.clone()))?;
    path.push(file_name);
    Ok(path)
}
