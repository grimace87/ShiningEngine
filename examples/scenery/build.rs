
use collada::COLLADA;
use collada::config::Config;

use std::{
    env,
    path::{Path, PathBuf},
    fs::File,
    io::Read
};

/// Build script
///
/// Reads Collada files (*.dae) from the ./resources/models/ directory and writes the model data to
/// a custom binary format.
fn main() {
    let collada_models_dir = {
        let mut dir = std::env::current_dir().unwrap();
        dir.pop();
        dir.push("resources");
        dir.push("models");
        dir
    };
    let binary_models_dir = {
        let mut dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        dir.push("models");
        if !dir.is_dir() {
            std::fs::create_dir(&dir).unwrap();
        }
        dir
    };

    convert_collada_files_in_directory(&collada_models_dir, &binary_models_dir);
}

fn convert_collada_files_in_directory(collada_models_dir: &Path, binary_models_dir: &Path) {
    let mut files_processed = 0;
    for entry in std::fs::read_dir(collada_models_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let extension = match path.extension() {
            Some(e) => e,
            None => continue
        };
        match extension.to_str() {
            Some("dae") => {
                let mut config_path = path.clone();
                config_path.set_extension("toml");
                let config = match config_path.exists() {
                    true => Config::from_toml_file(&config_path),
                    false => Config::default()
                };
                convert_collada_file(&path, config, binary_models_dir);
                files_processed += 1;
            },
            _ => continue
        };
    }
    println!("Processed {} file(s) in directory {:?}", files_processed, collada_models_dir);
}

fn convert_collada_file(source_file: &Path, config: Config, binary_models_dir: &Path) {
    println!("Processing models in file {:?}: ", source_file);
    let mut collada_file = File::open(source_file)
        .expect("Failed to open a file");
    let file_metadata = std::fs::metadata(source_file)
        .expect("Failed to read file metadata");
    let mut file_bytes = vec![0; file_metadata.len() as usize];
    collada_file.read(&mut file_bytes)
        .expect("Buffer overflow reading from file");
    let collada = COLLADA::new(file_bytes.as_slice());
    let models = collada.extract_models(config);
    for model in models.iter() {
        let mut file_path = PathBuf::from(binary_models_dir);
        file_path.push(model.name.as_str());
        file_path.set_extension("mdl");
        unsafe {
            model.write_to_binary_file(&file_path).unwrap();
        }
        println!("  Wrote to {:?}", &file_path);
    }
    println!("  Processed {} models", models.len());
}
