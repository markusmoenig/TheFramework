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
            world_space.set_shape_property(self.circle_id, Normal, Color, vec!(1.0, 0.0, 0.0, 1.0));
        }
    }

    /// Draw a circle in the middle of the window
    fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext) {
        ctx.renderer.draw(pixels, ctx.width, ctx.height);
    }

    /// Click / touch at the given position, check if we clicked inside the circle
    fn touch_down(&mut self, x: f32, y: f32) -> bool {
        /*

        /// Length of a 2d vector
        #[inline(always)]
        fn length(v: (f32, f32)) -> f32 {
            ((v.0).powf(2.0) + (v.1).powf(2.0)).sqrt()
        }

        let dist = length((x - self.circle_x as f32, y - self.circle_y as f32)) - self.radius as f32;

        if dist <= 0.0 {
            // Clicked inside
            self.clicked = true;
        } else {
            self.clicked = false;
        }
        */
        true
    }

    fn touch_up(&mut self, _x: f32, _y: f32) -> bool {
        true
    }

    /// Update the app state
    fn update(&mut self) {
    }
}