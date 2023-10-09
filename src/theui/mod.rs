pub mod thecanvas;
pub mod thedim;
pub mod thergbabuffer;
pub mod theuicontext;
pub mod thewidget;
pub mod thesizelimiter;

pub use crate::prelude::*;

pub type RGBA = [u8;4];
pub const BLACK : RGBA =  [0, 0, 0, 255];
pub const WHITE : RGBA =  [255, 255, 255, 255];

pub mod prelude {

    pub use crate::theui::RGBA;

    pub use crate::theui::BLACK;
    pub use crate::theui::WHITE;

    pub use crate::theui::thecanvas::*;
    pub use crate::theui::thedim::*;
    pub use crate::theui::thergbabuffer::TheRGBABuffer;
    pub use crate::theui::theuicontext::*;
    pub use crate::theui::TheUI;
    pub use crate::theui::thesizelimiter::TheSizeLimiter;

    pub use crate::theui::thewidget::prelude::*;

    pub use crate::theui::thewidget::colorbutton::*;
    pub use crate::theui::thewidget::TheWidget;
}

pub struct TheUI {
    pub canvas: TheCanvas,
}

#[allow(unused)]
impl TheUI {
    pub fn new() -> Self {
        Self {
            canvas: TheCanvas::new(),
        }
    }

    fn init(&mut self, ctx: &mut TheContext) {}

    pub fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext) {
        self.canvas.resize(ctx.width as i32, ctx.height as i32);
        self.canvas.draw(ctx);

        pixels.copy_from_slice(self.canvas.get_buffer().get())
    }

    fn update(&mut self, ctx: &mut TheContext) {}

    fn needs_update(&mut self, ctx: &mut TheContext) -> bool {
        false
    }
}
