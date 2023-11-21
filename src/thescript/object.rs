use crate::prelude::*;

// #[derive(PartialEq, Debug, Clone)]
// enum NodeType {
//     Node,
//     _Function,
//     _Object2D,
//     _Object3D,
// }

#[derive(PartialEq, Debug, Clone)]
pub struct Object {
    pub name: String,
    pub chunk: TheChunk,
}

impl Object {
    pub fn new(name: String) -> Self {
        println!("new node {}", name);

        Self {
            name,
            chunk: TheChunk::new(),
        }
    }

    pub fn add_property(&mut self, name: String, value: Value) {
        println!("add property {} : {:?}", name, value.as_string());
    }
}
