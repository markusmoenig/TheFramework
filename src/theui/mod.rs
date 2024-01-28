pub mod thecanvas;
pub mod thecontextmenu;
pub mod thedim;
pub mod thedrop;
pub mod theid;
pub mod thelayout;
pub mod thergbabuffer;
pub mod thesdf;
pub mod thesizelimiter;
pub mod thestyle;
pub mod thetheme;
pub mod theuicontext;
pub mod theuiglobals;
pub mod theundo;
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

    #[cfg(feature = "code")]
    pub use crate::thecode::prelude::*;

    pub use crate::theui::thecanvas::*;
    pub use crate::theui::thedim::*;
    pub use crate::theui::thergbabuffer::{
        TheRGBABuffer, TheRGBARegion, TheRGBARegionSequence, TheRGBATile,
    };
    pub use crate::theui::thesizelimiter::TheSizeLimiter;
    pub use crate::theui::theuicontext::*;
    pub use crate::theui::TheUI;

    pub use crate::theui::thevalue::{TheValue, TheValueAssignment, TheValueComparison};
    pub use crate::theui::thevent::TheEvent;

    pub use crate::theui::thewidget::prelude::*;
    pub use crate::theui::thewidget::thecolorbutton::*;

    pub use crate::theui::thestyle::prelude::*;
    pub use crate::theui::thestyle::TheStyle;

    pub use crate::theui::thetheme::prelude::*;
    pub use crate::theui::thetheme::{TheTheme, TheThemeColors, TheThemeColors::*};

    pub use crate::theui::thelayout::prelude::*;
    pub use crate::theui::thesdf::thepattern::ThePattern;
    pub use crate::theui::thesdf::thesdfcanvas::TheSDFCanvas;
    pub use crate::theui::thesdf::*;
    pub use crate::theui::thewidget::TheWidget;

    pub use crate::theui::thecontextmenu::*;
    pub use crate::theui::thedrop::*;
    pub use crate::theui::theuiglobals::*;
    pub use crate::theui::theundo::*;

    pub use crate::theui::TheDialogButtonRole;
}

/// The roles for dialog buttons.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TheDialogButtonRole {
    Accept,
    Reject,
}

impl TheDialogButtonRole {
    pub fn to_string(self) -> &'static str {
        match self {
            Self::Accept => "Accept",
            Self::Reject => "Cancel",
        }
    }
    pub fn to_id(self) -> &'static str {
        match self {
            Self::Accept => "TheDialogButtonRole::Accept",
            Self::Reject => "TheDialogButtonRole::Reject",
        }
    }
    pub fn iterator() -> impl Iterator<Item = TheDialogButtonRole> {
        [Self::Accept, Self::Reject].iter().copied()
    }
}

pub struct TheUI {
    pub canvas: TheCanvas,

    pub dialog_text: String,
    pub dialog: Option<TheCanvas>,

    pub style: Box<dyn TheStyle>,

    state_events_receiver: Option<Receiver<TheEvent>>,

    app_state_events: FxHashMap<String, Sender<TheEvent>>,

    statusbar_name: Option<String>,

    pub context_menu: Option<TheContextMenu>,

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

            dialog_text: "".to_string(),
            dialog: None,

            statusbar_name: None,

            context_menu: None,

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

    pub fn set_statusbar_name(&mut self, name: String) {
        self.statusbar_name = Some(name);
    }

    pub fn relayout(&mut self, ctx: &mut TheContext) {
        let width = self.canvas.buffer().dim().width;
        let height = self.canvas.buffer().dim().height;
        self.canvas.layout(width, height, ctx);
        ctx.ui.relayout = false;
    }

