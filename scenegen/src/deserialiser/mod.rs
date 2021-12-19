
pub mod app;
pub mod scene;
pub mod validator;

use crate::GeneratorError;
use app::App;
use scene::*;
use validator::validate_app_file;
use validator::validate_scene_file;
use std::path::PathBuf;
use crate::deserialiser::app::{AppGraphicsApi, AppPlatform};
use crate::deserialiser::validator::validate_app_spec;
use crate::generator::AppSpec;

pub fn parse_directory(project_dir: &PathBuf, spec_dir_name: &'static str) -> Result<AppSpec, GeneratorError> {
    if !project_dir.is_dir() {
        return Err(GeneratorError::NotADirectory(
            format!("Not a project directory: {:?}", project_dir.as_os_str())));
    }
    let spec_dir = {
        let mut path = PathBuf::from(&project_dir);
        path.push(spec_dir_name);
        path
    };
    if !spec_dir.is_dir() {
        return Err(GeneratorError::NotADirectory(
            format!("Not a spec directory: {:?}", spec_dir.as_os_str())));
    }

    let mut app_spec = AppSpec {
        app: App {
            name: "unset".to_string(),
            features: vec![],
            platform: AppPlatform::windows,
            graphics: AppGraphicsApi::vulkan,
            start_scene_id: "unset".to_string()
        },
        scenes: vec![]
    };
    for entry in std::fs::read_dir(spec_dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }
        if !is_json_file(&path) {
            continue;
        }
        if path.file_name().unwrap().to_str().unwrap() == "app.json" {
            let app_config = parse_app_file(&path)?;
            app_spec.app = app_config;
        } else {
            let scene_config = parse_scene_file(&path)?;
            app_spec.scenes.push(scene_config);
        }
    }

    validate_app_spec(&app_spec)
        .map_err(|e| GeneratorError::InvalidSpec(e))?;

    Ok(app_spec)
}

fn is_json_file(file_path: &PathBuf) -> bool {
    match file_path.extension() {
        Some(e) => e.to_str().unwrap() == "json",
        None => false
    }
}

fn parse_app_file(src_file: &PathBuf) -> Result<App, GeneratorError> {
    let json_value = get_file_json(src_file)?;
    validate_app_file(&json_value)
        .map_err(|e| GeneratorError::InvalidSchema(src_file.clone(), e))?;
    serde_json::from_value::<App>(json_value)
        .map_err(|e| GeneratorError::InvalidSchema(src_file.clone(), e.to_string()))
}

fn parse_scene_file(src_file: &PathBuf) -> Result<Scene, GeneratorError> {
    let json_value = get_file_json(src_file)?;
    validate_scene_file(&json_value)
        .map_err(|e| GeneratorError::InvalidSchema(src_file.clone(), e))?;
    serde_json::from_value::<Scene>(json_value)
        .map_err(|e| GeneratorError::InvalidSchema(src_file.clone(), e.to_string()))
}

fn get_file_json(src_file: &PathBuf) -> Result<serde_json::Value, GeneratorError> {
    let src = std::fs::read_to_string(src_file)
        .map_err(|_| GeneratorError::OpenError(src_file.clone()))?;
    serde_json::from_str(src.as_str())
        .map_err(|e| GeneratorError::BadJson(src_file.clone(), e.to_string()))
}

