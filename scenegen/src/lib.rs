use serde::Deserialize;

/// Config struct
/// Top-level struct defining an entire application in an abstract way
#[derive(Debug, Deserialize)]
pub struct Config {
    pub app: App,
    pub scenes: Vec<Scene>
}

/// App struct
/// Defines some top-level properties of the application
#[derive(Debug, Deserialize)]
pub struct App {
    pub name: String,
    pub features: Vec<String>
}

/// Scene struct
/// Defines a scene of the application - generally this will be a continuous piece of gameplay
/// without interruptions, such as a player flying across a planet - and typically separated from
/// other scenes by a screen transition of some sort in which there's briefly no player interaction
#[derive(Debug, Deserialize)]
pub struct Scene {
    pub id: String,
    pub camera: String,
    pub resources: Resources,
    pub passes: Vec<Pass>
}

/// Resources struct
/// Defines all the resources that will be used by the scene and should therefore be preloaded
/// upfront at the same time
#[derive(Debug, Deserialize)]
pub struct Resources {
    pub models: Vec<Model>,
    pub textures: Vec<Texture>,
    pub fonts: Vec<Font>
}

/// Model struct
/// Defines a model, including some kind of source of its vertex data
#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
    pub file: Option<String>,
    pub generator: Option<String>
}

/// Texture struct
/// Defines a texture, optionally including some kind of source of its pixel data
#[derive(Debug, Deserialize)]
pub struct Texture {
    pub id: String,
    pub file: Option<String>,
    pub kind: Option<String>
}

/// Font struct
/// Defines a font, including its glyph definition file and a reference to the texture which holds
/// its pixel data
#[derive(Debug, Deserialize)]
pub struct Font {
    pub id: String,
    pub file: String,
    pub texture_id: String
}

/// Pass struct
/// Defines one of the one-or-more rendering passes required to draw this scene, including a shader
/// and render target (offscreen buffer or the default framebuffer), plus the various steps to draw
/// using that configuration
#[derive(Debug, Deserialize)]
pub struct Pass {
    pub kind: String,
    pub target_texture_id: Option<String>,
    pub render: String,
    pub steps: Vec<Step>
}

/// Step struct
/// Defines a step within a pass - this basically comprises drawing a model with whatever number of
/// textures are required by the shader for the parent pass
#[derive(Debug, Deserialize)]
pub struct Step {
    pub model_id: String,
    pub texture_ids: Vec<String>
}

/// Test suite
/// Needs to test that JSON specifications pass validation as per the JSON Schema, that they can
/// also be decoded to Rust structures, and that working renderers will be generated from those
/// structures.
#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use jsonschema::JSONSchema;

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

    fn compile_schema() -> JSONSchema {
        let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        src_path.push("resources");
        src_path.push("scenegen.schema.json");
        let src_string = std::fs::read_to_string(src_path)
            .expect("Failed to open schema file");
        let src_json = serde_json::from_str(src_string.as_str())
            .expect("Invalid JSON schema");
        let compiled_schema = JSONSchema::compile(&src_json)
            .expect("Invalid JSON schema");
        compiled_schema
    }

    #[test]
    fn app_with_known_features_passes_validation() {
        let src_json = get_test_file("features_good.json");
        let schema = compile_schema();
        let validation_result = schema.validate(&src_json);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn app_with_unknown_features_fails_validation() {
        let src_json = get_test_file("features_bad.json");
        let schema = compile_schema();
        let validation_result = schema.validate(&src_json);
        assert!(validation_result.is_err());
    }

    #[test]
    fn valid_full_featured_app_passes_validation() {
        let src_json = get_test_file("full_featured_app.json");
        let schema = compile_schema();
        let validation_result = schema.validate(&src_json);
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
        use super::*;

        let src_json = get_test_file("full_featured_app.json");
        let object = serde_json::from_value::<super::Config>(src_json).unwrap();
        let expected = Config {
            app: App {
                name: "Scenery Example".to_string(),
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
