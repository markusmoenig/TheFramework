use theframework::*;

pub struct Circle {
    radius          : usize,
}

impl TheTrait for Circle {
    fn new() -> Self where Self: Sized {
        Self {
            radius  : 100,
        }
    }

    /// Draw a circle in the middle of the window
    fn draw(&mut self, pixels: &mut [u8], ctx: &TheContext) {

        ctx.draw.circle(pixels, &(ctx.width / 2 - self.radius, ctx.height / 2 - self.radius, self.radius * 2, self.radius * 2), ctx.width, &[255, 255, 255, 255], self.radius);
    }

    /// Update the app state.
    fn update(&mut self) {
    }
}