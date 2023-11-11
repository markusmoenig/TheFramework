pub mod thecanvas;
pub mod thedim;
pub mod theid;
pub mod thelayout;
pub mod thergbabuffer;
pub mod thesizelimiter;
pub mod thestyle;
pub mod thetheme;
pub mod theuicontext;
pub mod thevalue;
pub mod thevent;
pub mod thewidget;

use ::serde::de::{self, Deserializer};
use ::serde::ser::{self, Serializer};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};

fn compress<S>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).map_err(ser::Error::custom)?;
    let compressed_data = encoder.finish().map_err(ser::Error::custom)?;

    serializer.serialize_bytes(&compressed_data)
}

fn decompress<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let data = Vec::<u8>::deserialize(deserializer)?;
    let mut decoder = ZlibDecoder::new(&data[..]);
    let mut decompressed_data = Vec::new();
    decoder
        .read_to_end(&mut decompressed_data)
        .map_err(de::Error::custom)?;

    Ok(decompressed_data)
}

pub use crate::prelude::*;

pub type RGBA = [u8; 4];
pub const TRANSPARENT: RGBA = [0, 0, 0, 0];
pub const BLACK: RGBA = [0, 0, 0, 255];
pub const WHITE: RGBA = [255, 255, 255, 255];

pub mod prelude {
    pub use serde::{Deserialize, Serialize};

    pub use crate::theui::RGBA;

    pub use crate::theui::BLACK;
    pub use crate::theui::WHITE;

    pub use std::rc::Rc;

    pub use crate::theui::theid::TheId;

    pub use crate::theui::thecanvas::*;
    pub use crate::theui::thedim::*;
    pub use crate::theui::thergbabuffer::TheRGBABuffer;
    pub use crate::theui::thesizelimiter::TheSizeLimiter;
    pub use crate::theui::theuicontext::*;
    pub use crate::theui::TheUI;

    pub use crate::theui::thevalue::TheValue;
    pub use crate::theui::thevent::TheEvent;

    pub use crate::theui::thewidget::prelude::*;
    pub use crate::theui::thewidget::thecolorbutton::*;

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

