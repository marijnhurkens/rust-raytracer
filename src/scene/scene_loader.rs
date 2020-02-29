use serde_yaml;

use super::*;

pub fn load_scene(yaml: &String) {
    let scene: Scene = serde_yaml::from_str(&yaml).unwrap();
}

