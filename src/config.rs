use directories::ProjectDirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Stores the sentinel command map structure
// extension -> list of command strings
#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub commands: HashMap<String, Vec<String>>,
}

impl Config {
    fn get_config_path(dir: String) -> Option<PathBuf> {
        // check if the project has a .sentinel.yaml file
        let project_config = Path::new(&dir).join(".sentinel.yaml");

        if project_config.exists() {
            println!("Using project config file: {:?}", project_config);
            return Some(project_config);
        }

        // if theres a global config yaml file
        if let Some(proj_dirs) = ProjectDirs::from("", "", "sentinel") {
            let global_config = proj_dirs.config_dir().join("global.yaml");
            if global_config.exists() {
                println!("Using global config file: {:?}", global_config);
                return Some(global_config);
            }
        }
        None
    }

    pub fn load_config(dir: String) -> Option<Self> {
        if let Some(config_path) = Self::get_config_path(dir) {
            let file = std::fs::File::open(config_path).expect("Could not open config file");
            let config: Config =
                serde_yaml::from_reader(file).expect("Could not parse config file");
            return Some(config);
        }
        None
    }
}
