
mod structures;
mod validator;

use structures::*;
use validator::validate_file;
use std::path::PathBuf;

pub fn parse_file(src_file: &PathBuf) -> Result<Config, String> {
    let src = std::fs::read_to_string(src_file)
        .map_err(|_| format!("Failed to open {}", src_file.to_str().unwrap()))?;
    let json_value = serde_json::from_str(src.as_str())
        .map_err(|e| format!("Failed to parse test JSON: {:?}", e))?;
    validate_file(&json_value)
        .map_err(|e| format!("File {:?} failed validation: {}", src_file.to_str().unwrap(), e))?;
    serde_json::from_value::<Config>(json_value)
        .map_err(|e| format!("Failed to deserialise: {:?}", e))
}

/// Test suite
/// Needs to test that JSON specifications pass validation as per the JSON Schema, that they can
/// also be decoded to Rust structures, and that working renderers will be generated from those
/// structures.
#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use crate::deserialiser::validator::validate_file;

    fn get_test_file(file_name: &'static str) -> serde_json::Value {
        let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        src_path.push("resources");
        src_path.push("test");
        src_path.push(file_name);
        let src = std::fs::read_to_string(src_path)
            .expect(format!("Failed to open {}", file_name).as_str());
        serde_json::from_str(src.as_str())
            .expect("Failed to parse test JSON")
    }

    #[test]
    fn app_with_known_features_passes_validation() {
        let src_json = get_test_file("features_good.json");
        let validation_result = validate_file(&src_json);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn app_with_unknown_features_fails_validation() {
        let src_json = get_test_file("features_bad.json");
        let validation_result = validate_file(&src_json);
        assert!(validation_result.is_err());
    }

    #[test]
    fn valid_full_featured_app_passes_validation() {
        let src_json = get_test_file("full_featured_app.json");
        let validation_result = validate_file(&src_json);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn valid_full_featured_app_can_deserialise() {
        let src_json = get_test_file("full_featured_app.json");
        let deserialise_result = serde_json::from_value::<super::Config>(src_json);
        assert!(deserialise_result.is_ok());
    }

    #[test]
    fn deserialised_full_featured_app_matches_expectations() {
        use crate::deserialiser::structures::*;

        let src_json = get_test_file("full_featured_app.json");
        let object = serde_json::from_value::<Config>(src_json).unwrap();
        let expected = Config {
            app: App {
                name: "Full-featured example which should pass validation".to_string(),
                features: vec!["clip_planes".to_string()]
            },
            scenes: vec![
                Scene {
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
                },
                Scene {
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
                }
            ]
        };
        assert_eq!(format!("{:?}", object), format!("{:?}", expected));
    }
}
