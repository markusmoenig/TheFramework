use theframework::*;

pub struct Circle {

    circle_x            : usize,
    circle_y            : usize,

    radius              : usize,

    clicked             : bool,
}

impl TheTrait for Circle {
    fn new() -> Self where Self: Sized {
        Self {

            circle_x    : 0,
            circle_y    : 0,
            radius      : 200,

            clicked     : false,
        }
    }

    /// Draw a circle in the middle of the window
    fn draw(&mut self, pixels: &mut [u8], ctx: &TheContext) {

        let color = if self.clicked { [255, 0, 0, 255] } else { [255, 255, 255, 255] };

        self.circle_x = ctx.width / 2;
        self.circle_y = ctx.height / 2;

        ctx.draw.rect(pixels, &(0, 0, ctx.width, ctx.height), ctx.width, &[0, 0, 0, 255]);
        ctx.draw.circle(pixels,
            &(self.circle_x - self.radius, self.circle_y - self.radius, self.radius * 2, self.radius * 2), ctx.width,
            &color,
            self.radius);
    }

    /// Click / touch at the given position, check if we clicked inside the circle
    fn touch_down(&mut self, x: f32, y: f32) -> bool {

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

        true
    }

    fn touch_up(&mut self, _x: f32, _y: f32) -> bool {
        self.clicked = false;
        true
    }

    /// Update the app state
    fn update(&mut self) {
    }
}