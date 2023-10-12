pub mod thecanvas;
pub mod thedim;
pub mod thergbabuffer;
pub mod thesizelimiter;
pub mod thestyle;
pub mod thetheme;
pub mod theuicontext;
pub mod thevalue;
pub mod thevent;
pub mod thewidget;

use std::sync::mpsc::{self, Receiver};

pub use crate::prelude::*;

pub type RGBA = [u8; 4];
pub const BLACK: RGBA = [0, 0, 0, 255];
pub const WHITE: RGBA = [255, 255, 255, 255];

pub mod prelude {

    pub use crate::theui::RGBA;

    pub use crate::theui::BLACK;
    pub use crate::theui::WHITE;

    pub use crate::theui::thecanvas::*;
    pub use crate::theui::thedim::*;
    pub use crate::theui::thergbabuffer::TheRGBABuffer;
    pub use crate::theui::thesizelimiter::TheSizeLimiter;
    pub use crate::theui::theuicontext::*;
    pub use crate::theui::TheUI;

    pub use crate::theui::thevalue::TheValue;
    pub use crate::theui::thevent::TheEvent;

    pub use crate::theui::thewidget::colorbutton::*;
    pub use crate::theui::thewidget::prelude::*;

    pub use crate::theui::thestyle::prelude::*;
    pub use crate::theui::thestyle::TheStyle;

    pub use crate::theui::thetheme::prelude::*;
    pub use crate::theui::thetheme::{TheTheme, TheThemeColors, TheThemeColors::*};

    pub use crate::theui::thewidget::prelude::*;
    pub use crate::theui::thewidget::TheWidget;
}

pub struct TheUI {
    pub canvas: TheCanvas,

    pub style: Box<dyn TheStyle>,

    state_events_receiver: Option<Receiver<TheEvent>>,
}

impl Default for TheUI {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl TheUI {
    pub fn new() -> Self {
        Self {
            canvas: TheCanvas::new(),

            style: Box::new(TheClassicStyle::new()),

            state_events_receiver: None,
        }
    }

    pub fn init(&mut self, ctx: &mut TheContext) {
        let (tx, rx) = mpsc::channel();

        self.state_events_receiver = Some(rx);
        ctx.ui.state_events_sender = Some(tx);
    }

    pub fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext) {
        self.canvas.resize(ctx.width as i32, ctx.height as i32);
        self.canvas.draw(&mut self.style, ctx);

        pixels.copy_from_slice(self.canvas.buffer().pixels())
    }

    /// Processes widget state events, these are mostly send from TheUIContext based on state changes provided by the widgets.
    pub fn process_events(&mut self, ctx: &mut TheContext) {
        if let Some(receiver) = &mut self.state_events_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    TheEvent::Focus(id) => {
                        println!("Gained focus {:?}", id);
                    }
                    _ => {}
                }
            }
        }
    }

    fn needs_update(&mut self, ctx: &mut TheContext) -> bool {
        false
    }

    pub fn touch_down(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        let coord = vec2i(x as i32, y as i32);
        if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
            let event = TheEvent::MouseDown(TheValue::Coordinate(widget.dim().to_local((coord))));
            widget.on_event(&event, ctx);

            self.process_events(ctx);
        }

        false
    }
}