    pub fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext) {
        if self.canvas.resize(ctx.width as i32, ctx.height as i32, ctx) {
            ctx.ui.send(TheEvent::Resize);
            ctx.ui.relayout = false;
        }
        if ctx.ui.relayout {
            self.relayout(ctx);
        }
        self.canvas.draw(&mut self.style, ctx);
        if self.dialog.is_some() {
            self.draw_dialog(ctx);
        }
        self.canvas.draw_overlay(&mut self.style, ctx);
        if let Some(drop) = &ctx.ui.drop {
            if let Some(position) = &drop.position {
                self.canvas.buffer.blend_into(
                    position.x - drop.offset.x,
                    position.y - drop.offset.y,
                    &drop.image,
                )
            }
        }
        if let Some(menu) = &mut self.context_menu {
            menu.draw(self.canvas.buffer.pixels_mut(), &mut self.style, ctx);
        }
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
                    TheEvent::ShowContextMenu(id, coord, mut menu) => {
                        menu.set_position(coord, ctx);
                        menu.id = id;
                        self.context_menu = Some(menu);
                    }
                    TheEvent::RedirectWidgetValueToLayout(layout_id, widget_id, value) => {
                        if let Some(layout) = self.canvas.get_layout(None, Some(&layout_id.uuid)) {
                            layout.redirected_widget_value(&widget_id, &value, ctx);
                        }
                    }
                    TheEvent::DragStartedWithNoImage(drop) => {
                        let mut drop = drop.clone();
                        self.style.create_drop_image(&mut drop, ctx);
                        ctx.ui.drop = Some(drop);
                    }
                    TheEvent::NewListItemSelected(id, layout_id) => {
                        if let Some(layout) = self.canvas.get_layout(None, Some(&layout_id.uuid)) {
                            if let Some(list) = layout.as_list_layout() {
                                list.new_item_selected(id);
                            }
                        }
                    }
                    TheEvent::ScrollLayout(layout_id, delta) => {
                        if let Some(layout) = self.canvas.get_layout(None, Some(&layout_id.uuid)) {
                            if let Some(list) = layout.as_list_layout() {
                                list.scroll_by(delta);
                                self.is_dirty = true;
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
                        } else if let Some(layout) = self.canvas.get_layout(Some(&id.name), None) {
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

                        if let Some(dialog) = &mut self.dialog {
                            // If a dialog, close it if one of the dialog buttons was clicked.
                            if state == TheWidgetState::Clicked
                                && id.name.starts_with("TheDialogButtonRole")
                            {
                                for button in TheDialogButtonRole::iterator() {
                                    if id.name == button.to_id() {
                                        if let Some(widget) = dialog
                                            .get_widget(Some(&"Dialog Value".to_string()), None)
                                        {
                                            let value = widget.value();
                                            ctx.ui.send(TheEvent::DialogValueOnClose(
                                                self.dialog_text.clone(),
                                                value,
                                            ));
                                        }
                                    }
                                }
                                self.dialog = None;
                            }
                        }
                    }
                    TheEvent::SetState(name, state) => {
                        //println!("Set State {:?}: {:?}", name, state);
                        if let Some(widget) = self.canvas.get_widget(Some(&name), None) {
                            widget.set_state(state);
                        }
                        self.is_dirty = true;
                    }
                    TheEvent::ScrollBy(id, delta) => {
                        //println!("Set State {:?}: {:?}", name, state);
                        if let Some(widget) = self.canvas.get_widget(None, Some(&id.uuid)) {
                            widget.on_event(&TheEvent::ScrollBy(id.clone(), delta), ctx);
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
                        if let Some(statusbar_name) = &self.statusbar_name {
                            let mut status_text: Option<String> = None;
                            if let Some(widget) = self.canvas.get_widget(None, Some(&id.uuid)) {
                                status_text = widget.status_text();
                            }

                            if let Some(widget) = self.canvas.get_widget(Some(statusbar_name), None)
                            {
                                if let Some(widget) = widget.as_statusbar() {
                                    if let Some(status_text) = status_text {
                                        widget.set_text(status_text);
                                    } else {
                                        widget.set_text("".to_string());
                                    }
                                }
                            }
                        }
                    }
                    TheEvent::LostHover(id) => {
                        //println!("Lost hover {:?}", id);
                        if let Some(widget) = self.canvas.get_widget(None, Some(&id.uuid)) {
                            widget.on_event(&TheEvent::LostHover(widget.id().clone()), ctx);
                            widget.set_needs_redraw(true);
                        }
                        if let Some(statusbar_name) = &self.statusbar_name {
                            let mut status_text: Option<String> = None;

                            if let Some(widget) = self.canvas.get_widget(Some(statusbar_name), None)
                            {
                                if let Some(widget) = widget.as_statusbar() {
                                    if let Some(status_text) = status_text {
                                        widget.set_text(status_text);
                                    } else {
                                        widget.set_text("".to_string());
                                    }
                                }
                            }
                        }
                    }
                    TheEvent::SetStatusText(_id, text) => {
                        if let Some(statusbar_name) = &self.statusbar_name {
                            if let Some(widget) = self.canvas.get_widget(Some(statusbar_name), None)
                            {
                                if let Some(widget) = widget.as_statusbar() {
                                    widget.set_text(text);
                                }
                            }
                        }
                    }
                    TheEvent::ValueChanged(id, value) => {
                        //println!("Widget Value changed {:?}: {:?}", id, value);
                    }
                    TheEvent::SetValue(uuid, value) => {
                        //println!("Set Value {:?}: {:?}", name, value);
                        if let Some(widget) = self.canvas.get_widget(None, Some(&uuid)) {
                            widget.set_value(value.clone());
                            ctx.ui.send_widget_value_changed(widget.id(), value);
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

    pub fn context(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let coord = vec2i(x as i32, y as i32);
        if let Some(widget) = self.get_widget_at_coord(coord) {
            let event = TheEvent::Context(coord);
            redraw = widget.on_event(&event, ctx);

            self.process_events(ctx);
        }
        redraw
    }

    pub fn touch_down(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let coord = vec2i(x as i32, y as i32);

        if let Some(context) = &mut self.context_menu {
            if context.dim.contains(coord) {
                let event = TheEvent::MouseDown(context.dim.to_local(coord));
                if context.on_event(&event, ctx) {
                    redraw = true;
                    if let Some(menu_id) = &context.hovered {
                        ctx.ui.send(TheEvent::ContextMenuSelected(
                            context.id.clone(),
                            menu_id.clone(),
                        ));
                    }
                    self.context_menu = None;
                }
            } else {
                self.context_menu = None;
                redraw = true;
            }
            return redraw;
        }

        if let Some(widget) = self.get_widget_at_coord(coord) {
            let event = TheEvent::MouseDown(widget.dim().to_local(coord));
            redraw = widget.on_event(&event, ctx);

            self.process_events(ctx);
        }
        redraw
    }

    pub fn touch_dragged(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let coord = vec2i(x as i32, y as i32);

        if let Some(context) = &mut self.context_menu {
            return redraw;
        }

        if let Some(id) = &ctx.ui.focus {
            if let Some(widget) = self.get_widget_abs(None, Some(&id.uuid)) {
                let event = TheEvent::MouseDragged(widget.dim().to_local(coord));
                redraw = widget.on_event(&event, ctx);
                self.process_events(ctx);
            }
        } else if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
            let event = TheEvent::MouseDragged(widget.dim().to_local(coord));
            redraw = widget.on_event(&event, ctx);
            self.process_events(ctx);
        }

        if let Some(drop) = &mut ctx.ui.drop {
            drop.set_position(coord);
            if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
                let event = TheEvent::DropPreview(widget.dim().to_local(coord), drop.clone());
                redraw = widget.on_event(&event, ctx);
                self.process_events(ctx);
            }
            redraw = true;
        }

        redraw
    }

    pub fn touch_up(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let coord = vec2i(x as i32, y as i32);

        if let Some(context) = &mut self.context_menu {
            return redraw;
        }

        if let Some(id) = &ctx.ui.focus {
            if let Some(widget) = self.get_widget_abs(Some(&id.name), Some(&id.uuid)) {
                let event = TheEvent::MouseUp(widget.dim().to_local(coord));
                redraw = widget.on_event(&event, ctx);
                self.process_events(ctx);
            }
        } else if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
            let event = TheEvent::MouseUp(widget.dim().to_local(coord));
            redraw = widget.on_event(&event, ctx);
            self.process_events(ctx);
        }

        if let Some(drop) = &ctx.ui.drop {
            if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
                let event = TheEvent::Drop(widget.dim().to_local(coord), drop.clone());
                redraw = widget.on_event(&event, ctx);
                self.process_events(ctx);
            }
            redraw = true;
        }

        ctx.ui.clear_drop();
        redraw
    }

    pub fn hover(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let coord = vec2i(x as i32, y as i32);

        if let Some(context) = &mut self.context_menu {
            if context.dim.contains(coord) {
                let event = TheEvent::Hover(context.dim.to_local(coord));
                redraw = context.on_event(&event, ctx);
            }
            return redraw;
        }

        if let Some(widget) = self.get_widget_at_coord(coord) {
            //println!("Hover {:?}", widget.id());
            let event = TheEvent::Hover(widget.dim().to_local(coord));
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

    pub fn mouse_wheel(&mut self, delta: (i32, i32), ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        if let Some(id) = &ctx.ui.hover {
            if let Some(widget) = self.get_widget_abs(Some(&id.name), Some(&id.uuid)) {
                redraw = widget.on_event(&TheEvent::MouseWheel(vec2i(delta.0, delta.1)), ctx);
                self.process_events(ctx);
            }
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
        let event = if let Some(c) = char {
            TheEvent::KeyDown(TheValue::Char(c))
        } else {
            TheEvent::KeyCodeDown(TheValue::KeyCode(key.unwrap()))
        };
        ctx.ui.send(event.clone());
        if let Some(id) = &ctx.ui.focus {
            if let Some(widget) = self.get_widget_abs(Some(&id.name), Some(&id.uuid)) {
                redraw = widget.on_event(&event, ctx);
                self.process_events(ctx);
            }
        }
        redraw
    }

    pub fn modifier_changed(
        &mut self,
        shift: bool,
        ctrl: bool,
        alt: bool,
        logo: bool,
        ctx: &mut TheContext,
    ) -> bool {
        let mut redraw = false;
        if let Some(id) = &ctx.ui.focus {
            if let Some(widget) = self.get_widget_abs(Some(&id.name), Some(&id.uuid)) {
                let event = TheEvent::ModifierChanged(shift, ctrl, alt, logo);
                redraw = widget.on_event(&event, ctx);
                self.process_events(ctx);
            }
        }
        if ctx.ui.focus != ctx.ui.hover {
            if let Some(id) = &ctx.ui.hover {
                if let Some(widget) = self.get_widget_abs(Some(&id.name), Some(&id.uuid)) {
                    let event = TheEvent::ModifierChanged(shift, ctrl, alt, logo);
                    redraw = widget.on_event(&event, ctx);
                    self.process_events(ctx);
                }
            }
        }
        redraw
    }

    /// Returns the absolute widget at the given position.
    pub fn get_widget_at_coord(&mut self, coord: Vec2i) -> Option<&mut Box<dyn TheWidget>> {
        if let Some(dialog) = &mut self.dialog {
            if let Some(widget) = dialog.get_widget_at_coord(coord) {
                return Some(widget);
            }
        } else if let Some(widget) = self.canvas.get_widget_at_coord(coord) {
            return Some(widget);
        }
        None
    }

    pub fn get_widget_abs(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>> {
        if let Some(dialog) = &mut self.dialog {
            dialog.get_widget(name, uuid)
        } else {
            self.canvas.get_widget(name, uuid)
        }
    }

    /// Gets a given widget by name
    pub fn get_widget(&mut self, name: &str) -> Option<&mut Box<dyn TheWidget>> {
        self.canvas.get_widget(Some(&name.to_string()), None)
    }

    /// Gets a given widget by id
    pub fn get_widget_id(&mut self, id: Uuid) -> Option<&mut Box<dyn TheWidget>> {
        self.canvas.get_widget(None, Some(&id))
    }

    /// Gets a given text line edit by name
    pub fn get_text_line_edit(&mut self, name: &str) -> Option<&mut dyn TheTextLineEditTrait> {
        if let Some(text_line_edit) = self.canvas.get_widget(Some(&name.to_string()), None) {
            return text_line_edit.as_text_line_edit();
        }
        None
    }

    /// Gets a given icon view by name
    pub fn get_icon_view(&mut self, name: &str) -> Option<&mut dyn TheIconViewTrait> {
        if let Some(text_line_edit) = self.canvas.get_widget(Some(&name.to_string()), None) {
            return text_line_edit.as_icon_view();
        }
        None
    }

    /// Gets a given text by name
    pub fn get_text(&mut self, name: &str) -> Option<&mut dyn TheTextTrait> {
        if let Some(text) = self.canvas.get_widget(Some(&name.to_string()), None) {
            return text.as_text();
        }
        None
    }

    /// Gets a given group button by name
    pub fn get_group_button(&mut self, name: &str) -> Option<&mut dyn TheGroupButtonTrait> {
        if let Some(text) = self.canvas.get_widget(Some(&name.to_string()), None) {
            return text.as_group_button();
        }
        None
    }

    /// Gets a given statusbar by name
    pub fn get_statusbar(&mut self, name: &str) -> Option<&mut dyn TheStatusbarTrait> {
        if let Some(text) = self.canvas.get_widget(Some(&name.to_string()), None) {
            return text.as_statusbar();
        }
        None
    }

    /// Gets a given drop down menu by name
    pub fn get_drop_down_menu(&mut self, name: &str) -> Option<&mut dyn TheDropdownMenuTrait> {
        if let Some(drop_down_menu) = self.canvas.get_widget(Some(&name.to_string()), None) {
            return drop_down_menu.as_drop_down_menu();
        }
        None
    }

    /// Gets a given layout by name
    pub fn get_layout(&mut self, name: &str) -> Option<&mut Box<dyn TheLayout>> {
        self.canvas.get_layout(Some(&name.to_string()), None)
    }

    /// Relayouts the given layout.
    pub fn relayout_layout(&mut self, name: &str, ctx: &mut TheContext) {
        if let Some(l) = self.canvas.get_layout(Some(&name.to_string()), None) {
            l.relayout(ctx);
        }
    }

    /// Gets a given TheListLayout by name
    pub fn get_list_layout(&mut self, name: &str) -> Option<&mut dyn TheListLayoutTrait> {
        if let Some(text_line_edit) = self.canvas.get_layout(Some(&name.to_string()), None) {
            return text_line_edit.as_list_layout();
        }
        None
    }

    /// Gets a given TheStackLayout by name
    pub fn get_stack_layout(&mut self, name: &str) -> Option<&mut dyn TheStackLayoutTrait> {
        if let Some(text_line_edit) = self.canvas.get_layout(Some(&name.to_string()), None) {
            return text_line_edit.as_stack_layout();
        }
        None
    }

    /// Selects the first item of a list layout.
    pub fn select_first_list_item(&mut self, name: &str, ctx: &mut TheContext) {
        if let Some(layout) = self.get_list_layout(name) {
            layout.select_first_item(ctx);
        }
    }

    /// Gets a given TheRGBALayout by name
    pub fn get_rgba_layout(&mut self, name: &str) -> Option<&mut dyn TheRGBALayoutTrait> {
        if let Some(layout) = self.canvas.get_layout(Some(&name.to_string()), None) {
            return layout.as_rgba_layout();
        }
        None
    }

    /// Gets a given TheSharedLayout by name
    pub fn get_shared_layout(&mut self, name: &str) -> Option<&mut dyn TheSharedLayoutTrait> {
        if let Some(layout) = self.canvas.get_layout(Some(&name.to_string()), None) {
            return layout.as_shared_layout();
        }
        None
    }

    /// Gets a given TheSharedLayout by name
    pub fn get_hlayout(&mut self, name: &str) -> Option<&mut dyn TheHLayoutTrait> {
        if let Some(layout) = self.canvas.get_layout(Some(&name.to_string()), None) {
            return layout.as_hlayout();
        }
        None
    }

    /// Gets a given TheRGBALayout by name
    #[cfg(feature = "code")]
    pub fn get_code_layout(&mut self, name: &str) -> Option<&mut dyn TheCodeLayoutTrait> {
        if let Some(layout) = self.canvas.get_layout(Some(&name.to_string()), None) {
            return layout.as_code_layout();
        }
        None
    }

    /// Set the disabled state of the given widget.
    pub fn set_widget_disabled_state(&mut self, name: &str, ctx: &mut TheContext, disabled: bool) {
        if let Some(widget) = self.canvas.get_widget(Some(&name.to_string()), None) {
            widget.set_disabled(disabled);
            if disabled && widget.id().equals(&ctx.ui.hover) {
                ctx.ui.clear_hover();
            }
            if disabled && widget.id().equals(&ctx.ui.focus) {
                ctx.ui.clear_focus();
            }
        }
    }

    /// Sets the context menu for the widget.
    pub fn set_widget_context_menu(&mut self, name: &str, menu: Option<TheContextMenu>) {
        if let Some(widget) = self.canvas.get_widget(Some(&name.to_string()), None) {
            widget.set_context_menu(menu);
        }
    }

    /// Set the value of the given widget.
    pub fn set_widget_value(&mut self, name: &str, ctx: &mut TheContext, value: TheValue) {
        if let Some(widget) = self.canvas.get_widget(Some(&name.to_string()), None) {
            widget.set_value(value);
        }
    }

    #[cfg(feature = "ui")]
    /// Opens a dialog which will have the canvas as context and the given text as title.
    pub fn show_dialog(
        &mut self,
        text: &str,
        mut canvas: TheCanvas,
        buttons: Vec<TheDialogButtonRole>,
        ctx: &mut TheContext,
    ) {
        self.dialog_text = text.to_string();

        let width = canvas.limiter.get_max_width();
        let mut height = canvas.limiter.get_max_height();

        if !buttons.is_empty() {
            let mut toolbar_hlayout = TheHLayout::new(TheId::empty());
            toolbar_hlayout.set_background_color(Some(TheThemeColors::ListLayoutBackground));
            toolbar_hlayout.limiter_mut().set_max_width(width);
            toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));

            for b in &buttons {
                let mut button = TheTraybarButton::new(TheId::named(b.to_id()));
                button.set_text(b.to_string().to_string());
                toolbar_hlayout.add_widget(Box::new(button));
            }

            toolbar_hlayout.set_reverse_index(Some(buttons.len() as i32));

            let mut toolbar_canvas = TheCanvas::default();
            // toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));
            toolbar_hlayout.limiter_mut().set_max_height(30);
            toolbar_canvas.set_layout(toolbar_hlayout);
            canvas.set_bottom(toolbar_canvas);
        }

        let off_x = (ctx.width as i32 - width) / 2;
        let off_y = (ctx.height as i32 - height) / 2;

        let mut dim = TheDim::new(off_x, off_y, width, height);
        dim.buffer_x = off_x;
        dim.buffer_y = off_y;

        canvas.set_dim(dim, ctx);

        ctx.ui.clear_focus();
        ctx.ui.clear_hover();

        self.dialog = Some(canvas);
    }

    #[cfg(feature = "ui")]
    /// Clears / closes the dialog.
    pub fn clear_dialog(&mut self) {
        self.dialog = None;
    }

    #[cfg(feature = "ui")]
    /// Draws the current dialog.
    pub fn draw_dialog(&mut self, ctx: &mut TheContext) {
        if let Some(dialog_canvas) = &mut self.dialog {
            dialog_canvas.draw(&mut self.style, ctx);

            let width = dialog_canvas.limiter.get_max_width();
            let height = dialog_canvas.limiter.get_max_height();

            // ctx.draw.rect(
            //     self.canvas.buffer.pixels_mut(),
            //     &(
            //         dialog_canvas.dim.buffer_x as usize,
            //         dialog_canvas.dim.buffer_y as usize,
            //         width as usize,
            //         height as usize,
            //     ),
            //     ctx.width,
            //     &BLACK,
            // );

            let mut tuple = dialog_canvas.dim.to_buffer_utuple();

            let window_margin = vec4i(3, 29, 3, 3);

            let mut border_shrinker = TheDimShrinker::zero();
            let mut border_dim = TheDim::new(
                tuple.0 as i32 - window_margin.x,
                tuple.1 as i32 - window_margin.y,
                tuple.2 as i32 + window_margin.x + window_margin.z,
                tuple.3 as i32 + window_margin.y + window_margin.w,
            );
            border_dim.buffer_x = border_dim.x;
            border_dim.buffer_y = border_dim.y;

            tuple = border_dim.to_buffer_utuple();

            ctx.draw.rect_outline(
                self.canvas.buffer.pixels_mut(),
                &tuple,
                ctx.width,
                self.style.theme().color(WindowBorderOuter),
            );

            border_shrinker.shrink(1);
            tuple = border_dim.to_buffer_shrunk_utuple(&border_shrinker);
            ctx.draw.rect_outline(
                self.canvas.buffer.pixels_mut(),
                &tuple,
                ctx.width,
                self.style.theme().color(WindowBorderInner),
            );

            border_shrinker.shrink(1);
            tuple = border_dim.to_buffer_shrunk_utuple(&border_shrinker);
            ctx.draw.rect_outline(
                self.canvas.buffer.pixels_mut(),
                &tuple,
                ctx.width,
                self.style.theme().color(WindowBorderInner),
            );

            // Header

            border_shrinker.shrink(1);
            tuple = border_dim.to_buffer_shrunk_utuple(&border_shrinker);
            ctx.draw.rect(
                self.canvas.buffer.pixels_mut(),
                &(tuple.0, tuple.1, tuple.2, 23),
                ctx.width,
                self.style.theme().color(WindowHeaderBackground),
            );

            ctx.draw.rect(
                self.canvas.buffer.pixels_mut(),
                &(tuple.0, tuple.1 + 23, tuple.2, 1),
                ctx.width,
                self.style.theme().color(WindowHeaderBorder1),
            );

            ctx.draw.rect(
                self.canvas.buffer.pixels_mut(),
                &(tuple.0, tuple.1 + 24, tuple.2, 1),
                ctx.width,
                self.style.theme().color(WindowBorderInner),
            );

            ctx.draw.rect(
                self.canvas.buffer.pixels_mut(),
                &(tuple.0, tuple.1 + 25, tuple.2, 1),
                ctx.width,
                self.style.theme().color(WindowHeaderBorder2),
            );

            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    self.canvas.buffer.pixels_mut(),
                    &(tuple.0 + 13, tuple.1, tuple.2 - 13, 23),
                    ctx.width,
                    font,
                    15.0,
                    &self.dialog_text,
                    &WHITE,
                    TheHorizontalAlign::Left,
                    TheVerticalAlign::Center,
                );
            }

            self.canvas.buffer.copy_into(
                dialog_canvas.dim.buffer_x,
                dialog_canvas.dim.buffer_y,
                &dialog_canvas.buffer,
            );
        }
    }
}
