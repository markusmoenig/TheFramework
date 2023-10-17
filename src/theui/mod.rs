pub mod thecanvas;
pub mod thedim;
pub mod thelayout;
pub mod thergbabuffer;
pub mod thesizelimiter;
pub mod thestyle;
pub mod thetheme;
pub mod theuicontext;
pub mod thevalue;
pub mod thevent;
pub mod thewidget;

use std::sync::mpsc::{self, Sender, Receiver};

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

    pub use crate::theui::thelayout::prelude::*;
    pub use crate::theui::thewidget::prelude::*;
    pub use crate::theui::thewidget::TheWidget;
}

pub struct TheUI {
    pub canvas: TheCanvas,

    pub style: Box<dyn TheStyle>,

    state_events_receiver: Option<Receiver<TheEvent>>,

    app_state_events: FxHashMap<String, Sender<TheEvent>>,

    is_dirty: bool
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
            app_state_events: FxHashMap::default(),

            is_dirty: false,
        }
    }

    pub fn init(&mut self, ctx: &mut TheContext) {
        let (tx, rx) = mpsc::channel();

        self.state_events_receiver = Some(rx);
        ctx.ui.state_events_sender = Some(tx);
    }

    /// Adds a widget state listener of the given name. Returns the Receiver<TheEvent> which the app can user to react to widget state changes. An app can add several listeners.
    pub fn add_state_listener(&mut self, name: String) -> Receiver<TheEvent> {
        let (tx, rx) = mpsc::channel();
        self.app_state_events.insert(name, tx);
        rx
    }

    pub fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext) {
        self.canvas.resize(ctx.width as i32, ctx.height as i32);
        self.canvas.draw(&mut self.style, ctx);

        pixels.copy_from_slice(self.canvas.buffer().pixels());
        self.is_dirty = false;
    }

    /// Processes widget state events, these are mostly send from TheUIContext based on state changes provided by the widgets.
    pub fn process_events(&mut self, ctx: &mut TheContext) {
        if let Some(receiver) = &mut self.state_events_receiver {
            while let Ok(event) = receiver.try_recv() {

                // Resend event to all app listeners
                for (name, sender) in &self.app_state_events {
                    sender.send(event.clone()).unwrap();
                }

                match event {
                    TheEvent::StateChanged(id, state) => {
                        println!("Widget State changed {:?}: {:?}", id, state);
                    },
                    TheEvent::SetState(name, state) => {
                        println!("Set State {:?}: {:?}", name, state);
                        if let Some(widget) = self.canvas.get_widget(Some(&name), None) {
                            widget.set_state(state);
                        }
                        self.is_dirty = true;
                    },
                    TheEvent::GainedFocus(id) => {
                        println!("Gained focus {:?}", id);
                    },
                    TheEvent::LostFocus(id) => {
                        println!("Lost focus {:?}", id);
                        if let Some(widget) = self.canvas.get_widget(None, Some(&id.uuid)) {
                            widget.set_needs_redraw(true);
                        }
                    },
                    TheEvent::GainedHover(id) => {
                        println!("Gained hover {:?}", id);
                    },
                    TheEvent::LostHover(id) => {
                        println!("Lost hover {:?}", id);
                        if let Some(widget) = self.canvas.get_widget(None, Some(&id.uuid)) {
                            widget.set_needs_redraw(true);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn needs_update(&mut self, ctx: &mut TheContext) -> bool {
        self.process_events(ctx);
        self.is_dirty
    }

    pub fn touch_down(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let coord = vec2i(x as i32, y as i32);
        if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
            let event = TheEvent::MouseDown(TheValue::Coordinate(widget.dim().to_local(coord)));
            redraw = widget.on_event(&event, ctx);

            self.process_events(ctx);
        }
        redraw
    }

    pub fn touch_dragged(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        false
    }

    pub fn touch_up(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        false
    }

    pub fn hover(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let coord = vec2i(x as i32, y as i32);
        if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
            let event = TheEvent::Hover(TheValue::Coordinate(widget.dim().to_local(coord)));
            redraw = widget.on_event(&event, ctx);

            // If the new hover widget does not support a hover state, make sure to unhover the current widget if any
            if !widget.supports_hover() {
                if let Some(hover) = &ctx.ui.hover {
                    ctx.ui.send_state(TheEvent::LostHover(hover.clone()));
                    redraw = true;
                    ctx.ui.hover = None;
                }
            }

            self.process_events(ctx);
        } else if let Some(hover) = &ctx.ui.hover {
            ctx.ui.send_state(TheEvent::LostHover(hover.clone()));
            redraw = true;
            ctx.ui.hover = None;
            self.process_events(ctx);
        }
        redraw
    }
}
