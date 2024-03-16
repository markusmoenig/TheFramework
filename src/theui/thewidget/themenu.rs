use crate::prelude::*;

pub struct TheMenu {
    id: TheId,

    limiter: TheSizeLimiter,

    menus: Vec<TheContextMenu>,
    menus_text: Vec<TheDim>,

    hovered: Option<usize>,
    selected: Option<usize>,

    dim: TheDim,
    is_dirty: bool,
}

impl TheWidget for TheMenu {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(22);

        Self {
            id,
            limiter,

            dim: TheDim::zero(),
            is_dirty: false,

            menus: Vec::new(),
            menus_text: Vec::new(),

            hovered: None,
            selected: None,
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        match event {
            TheEvent::MouseDown(_coord) => {
                if self.hovered.is_some() {
                    self.selected = self.hovered;
                    if let Some(selected) = self.selected {
                        ctx.ui.send(TheEvent::ShowMenu(
                            self.id().clone(),
                            vec2i(
                                self.menus_text[selected].x,
                                self.menus_text[selected].y + 22,
                            ),
                            self.menus[selected].clone(),
                        ));
                    }
                    redraw = true;
                }
            }
            TheEvent::Hover(coord) => {
                ctx.ui.set_hover(self.id());
                let old = self.hovered;
                self.hovered = None;
                for i in 0..self.menus.len() {
                    if self.menus_text[i].contains(*coord) {
                        self.hovered = Some(i);
                        if self.selected.is_some() && self.selected != Some(i) {
                            self.selected = self.hovered;
                            if let Some(selected) = self.selected {
                                ctx.ui.send(TheEvent::ShowMenu(
                                    self.id().clone(),
                                    vec2i(
                                        self.menus_text[selected].x,
                                        self.menus_text[selected].y + 22,
                                    ),
                                    self.menus[selected].clone(),
                                ));
                            }
                        }
                    }
                }

                if self.hovered != old {
                    redraw = true;
                    self.is_dirty = true;
                }
            }
            TheEvent::ContextMenuClosed(id) => {
                if *id == self.id {
                    self.selected = None;
                    self.hovered = None;
                }
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

            self.menus_text.clear();

            let mut x = 10;
            let width = 50;

            for _ in 0..self.menus.len() {
                let dim = TheDim::new(x, 3, width, 18);
                self.menus_text.push(dim);
                x += width
            }
        }
    }

    fn supports_hover(&mut self) -> bool {
        true
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

        if let Some(icon) = ctx.ui.icon("dark_menu") {
            for x in 0..utuple.2 {
                let r = (utuple.0 + x, utuple.1, 1, icon.dim().height as usize);
                ctx.draw
                    .copy_slice_3(buffer.pixels_mut(), icon.pixels(), &r, stride);
            }
        }

        let r = self.dim.to_buffer_utuple();
        for i in 0..self.menus.len() {
            let tr = self.menus_text[i];
            let rect = (
                r.0 + tr.x as usize,
                r.1 + tr.y as usize,
                tr.width as usize,
                tr.height as usize,
            );

            if self.hovered == Some(i) {
                ctx.draw.rounded_rect(
                    buffer.pixels_mut(),
                    &(rect.0, rect.1 - 2, rect.2, rect.3 + 2),
                    stride,
                    style.theme().color(MenuHover),
                    &(2.0, 2.0, 2.0, 2.0),
                );
            }

            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    buffer.pixels_mut(),
                    &rect,
                    stride,
                    font,
                    14.0,
                    &self.menus[i].name,
                    if self.hovered == Some(i) {
                        style.theme().color(MenuTextHighlighted)
                    } else {
                        style.theme().color(MenuText)
                    },
                    TheHorizontalAlign::Center,
                    TheVerticalAlign::Center,
                );
            }
        }

        self.is_dirty = false;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheMenuTrait {
    fn add_context_menu(&mut self, context_menu: TheContextMenu);
}

impl TheMenuTrait for TheMenu {
    fn add_context_menu(&mut self, context_menu: TheContextMenu) {
        self.menus.push(context_menu);
    }
}
