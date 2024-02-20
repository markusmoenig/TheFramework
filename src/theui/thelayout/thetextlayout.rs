use crate::prelude::*;

pub struct TheTextLayout {
    id: TheId,
    limiter: TheSizeLimiter,

    dim: TheDim,

    text: Vec<String>,
    text_rect: Vec<(usize, usize, usize, usize)>,

    widgets: Vec<Box<dyn TheWidget>>,

    text_size: f32,
    text_margin: i32,

    margin: Vec4i,
    padding: i32,

    background: Option<TheThemeColors>,
}

impl TheLayout for TheTextLayout {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        Self {
            id,
            limiter: TheSizeLimiter::new(),

            dim: TheDim::zero(),

            text: vec![],
            text_rect: vec![],

            widgets: vec![],

            text_size: 13.0,
            text_margin: 10,

            margin: vec4i(10, 10, 10, 10),
            padding: 10,

            background: Some(TextLayoutBackground),
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn set_margin(&mut self, margin: Vec4i) {
        self.margin = margin;
    }

    fn set_padding(&mut self, padding: i32) {
        self.padding = padding;
    }

    fn set_background_color(&mut self, color: Option<TheThemeColors>) {
        self.background = color;
    }

    fn widgets(&mut self) -> &mut Vec<Box<dyn TheWidget>> {
        &mut self.widgets
    }

    fn get_widget_at_coord(&mut self, coord: Vec2i) -> Option<&mut Box<dyn TheWidget>> {
        let widgets = self.widgets();
        widgets.iter_mut().find(|w| w.dim().contains(coord))
    }

    fn get_widget(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>> {
        self.widgets.iter_mut().find(|w| w.id().matches(name, uuid))
    }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim, ctx: &mut TheContext) {
        if self.dim != dim || ctx.ui.relayout {
            self.dim = dim;

            let x = self.margin.x;
            let mut y = self.margin.y;

            let mut text_width = 0;

            for t in &mut self.text {
                if let Some(font) = &ctx.ui.font {
                    let size = if !t.is_empty() {
                        ctx.draw.get_text_size(font, self.text_size, t)
                    } else {
                        (0, 0)
                    };
                    if size.0 > text_width {
                        text_width = size.0;
                    }
                }
            }

            text_width += self.text_margin as usize;

            let mut texts_rect: Vec<(usize, usize, usize, usize)> = vec![];
            let max_width = dim.width - text_width as i32 - self.margin.x - self.margin.z;

            for (index, w) in &mut self.widgets.iter_mut().enumerate() {
                w.calculate_size(ctx);

                let text_is_empty = self.text[index].is_empty();

                let width = w.limiter().get_width(if text_is_empty {
                    max_width + text_width as i32
                } else {
                    max_width
                });
                let height = w.limiter().get_height(dim.height);

                // Limit to visible area
                // if y + height > dim.height {
                //     break;
                // }

                texts_rect.push((
                    (self.dim.buffer_x + x) as usize,
                    (self.dim.buffer_y + y) as usize,
                    text_width,
                    self.text_size as usize,
                ));

                if text_is_empty {
                    let offset = (max_width + text_width as i32 - width) / 2;
                    w.set_dim(TheDim::new(dim.x + x + offset, dim.y + y, width, height));
                    w.dim_mut()
                        .set_buffer_offset(self.dim.buffer_x + x + offset, self.dim.buffer_y + y);
                } else {
                    w.set_dim(TheDim::new(
                        dim.x + x + text_width as i32,
                        dim.y + y,
                        width,
                        height,
                    ));
                    w.dim_mut().set_buffer_offset(
                        self.dim.buffer_x + x + text_width as i32,
                        self.dim.buffer_y + y,
                    );
                }

                y += height + self.padding;
            }

            self.text_rect = texts_rect;
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
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

        let stride: usize = buffer.stride();

        if let Some(background) = self.background {
            ctx.draw.rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_utuple(),
                stride,
                style.theme().color(background),
            );

            ctx.draw.rect_outline(
                buffer.pixels_mut(),
                &self.dim.to_buffer_utuple(),
                stride,
                style.theme().color(TextLayoutBorder),
            );
        }

        for i in 0..self.text.len() {
            if self.text[i].is_empty() {
                continue;
            }
            if let Some(font) = &ctx.ui.font {
                ctx.draw.text_rect_blend(
                    buffer.pixels_mut(),
                    &self.text_rect[i],
                    stride,
                    font,
                    self.text_size,
                    &self.text[i],
                    &WHITE,
                    TheHorizontalAlign::Left,
                    TheVerticalAlign::Top,
                );
            }
        }

        for w in &mut self.widgets {
            w.draw(buffer, style, ctx);
        }
    }

    fn as_text_layout(&mut self) -> Option<&mut dyn TheTextLayoutTrait> {
        Some(self)
    }
}

/// TheTextLayout specific functions.
pub trait TheTextLayoutTrait {
    /// Clear the text and widget pairs.
    fn clear(&mut self);
    /// Add a text / widget pair.
    fn add_pair(&mut self, text: String, widget: Box<dyn TheWidget>);
    /// Set the text size to use for the left handed text.
    fn set_text_size(&mut self, text_size: f32);
    /// Set the text margin between the text and the widget.
    fn set_text_margin(&mut self, text_margin: i32);
}

impl TheTextLayoutTrait for TheTextLayout {
    fn clear(&mut self) {
        self.text.clear();
        self.widgets.clear();
    }
    fn add_pair(&mut self, text: String, widget: Box<dyn TheWidget>) {
        self.text.push(text);
        self.widgets.push(widget);
    }
    fn set_text_size(&mut self, text_size: f32) {
        self.text_size = text_size;
    }
    fn set_text_margin(&mut self, text_margin: i32) {
        self.text_margin = text_margin;
    }
}
