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
    pub name: String,
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
            name: "".to_string(),
            id: TheId::empty(),

            items: vec![],
            width: 160,
            item_height: 21,

            dim: TheDim::zero(),

            hovered: None,
        }
    }

    pub fn named(name: String) -> Self {
        Self {
            name,
            id: TheId::empty(),

            items: vec![],
            width: 160,
            item_height: 23,

            dim: TheDim::zero(),

            hovered: None,
        }
    }

    /// Add an item,
    pub fn add(&mut self, item: TheContextMenuItem) {
        self.items.push(item);
    }

    /// Add a separator.
    pub fn add_separator(&mut self) {
        self.items
            .push(TheContextMenuItem::new("".to_string(), TheId::empty()));
    }

    /// Sets the position of the context menu while making it sure it fits on the screen.
    pub fn set_position(&mut self, position: Vec2i, _ctx: &mut TheContext) {
        let mut height = 2 * 8; // Borders
        for item in self.items.iter() {
            if item.name.is_empty() {
                height += self.item_height / 2;
            } else {
                height += self.item_height;
            }
        }
        self.dim = TheDim::new(position.x, position.y, self.width, height);
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

        let mut y = tuple.1 + 7;
        for item in self.items.iter_mut() {
            let rect = (
                tuple.0,
                y,
                self.width as usize - 2,
                if item.name.is_empty() {
                    self.item_height as usize / 2
                } else {
                    self.item_height as usize
                },
            );

            let mut text_color = style.theme().color(ContextMenuTextNormal);
            if Some(item.id.clone()) == self.hovered && !item.name.is_empty() {
                ctx.draw.rect(
                    pixels,
                    &rect,
                    ctx.width,
                    style.theme().color(ContextMenuHighlight),
                );
                text_color = style.theme().color(ContextMenuTextHighlight);
            }

            if item.name.is_empty() {
                ctx.draw.rect(
                    pixels,
                    &(rect.0, rect.1 + rect.3 / 2, rect.2, 1),
                    ctx.width,
                    style.theme().color(ContextMenuSeparator),
                );
            } else if let Some(font) = &ctx.ui.font {
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
            y += rect.3;
        }
    }
}