/// Test suite
/// Needs to test that JSON specifications pass validation as per the JSON Schema, that they can
/// also be decoded to Rust structures, and that working renderers will be generated from those
/// structures.
#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use crate::deserialiser::parse_app_file;
    use crate::deserialiser::parse_scene_file;
    use crate::deserialiser::app::{App, AppFeature, AppGraphicsApi, AppPlatform};
    use crate::deserialiser::scene::*;
    use crate::GeneratorError;

    fn get_test_app_file(app_name: &'static str, file_name: &'static str) -> PathBuf {
        let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        src_path.push("resources");
        src_path.push("test");
        src_path.push("deserialiser");
        src_path.push(app_name);
        src_path.push(file_name);
        src_path
    }

    #[test]
    fn app_with_known_features_passes_validation() {
        let src_json = get_test_app_file("features_good", "app.json");
        let parse_result = parse_app_file(&src_json);
        assert!(parse_result.is_ok());
    }

    #[test]
    fn app_with_unknown_features_fails_validation() {
        let src_json = get_test_app_file("features_bad", "app.json");
        let parse_result = parse_app_file(&src_json);
        assert!(matches!(parse_result, Err(GeneratorError::InvalidSchema(_, _))));
    }

    #[test]
    fn app_with_unknown_platform_fails_validation() {
        let src_json = get_test_app_file("platform_bad", "app.json");
        let parse_result = parse_app_file(&src_json);
        assert!(matches!(parse_result, Err(GeneratorError::InvalidSchema(_, _))));
    }

    #[test]
    fn app_with_unknown_graphics_fails_validation() {
        let src_json = get_test_app_file("graphics_bad", "app.json");
        let parse_result = parse_app_file(&src_json);
        assert!(matches!(parse_result, Err(GeneratorError::InvalidSchema(_, _))));
    }

    #[test]
    fn valid_full_featured_app_passes_validation() {
        let src_json = get_test_app_file("full_featured_app", "app.json");
        let parse_result = parse_app_file(&src_json);
        assert!(parse_result.is_ok());
    }

    #[test]
    fn valid_full_featured_app_can_deserialise() {
        let src_json = get_test_app_file("full_featured_app", "app.json");
        let parse_result = parse_app_file(&src_json);
        assert!(parse_result.is_ok());

        let src_json = get_test_app_file("full_featured_app", "scene.json");
        let parse_result = parse_scene_file(&src_json);
        assert!(parse_result.is_ok());

        let src_json = get_test_app_file("full_featured_app", "cutscene.json");
        let parse_result = parse_scene_file(&src_json);
        assert!(parse_result.is_ok());
    }

    #[test]
    fn deserialised_full_featured_app_matches_expectations() {

        let app_src = get_test_app_file("full_featured_app", "app.json");
        let app_object = parse_app_file(&app_src).unwrap();
        let expected_app = App {
            name: "Full-featured example which should pass validation".to_string(),
            features: vec![
                AppFeature::clip_planes
            ],
            platform: AppPlatform::windows,
            graphics: AppGraphicsApi::vulkan,
            start_scene_id: "scene".to_string()
        };
        assert_eq!(format!("{:?}", app_object), format!("{:?}", expected_app));

        let scene1_src = get_test_app_file("full_featured_app", "scene.json");
        let scene1_object = parse_scene_file(&scene1_src).unwrap();
        let expected_scene1 = Scene {
            id: "scene".to_string(),
            camera: Camera::player,
            resources: Resources {
                models: vec![
                    Model {
                        id: "terrain".to_string(),
                        file: Some("SceneTerrain.mdl".to_string()),
                        generator: None
                    },
                    Model {
                        id: "river".to_string(),
                        file: Some("River.mdl".to_string()),
                        generator: None
                    },
                    Model {
                        id: "skybox".to_string(),
                        file: None,
                        generator: Some(ModelGenerator::skybox)
                    },
                    Model {
                        id: "hud".to_string(),
                        file: None,
                        generator: Some(ModelGenerator::text)
                    }
                ],
                textures: vec![
                    Texture {
                        id: "outdoors".to_string(),
                        format: TextureFormat::rgba8,
                        file: Some("simple_outdoor_texture.jpg".to_string()),
                        kind: None
                    },
                    Texture {
                        id: "musica".to_string(),
                        format: TextureFormat::rgba8,
                        file: Some("Musica.png".to_string()),
                        kind: None
                    },
                    Texture {
                        id: "skybox".to_string(),
                        format: TextureFormat::rgb8,
                        file: Some("bluecloud.jpg".to_string()),
                        kind: Some(TextureKind::cubemap)
                    },
                    Texture {
                        id: "reflection_colour".to_string(),
                        format: TextureFormat::rgb8,
                        file: None,
                        kind: Some(TextureKind::uninitialised)
                    },
                    Texture {
                        id: "reflection_depth".to_string(),
                        format: TextureFormat::d16,
                        file: None,
                        kind: Some(TextureKind::uninitialised)
                    }
                ],
                fonts: vec![
                    Font {
                        id: "musica".to_string(),
                        file: "Musica.fnt".to_string(),
                        texture_id: "musica".to_string()
                    }
                ]
            },
            passes: vec![
                Pass {
                    name: "pre_reflection".to_string(),
                    kind: PassKind::offscreen,
                    target_texture_ids: Some(TextureTarget {
                        colour_texture_id: "reflection_colour".to_string(),
                        depth_texture_id: Some("reflection_depth".to_string())
                    }),
                    render: RenderFunction::reflection_pre_render,
                    steps: vec![
                        Step {
                            name: "skybox".to_string(),
                            model_id: "skybox".to_string(),
                            texture_ids: vec!["skybox".to_string()]
                        },
                        Step {
                            name: "terrain".to_string(),
                            model_id: "terrain".to_string(),
                            texture_ids: vec!["outdoors".to_string()]
                        }
                    ]
                },
                Pass {
                    name: "compose".to_string(),
                    kind: PassKind::default,
                    target_texture_ids: None,
                    render: RenderFunction::basic_textured,
                    steps: vec![
                        Step {
                            name: "skybox".to_string(),
                            model_id: "skybox".to_string(),
                            texture_ids: vec!["skybox".to_string()]
                        },
                        Step {
                            name: "river".to_string(),
                            model_id: "river".to_string(),
                            texture_ids: vec!["reflection".to_string()]
                        },
                        Step {
                            name: "terrain".to_string(),
                            model_id: "terrain".to_string(),
                            texture_ids: vec!["outdoors".to_string()]
                        }
                    ]
                },
                Pass {
                    name: "hud".to_string(),
                    kind: PassKind::default,
                    target_texture_ids: None,
                    render: RenderFunction::text_paint,
                    steps: vec![
                        Step {
                            name: "text_overlay".to_string(),
                            model_id: "hud".to_string(),
                            texture_ids: vec!["musica".to_string()]
                        }
                    ]
                }
            ]
        };
        assert_eq!(format!("{:?}", scene1_object), format!("{:?}", expected_scene1));

        let scene2_src = get_test_app_file("full_featured_app", "cutscene.json");
        let scene2_object = parse_scene_file(&scene2_src).unwrap();
        let expected_scene2 = Scene {
            id: "cutscene".to_string(),
            camera: Camera::flight_path,
            resources: Resources {
                models: vec![
                    Model {
                        id: "skybox".to_string(),
                        file: None,
                        generator: Some(ModelGenerator::skybox)
                    }
                ],
                textures: vec![
                    Texture {
                        id: "skybox".to_string(),
                        format: TextureFormat::rgb8,
                        file: Some("bluecloud.jpg".to_string()),
                        kind: Some(TextureKind::cubemap)
                    }
                ],
                fonts: vec![]
            },
            passes: vec![
                Pass {
                    name: "skybox".to_string(),
                    kind: PassKind::default,
                    target_texture_ids: None,
                    render: RenderFunction::basic_textured,
                    steps: vec![
                        Step {
                            name: "box".to_string(),
                            model_id: "skybox".to_string(),
                            texture_ids: vec!["skybox".to_string()]
                        }
                    ]
                }
            ]
        };
        assert_eq!(format!("{:?}", scene2_object), format!("{:?}", expected_scene2));
    }
}
