use crate::prelude::*;

pub struct ThePalettePicker {
    id: TheId,
    limiter: TheSizeLimiter,

    is_dirty: bool,

    palette: ThePalette,
    index: usize,

    border_color: Option<RGBA>,

    dim: TheDim,
}

impl TheWidget for ThePalettePicker {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_size(vec2i(200, 200));

        Self {
            id,
            limiter,

            is_dirty: true,

            palette: ThePalette::default(),
            index: 0,

            border_color: None,

            dim: TheDim::zero(),
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(_coord) => {
                ctx.ui
                    .send_widget_state_changed(self.id(), TheWidgetState::Clicked);
                self.is_dirty = true;
                redraw = true;
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

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        _style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        let stride: usize = buffer.stride();

        if !self.dim().is_valid() {
            return;
        }

        let utuple = self.dim.to_buffer_utuple();

        self.is_dirty = false;
    }

    fn as_palette_picker(&mut self) -> Option<&mut dyn ThePalettePickerTrait> {
        Some(self)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait ThePalettePickerTrait {
    fn set_palette(&mut self, palette: ThePalette);
}

impl ThePalettePickerTrait for ThePalettePicker {
    fn set_palette(&mut self, palette: ThePalette) {
        self.palette = palette;
    }
}
