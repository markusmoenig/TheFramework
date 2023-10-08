use theframework::prelude::*;

pub struct UIDemo {}

impl TheTrait for UIDemo {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {}
}
