use crate::prelude::*;

/// TheCodeBundle is a collections of codegrids which make up the behavior of an entity.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TheCodeBundle {
    pub name: String,
    pub uuid: Uuid,
    pub grids: FxHashMap<Uuid, TheCodeGrid>,
}

impl Default for TheCodeBundle {
    fn default() -> Self {
        TheCodeBundle::new()
    }
}

impl TheCodeBundle {
    pub fn new() -> Self {

        let mut grids = FxHashMap::default();
        let def = TheCodeGrid::default();
        grids.insert(def.uuid, def);

        Self {
            name: "Unnamed".to_string(),
            uuid: Uuid::new_v4(),
            grids,
        }
    }

    /// Insert a codegrid into the bundle.
    pub fn insert_grid(&mut self, grid: TheCodeGrid) {
        self.grids.insert(grid.uuid, grid);
    }

    /// Get a grid from the module.
    pub fn get_grid(&self, id: &Uuid) -> Option<&TheCodeGrid> {
        self.grids.get(id)
    }

    /// Get a mutable grid from the module.
    pub fn get_grid_mut(&mut self, id: &Uuid) -> Option<&mut TheCodeGrid> {
        self.grids.get_mut(id)
    }
}
