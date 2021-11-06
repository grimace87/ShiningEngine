
pub mod structures;
pub mod validator;

use crate::GeneratorError;
use structures::*;
use validator::validate_app_file;
use validator::validate_scene_file;
use std::path::PathBuf;

pub fn parse_app_file(src_file: &PathBuf) -> Result<App, GeneratorError> {
    let json_value = get_file_json(src_file)?;
    validate_app_file(&json_value)
        .map_err(|e| GeneratorError::InvalidSchema(src_file.clone(), e))?;
    serde_json::from_value::<App>(json_value)
        .map_err(|e| GeneratorError::InvalidSchema(src_file.clone(), e.to_string()))
}

pub fn parse_scene_file(src_file: &PathBuf) -> Result<Scene, GeneratorError> {
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
    use crate::deserialiser::structures::*;
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
            features: vec!["clip_planes".to_string()],
            platform: "windows".to_string(),
            graphics: "vulkan".to_string(),
            start_scene: "scene".to_string()
        };
        assert_eq!(format!("{:?}", app_object), format!("{:?}", expected_app));

        let scene1_src = get_test_app_file("full_featured_app", "scene.json");
        let scene1_object = parse_scene_file(&scene1_src).unwrap();
        let expected_scene1 = Scene {
            id: "scene".to_string(),
            camera: "player".to_string(),
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
                        generator: Some("skybox".to_string())
                    }
                ],
                textures: vec![
                    Texture {
                        id: "outdoors".to_string(),
                        file: Some("simple_outdoor_texture.jpg".to_string()),
                        kind: None
                    },
                    Texture {
                        id: "musica".to_string(),
                        file: Some("Musica.png".to_string()),
                        kind: None
                    },
                    Texture {
                        id: "skybox".to_string(),
                        file: Some("bluecloud.jpg".to_string()),
                        kind: Some("cubemap".to_string())
                    },
                    Texture {
                        id: "reflection".to_string(),
                        file: None,
                        kind: Some("uninitialised".to_string())
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
                    kind: "offscreen".to_string(),
                    target_texture_id: Some("reflection".to_string()),
                    render: "reflection_pre_render".to_string(),
                    steps: vec![
                        Step {
                            model_id: "skybox".to_string(),
                            texture_ids: vec!["skybox".to_string()]
                        },
                        Step {
                            model_id: "terrain".to_string(),
                            texture_ids: vec!["outdoors".to_string()]
                        }
                    ]
                },
                Pass {
                    kind: "default".to_string(),
                    target_texture_id: None,
                    render: "basic_textured".to_string(),
                    steps: vec![
                        Step {
                            model_id: "river".to_string(),
                            texture_ids: vec!["reflection".to_string()]
                        },
                        Step {
                            model_id: "terrain".to_string(),
                            texture_ids: vec!["outdoors".to_string()]
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
            camera: "flight_path".to_string(),
            resources: Resources {
                models: vec![
                    Model {
                        id: "skybox".to_string(),
                        file: None,
                        generator: Some("skybox".to_string())
                    }
                ],
                textures: vec![
                    Texture {
                        id: "skybox".to_string(),
                        file: Some("bluecloud.jpg".to_string()),
                        kind: Some("cubemap".to_string())
                    }
                ],
                fonts: vec![]
            },
            passes: vec![
                Pass {
                    kind: "default".to_string(),
                    target_texture_id: None,
                    render: "basic_textured".to_string(),
                    steps: vec![
                        Step {
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
