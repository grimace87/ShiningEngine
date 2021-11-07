use std::path::PathBuf;

use crate::generator::{AppSpec, stubs};
use crate::GeneratorError;

pub fn write_app_files(project_dir: &PathBuf, app_spec: &AppSpec) -> Result<(), GeneratorError> {

    let app_src_file = {
        let mut path = PathBuf::from(project_dir);
        path.push("src");
        std::fs::create_dir_all(&path)
            .map_err(|_| GeneratorError::WriteError(path.clone()))?;
        path.push("app.rs");
        path
    };
    let app_stubs_file_contents = stubs::generate_app_stubs(&app_spec.app)?;
    std::fs::write(&app_src_file, app_stubs_file_contents)
        .map_err(|_| GeneratorError::WriteError(app_src_file.clone()))?;

    for scene in app_spec.scenes.iter() {
        let scene_src_file = {
            let mut path = PathBuf::from(project_dir);
            path.push("src");
            path.push("scenes");
            path.push(&scene.id);
            std::fs::create_dir_all(&path)
                .map_err(|_| GeneratorError::WriteError(path.clone()))?;
            path.push("mod.rs");
            path
        };
        let scene_stubs_file_contents = stubs::generate_scene_stubs(&scene)?;
        std::fs::write(&scene_src_file, scene_stubs_file_contents)
            .map_err(|_| GeneratorError::WriteError(scene_src_file.clone()))?;
    }

    Ok(())
}
