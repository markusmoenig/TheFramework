//use crate::prelude::*;
//use theframework::prelude::*;
use rust_pathtracer::prelude::*;

#[derive(Clone, Debug)]
pub struct Project {
    pub material: Material,
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

impl Project {
    pub fn new() -> Self {
        let mut material = Material::new();
        material.roughness = 0.04;
        material.metallic = 1.0;

        Self { material }
    }
}
