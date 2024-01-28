use crate::prelude::*;

// Item

#[derive(Clone, Debug)]
pub struct TheContextMenuItem {
    name: String,
    pub id: TheId,
    pub value: Option<TheValue>,
    pub disabled: bool,
}

impl TheContextMenuItem {
    pub fn new(name: String, id: TheId) -> Self {
        Self {
            name,
            id,
            value: None,
            disabled: false,
        }
    }
}

// Menu

#[derive(Clone, Debug)]
pub struct TheContextMenu {
    pub id: TheId,
    pub items: Vec<TheContextMenuItem>,
    pub width: i32,
    pub item_height: i32,

    pub dim: TheDim,

    pub hovered: Option<TheId>,
}

impl Default for TheContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl TheContextMenu {
    pub fn new() -> Self {
        Self {
            id: TheId::empty(),

            items: vec![],
            width: 160,
            item_height: 21,

            dim: TheDim::zero(),

            hovered: None,
        }
    }

    /// Add an item,
    pub fn add(&mut self, item: TheContextMenuItem) {
        self.items.push(item);
    }

    /// Sets the position of the context menu while making it sure it fits on the screen.
    pub fn set_position(&mut self, position: Vec2i, _ctx: &mut TheContext) {
        self.dim = TheDim::new(
            position.x,
            position.y,
            self.width,
            self.items.len() as i32 * self.item_height + 2 * 8,
        );
        self.dim.buffer_x = position.x;
        self.dim.buffer_y = position.y;
    }

    pub fn on_event(&mut self, event: &TheEvent, _ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        match event {
            TheEvent::MouseDown(_coord) => {
                if self.hovered.is_some() {
                    redraw = true;
                }
            }
            TheEvent::Hover(coord) => {
                if coord.y >= 7 && coord.y < self.dim.height - 7 {
                    let index = (coord.y - 7) / self.item_height;
                    if index < self.items.len() as i32 {
                        self.hovered = Some(self.items[index as usize].id.clone());
                    } else {
                        self.hovered = None;
                    }
                } else {
                    self.hovered = None;
                }
                redraw = true;
            }
            _ => {}
        }

        redraw
    }

    /// Draw the menu
    pub fn draw(&mut self, pixels: &mut [u8], style: &mut Box<dyn TheStyle>, ctx: &mut TheContext) {
        let mut tuple = self.dim.to_buffer_utuple();
        let mut shrinker = TheDimShrinker::zero();

        ctx.draw.rect_outline(
            pixels,
            &tuple,
            ctx.width,
            style.theme().color(ContextMenuBorder),
        );

        shrinker.shrink(1);
        tuple = self.dim.to_buffer_shrunk_utuple(&shrinker);

        ctx.draw.rect(
            pixels,
            &tuple,
            ctx.width,
            style.theme().color(ContextMenuBackground),
        );

        for (i, item) in self.items.iter_mut().enumerate() {
            let rect = (
                tuple.0,
                tuple.1 + 7 + i * self.item_height as usize,
                self.width as usize - 2,
                self.item_height as usize,
            );

            let mut text_color = style.theme().color(ContextMenuTextNormal);
            if Some(item.id.clone()) == self.hovered {
                ctx.draw.rect(
                    pixels,
                    &rect,
                    ctx.width,
                    style.theme().color(ContextMenuHighlight),
                );
                text_color = style.theme().color(ContextMenuTextHighlight);
            }

            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    pixels,
                    &(rect.0 + 16, rect.1, &rect.2 - 16, rect.3),
                    ctx.width,
                    font,
                    13.5,
                    &item.name,
                    text_color,
                    TheHorizontalAlign::Left,
                    TheVerticalAlign::Center,
                );
            }
        }
    }
}