    pub is_dirty: bool,
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
        self.canvas.resize(ctx.width as i32, ctx.height as i32, ctx);
        if ctx.ui.relayout {
            let width = self.canvas.buffer().dim().width;
            let height = self.canvas.buffer().dim().height;
            self.canvas.layout(width, height, ctx);
            ctx.ui.relayout = false;
        }
        self.canvas.draw(&mut self.style, ctx);
        self.canvas.draw_overlay(&mut self.style, ctx);
        ctx.ui.redraw_all = false;

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
                    TheEvent::NewListItemSelected(id, layout_id) => {
                        if let Some(layout) = self.canvas.get_layout(None, Some(&layout_id.uuid)) {
                            if let Some(list) = layout.as_list_layout() {
                                list.new_item_selected(id);
                            }
                        }
                    }
                    TheEvent::SetStackIndex(id, index) => {
                        if let Some(layout) = self.canvas.get_layout(None, Some(&id.uuid)) {
                            if let Some(stack) = layout.as_stack_layout() {
                                if stack.index() != index {
                                    stack.set_index(index);
                                    self.is_dirty = true;
                                    ctx.ui.redraw_all = true;
                                    ctx.ui.relayout = true;
                                }
                            }
                        }
                    }
                    TheEvent::StateChanged(id, state) => {
                        //println!("Widget State changed {:?}: {:?}", id, state);
                    }
                    TheEvent::SetState(name, state) => {
                        //println!("Set State {:?}: {:?}", name, state);
                        if let Some(widget) = self.canvas.get_widget(Some(&name), None) {
                            widget.set_state(state);
                        }
                        self.is_dirty = true;
                    }
                    TheEvent::GainedFocus(id) => {
                        //println!("Gained focus {:?}", id);
                    }
                    TheEvent::LostFocus(id) => {
                        //println!("Lost focus {:?}", id);
                        if let Some(widget) = self.canvas.get_widget(None, Some(&id.uuid)) {
                            widget.on_event(&TheEvent::LostFocus(widget.id().clone()), ctx);
                            widget.set_needs_redraw(true);
                        }
                    }
                    TheEvent::GainedHover(id) => {
                        //println!("Gained hover {:?}", id);
                    }
                    TheEvent::LostHover(id) => {
                        //println!("Lost hover {:?}", id);
                        if let Some(widget) = self.canvas.get_widget(None, Some(&id.uuid)) {
                            widget.on_event(&TheEvent::LostHover(widget.id().clone()), ctx);
                            widget.set_needs_redraw(true);
                        }
                    }
                    TheEvent::ValueChanged(id, value) => {
                        //println!("Widget Value changed {:?}: {:?}", id, value);
                    }
                    TheEvent::SetValue(name, value) => {
                        //println!("Set Value {:?}: {:?}", name, value);
                        if let Some(widget) = self.canvas.get_widget(Some(&name), None) {
                            widget.set_value(value);
                        }
                        self.is_dirty = true;
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn update(&mut self, ctx: &mut TheContext) -> bool {
        // Check if the result of an FileRequester is available, and if yes, send the result
        if let Some(rx) = &ctx.ui.file_requester_receiver {
            let rc = rx.1.try_recv();
            if let Ok(paths) = rc {
                ctx.ui
                    .send(TheEvent::FileRequesterResult(rx.0.clone(), paths));
                ctx.ui.file_requester_receiver = None;
            }
        }

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
        let mut redraw = false;
        let coord = vec2i(x as i32, y as i32);

        if let Some(id) = &ctx.ui.focus {
            if let Some(widget) = self.canvas.get_widget(Some(&id.name), Some(&id.uuid)) {
                let event =
                    TheEvent::MouseDragged(TheValue::Coordinate(widget.dim().to_local(coord)));
                redraw = widget.on_event(&event, ctx);
                self.process_events(ctx);
            }
        } else if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
            let event = TheEvent::MouseDragged(TheValue::Coordinate(widget.dim().to_local(coord)));
            redraw = widget.on_event(&event, ctx);
            self.process_events(ctx);
        }
        redraw
    }

    pub fn touch_up(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let coord = vec2i(x as i32, y as i32);

        if let Some(id) = &ctx.ui.focus {
            if let Some(widget) = self.canvas.get_widget(Some(&id.name), Some(&id.uuid)) {
                let event = TheEvent::MouseUp(TheValue::Coordinate(widget.dim().to_local(coord)));
                redraw = widget.on_event(&event, ctx);
                self.process_events(ctx);
            }
        } else if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
            let event = TheEvent::MouseUp(TheValue::Coordinate(widget.dim().to_local(coord)));
            redraw = widget.on_event(&event, ctx);
            self.process_events(ctx);
        }
        redraw
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
                    ctx.ui.send(TheEvent::LostHover(hover.clone()));
                    redraw = true;
                    ctx.ui.hover = None;
                }
            }

            self.process_events(ctx);
        } else if let Some(hover) = &ctx.ui.hover {
            ctx.ui.send(TheEvent::LostHover(hover.clone()));
            redraw = true;
            ctx.ui.hover = None;
            self.process_events(ctx);
        }
        redraw
    }

    pub fn key_down(
        &mut self,
        char: Option<char>,
        key: Option<TheKeyCode>,
        ctx: &mut TheContext,
    ) -> bool {
        let mut redraw = false;
        if let Some(id) = &ctx.ui.focus {
            if let Some(widget) = self.canvas.get_widget(Some(&id.name), Some(&id.uuid)) {
                let event = if let Some(c) = char {
                    TheEvent::KeyDown(TheValue::Char(c))
                } else {
                    TheEvent::KeyCodeDown(TheValue::KeyCode(key.unwrap()))
                };
                redraw = widget.on_event(&event, ctx);
                self.process_events(ctx);
            }
        }
        redraw
    }
}
