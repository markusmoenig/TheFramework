use crate::prelude::*;

pub struct TheTabbar {
    widget_id: TheId,
    limiter: TheSizeLimiter,

    state: TheWidgetState,

    tabs: Vec<String>,
    selected: i32,
    original: i32,

    selected_index: Option<i32>,
    hover_index: Option<i32>,

    dim: TheDim,
    is_dirty: bool,
}

impl TheWidget for TheTabbar {
    fn new(name: String) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(22);

        Self {
            widget_id: TheId::new(name),
            limiter,

            state: TheWidgetState::None,

            tabs: vec![],
            selected: 0,
            original: 0,

            selected_index: Some(0),
            hover_index: None,

            dim: TheDim::zero(),
            is_dirty: false,
        }
    }

    fn id(&self) -> &TheId {
        &self.widget_id
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(coord) => {
                self.is_dirty = true;
                if self.state != TheWidgetState::Selected {
                    self.state = TheWidgetState::Selected;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                    ctx.ui.set_focus(self.id());
                    self.original = self.selected;
                    redraw = true;
                }
                if let Some(coord) = coord.to_vec2i() {
                    let index = coord.x / 142;
                    if index >= 0 && index < self.tabs.len() as i32 {
                        if Some(index) != self.selected_index {
                            self.selected_index = Some(index);
                            redraw = true;
                            self.is_dirty = true;
                        }
                    } else if self.selected_index.is_some() {
                        self.selected_index = None;
                        redraw = true;
                        self.is_dirty = true;
                    }
                }
            }
            TheEvent::Hover(coord) => {
                if !self.id().equals(&ctx.ui.hover) {
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                    self.is_dirty = true;
                }
                if let Some(coord) = coord.to_vec2i() {
                    let index = coord.x / 142;
                    if index >= 0 && index < self.tabs.len() as i32 {
                        if Some(index) != self.hover_index {
                            self.hover_index = Some(index);
                            redraw = true;
                            self.is_dirty = true;
                        }
                    } else if self.hover_index.is_some() {
                        self.hover_index = None;
                        redraw = true;
                        self.is_dirty = true;
                    }
                }
            }
            TheEvent::LostHover(_id) => {
                self.hover_index = None;
                redraw = true;
                self.is_dirty = true;
            }
            _ => {}
        }
        redraw
    }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim) {
        if self.dim != dim {
            self.dim = dim;
            self.is_dirty = true;
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty
    }

    fn set_needs_redraw(&mut self, redraw: bool) {
        self.is_dirty = redraw;
    }

    fn state(&self) -> TheWidgetState {
        self.state
    }

    fn set_state(&mut self, state: TheWidgetState) {
        self.state = state;
        self.is_dirty = true;
    }

    fn supports_hover(&mut self) -> bool {
        true
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        let stride = buffer.stride();

        let utuple: (usize, usize, usize, usize) = self.dim.to_buffer_utuple();

        ctx.draw.rect(
            buffer.pixels_mut(),
            &utuple,
            stride,
            style.theme().color(TabbarBackground),
        );

        let mut x = 0;

        for (index,text) in self.tabs.iter().enumerate() {

            let mut icon_name = if Some(index as i32) == self.selected_index {
                "dark_tabbar_selected".to_string()
            } else {
                "dark_tabbar_normal".to_string()
            };

            if Some(index as i32) == self.hover_index && icon_name != "dark_tabbar_selected" {
                icon_name = "dark_tabbar_hover".to_string()
            }

            if let Some(icon) = ctx.ui.icon(&icon_name) {
                let r = (
                    utuple.0 + x,
                    utuple.1,
                    icon.dim().width as usize,
                    icon.dim().height as usize,
                );
                ctx.draw
                    .copy_slice_3(buffer.pixels_mut(), icon.pixels(), &r, stride);

                if let Some(font) = &ctx.ui.font {
                    ctx.draw.text_rect_blend(
                        buffer.pixels_mut(),
                        &r,
                        stride,
                        font,
                        12.5,
                        text.as_str(),
                        style.theme().color(TabbarText),
                        TheHorizontalAlign::Center,
                        TheVerticalAlign::Center,
                    );
                }

                x += icon.dim().width as usize;
            }

            if index < self.tabs.len() - 1 {
                // Connector

                let r = (
                    utuple.0 + x,
                    utuple.1 + utuple.3 - 1,
                    2,
                    1,
                );
                ctx.draw.rect(
                    buffer.pixels_mut(),
                    &r,
                    stride,
                    style.theme().color(TabbarConnector),
                );

                x += 2;
            }
        }

        /*
        let utuple: (usize, usize, usize, usize) = self.dim.to_buffer_utuple();

        let mut icon_name = if self.state == TheWidgetState::Clicked {
            "dark_dropdown_clicked".to_string()
        } else {
            "dark_dropdown_normal".to_string()
        };

        if self.state != TheWidgetState::Clicked && self.id().equals(&ctx.ui.hover) {
            icon_name = "dark_dropdown_hover".to_string()
        }
        if self.state != TheWidgetState::Clicked && self.id().equals(&ctx.ui.focus) {
            icon_name = "dark_dropdown_focus".to_string()
        }

        let text_color = if self.state == TheWidgetState::Selected {
            style.theme().color(SectionbarSelectedTextColor)
        } else {
            style.theme().color(SectionbarNormalTextColor)
        };

        if let Some(icon) = ctx.ui.icon(&icon_name) {
            let off = if icon.dim().width == 140 { 1 } else { 0 };
            let r = (
                utuple.0 + off,
                utuple.1 + off,
                icon.dim().width as usize,
                icon.dim().height as usize,
            );
            ctx.draw
                .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
        }

        if let Some(icon) = ctx.ui.icon("dark_dropdown_marker") {
            let r = (
                utuple.0 + 129,
                utuple.1 + 7,
                icon.dim().width as usize,
                icon.dim().height as usize,
            );
            ctx.draw
                .blend_slice(buffer.pixels_mut(), icon.pixels(), &r, stride);
        }

        shrinker.shrink_by(8, 0, 12, 0);

        if !self.tabs.is_empty() {
            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    buffer.pixels_mut(),
                    &self.dim.to_buffer_shrunk_utuple(&shrinker),
                    stride,
                    font,
                    12.5,
                    self.tabs[self.selected as usize].as_str(),
                    text_color,
                    TheHorizontalAlign::Left,
                    TheVerticalAlign::Center,
                );
            }
        }*/

        self.is_dirty = false;
    }

}

pub trait TheTabbarTrait {
    fn add_tab(&mut self, name: String);
    fn selection(&self) -> Option<String>;
    fn set_selection(&mut self, name: String);
}

impl TheTabbarTrait for TheTabbar {
    fn add_tab(&mut self, name: String) {
        self.tabs.push(name);
    }
    fn selection(&self) -> Option<String> {
        if let Some(index) = self.selected_index {
            if index < self.tabs.len() as i32 {
                return Some(self.tabs[index as usize].clone());
            }
        }
        None
    }
    fn set_selection(&mut self, name: String) {
        self.is_dirty = true;
        for (index,text) in self.tabs.iter().enumerate() {
            if name == *text {
                self.selected_index = Some(index as i32);
                return;
            }
        }
        self.selected_index = None;
    }
}
