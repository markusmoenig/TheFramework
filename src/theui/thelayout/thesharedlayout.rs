use crate::prelude::*;

/// The layout mode.
#[derive(PartialEq, Clone, Debug)]
pub enum TheSharedLayoutMode {
    ///
    Left,
    ////
    Shared,
    ///
    Right,
}

pub struct TheSharedLayout {
    id: TheId,
    limiter: TheSizeLimiter,

    mode: TheSharedLayoutMode,
    dim: TheDim,

    margin: Vec4i,
    padding: i32,

    canvas: Vec<TheCanvas>,
    widgets: Vec<Box<dyn TheWidget>>,

    background: Option<TheThemeColors>,
}

impl TheLayout for TheSharedLayout {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        Self {
            id,
            limiter: TheSizeLimiter::new(),
            mode: TheSharedLayoutMode::Left,

            dim: TheDim::zero(),

            canvas: vec![],
            widgets: vec![],

            margin: vec4i(10, 10, 10, 10),
            padding: 5,

            background: Some(DefaultWidgetBackground),
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
        if self.canvas.len() < 2 {
            return None;
        }

        if self.mode == TheSharedLayoutMode::Left {
            return self.canvas[0].get_widget_at_coord(coord);
        } else if self.mode == TheSharedLayoutMode::Right {
            return self.canvas[1].get_widget_at_coord(coord);
        } else {
            for c in &mut self.canvas {
                if let Some(w) = c.get_widget_at_coord(coord) {
                    return Some(w);
                }
            }
        }
        None
    }

    fn get_widget(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheWidget>> {
        if self.canvas.len() < 2 {
            return None;
        }

        if self.mode == TheSharedLayoutMode::Left {
            return self.canvas[0].get_widget(name, uuid);
        } else if self.mode == TheSharedLayoutMode::Right {
            return self.canvas[1].get_widget(name, uuid);
        } else {
            for c in &mut self.canvas {
                if let Some(w) = c.get_widget(name, uuid) {
                    return Some(w);
                }
            }
        }
        None
    }

    fn get_layout(
        &mut self,
        name: Option<&String>,
        uuid: Option<&Uuid>,
    ) -> Option<&mut Box<dyn TheLayout>> {
        if self.canvas.len() < 2 {
            return None;
        }
        if self.mode == TheSharedLayoutMode::Left {
            return self.canvas[0].get_layout(name, uuid);
        } else if self.mode == TheSharedLayoutMode::Right {
            return self.canvas[1].get_layout(name, uuid);
        } else {
            for c in &mut self.canvas {
                if let Some(w) = c.get_layout(name, uuid) {
                    return Some(w);
                }
            }
        }
        None
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

            if self.canvas.len() < 2 {
                return;
            }

            if self.mode == TheSharedLayoutMode::Left {
                self.canvas[0].set_dim(dim, ctx);
            } else if self.mode == TheSharedLayoutMode::Right {
                self.canvas[1].set_dim(dim, ctx);
            } else {
                self.canvas[0].set_dim(TheDim::new(dim.x, dim.y, dim.width / 2, dim.height), ctx);
                self.canvas[1].set_dim(
                    TheDim::new(
                        dim.x + dim.width / 2 + 1,
                        dim.y,
                        dim.width / 2 - 1,
                        dim.height,
                    ),
                    ctx,
                );
            }
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
        if self.canvas.len() < 2 {
            return;
        }

        if let Some(background) = self.background {
            let stride = buffer.stride();

            ctx.draw.rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_utuple(),
                stride,
                style.theme().color(background),
            );
        }

        if self.mode == TheSharedLayoutMode::Left {
            self.canvas[0].draw(style, ctx);

            buffer.copy_into(
                self.dim.buffer_x,
                self.dim.buffer_y,
                self.canvas[0].buffer(),
            );
        } else if self.mode == TheSharedLayoutMode::Right {
            self.canvas[1].draw(style, ctx);
            buffer.copy_into(
                self.dim.buffer_x,
                self.dim.buffer_y,
                self.canvas[1].buffer(),
            );
        } else {
            self.canvas[0].draw(style, ctx);

            buffer.copy_into(
                self.dim.buffer_x,
                self.dim.buffer_y,
                self.canvas[0].buffer(),
            );

            self.canvas[1].draw(style, ctx);
            buffer.copy_into(
                self.dim.buffer_x + self.dim.width / 2 + 1,
                self.dim.buffer_y,
                self.canvas[1].buffer(),
            );
        }
    }

    fn as_shared_layout(&mut self) -> Option<&mut dyn TheSharedLayoutTrait> {
        Some(self)
    }
}

/// TheHLayout specific functions.
pub trait TheSharedLayoutTrait {
    /// Add a canvas.
    fn add_canvas(&mut self, canvas: TheCanvas);
    /// Set the layout mode.
    fn set_mode(&mut self, mode: TheSharedLayoutMode);
}

impl TheSharedLayoutTrait for TheSharedLayout {
    fn add_canvas(&mut self, canvas: TheCanvas) {
        self.canvas.push(canvas);
    }
    fn set_mode(&mut self, mode: TheSharedLayoutMode) {
        self.mode = mode;
    }
}
