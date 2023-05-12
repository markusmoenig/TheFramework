use theframework::prelude::*;

pub struct Circle {
    circle_id           : u32,
}

impl TheTrait for Circle {
    fn new() -> Self where Self: Sized {
    Self {
            circle_id   : 0,
        }
    }

    /// Init the scene by adding a shape to the world space
    fn init(&mut self, ctx: &mut TheContext) {

        // The world space always has the id of 0
        if let Some(world_space) = ctx.renderer.get_space_mut(0) {
            world_space.set_coord_system(Center);
            self.circle_id = world_space.add_shape(Disc);
            world_space.set_shape_property(self.circle_id, Normal, Color, vec!(1.0, 1.0, 1.0, 1.0));
            world_space.set_shape_property(self.circle_id, Normal, Radius, vec!(100.0));
            world_space.set_shape_property(self.circle_id, Selected, Color, vec!(1.0, 0.0, 0.0, 1.0));
            world_space.set_shape_property(self.circle_id, Selected, Radius, vec!(120.0));
        }
    }

    /// Draw a circle in the middle of the window
    fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext) {
        ctx.renderer.draw(pixels, ctx.width, ctx.height);
    }

    /// If the touch event is inside the circle, set the circle state to Selected
    fn touch_down(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        if let Some(world_space) = ctx.renderer.get_space_mut(0) {
            if let Some(shape_id) = world_space.get_shape_at(x, y) {
                world_space.set_shape_state(shape_id, Selected);
            } else {
                world_space.set_shape_state(self.circle_id, Normal);
            }
        }
        ctx.renderer.needs_update()
    }

    /// Set the circle state to Selected.
    fn touch_up(&mut self, _x: f32, _y: f32, ctx: &mut TheContext) -> bool {
        if let Some(world_space) = ctx.renderer.get_space_mut(0) {
            world_space.set_shape_state(self.circle_id, Normal);
        }
        ctx.renderer.needs_update()
    }

    /// Query if the renderer needs an update (tramsition animation ongoing etc.)
    fn needs_update(&mut self, ctx: &mut TheContext) -> bool {
        ctx.renderer.needs_update()
    }
}