use crate::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum TheRGBAViewMode {
    Display,
    TileSelection,
    TileEditor,
}

pub struct TheCodeView {
    id: TheId,
    limiter: TheSizeLimiter,

    state: TheWidgetState,
    background: RGBA,

    code_grid: TheCodeGrid,
    grid_size: i32,

    buffer: TheRGBABuffer,

    scroll_offset: Vec2i,
    zoom: f32,

    grid: Option<i32>,
    grid_color: RGBA,
    selected: Option<(u32, u32)>,
    selection_color: RGBA,

    dim: TheDim,
    code_is_dirty: bool,
    is_dirty: bool,

    layout_id: TheId,
}

impl TheWidget for TheCodeView {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_height(17);

        Self {
            id,
            limiter,

            state: TheWidgetState::None,
            background: BLACK,

            buffer: TheRGBABuffer::empty(),

            code_grid: TheCodeGrid::new(),
            grid_size: 70,

            scroll_offset: vec2i(0, 0),
            zoom: 1.0,

            grid: None,
            grid_color: [200, 200, 200, 255],
            selected: None,
            selection_color: [255, 255, 255, 180],

            dim: TheDim::zero(),
            code_is_dirty: true,
            is_dirty: true,

            layout_id: TheId::empty(),
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        //println!("event ({}): {:?}", self.id.name, event);
        match event {
            TheEvent::MouseDown(coord) => {
                ctx.ui.set_focus(self.id());

                self.selected = self.get_code_grid_offset(*coord);
                self.code_is_dirty = true;
                ctx.ui.send(TheEvent::CodeEditorSelectionChanged(
                    self.id().clone(),
                    self.selected,
                ));

                redraw = true;
            }
            TheEvent::Hover(_coord) => {
                if self.state != TheWidgetState::Selected && !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }
            }
            TheEvent::KeyCodeDown(code) => {
                if let Some(code) = code.to_key_code() {
                    if code == TheKeyCode::Return {
                        ctx.ui.send(TheEvent::CodeEditorApply(self.id.clone()));
                        redraw = true;
                    }
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
        }
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty | self.code_is_dirty
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
        if !self.dim().is_valid() || !self.buffer.dim().is_valid() {
            return;
        }

        // --- Draw the code grid into the buffer

        let stride: usize = self.buffer.stride();

        if self.code_is_dirty {
            let grid_x = 10;
            let grid_y = 10;

            let background = *style.theme().color(CodeGridBackground);
            let normal = *style.theme().color(CodeGridNormal);
            let selected = *style.theme().color(CodeGridSelected);
            let text_color = *style.theme().color(CodeGridText);

            let utuple = self.buffer.dim().to_buffer_utuple();
            ctx.draw
                .rect(self.buffer.pixels_mut(), &utuple, stride, &background);

            for y in 0..grid_y {
                for x in 0..grid_x {
                    let color = if Some((x, y)) == self.selected {
                        selected
                    } else {
                        normal
                    };

                    let rect = (
                        (x * self.grid_size as u32) as usize + 2,
                        (y * self.grid_size as u32) as usize + 2,
                        self.grid_size as usize - 4,
                        self.grid_size as usize - 4,
                    );

                    if let Some(atom) = self.code_grid.code.get(&(x, y)) {
                        let text = atom.describe();

                        //let back_color = [60, 60, 60, 255];

                        //ctx.draw
                        //    .rect(self.buffer.pixels_mut(), &rect, stride, &[60, 60, 60, 255]);

                        ctx.draw.rect_outline_border(
                            self.buffer.pixels_mut(),
                            &rect,
                            stride,
                            &color,
                            1,
                        );

                        ctx.draw.rect(
                            self.buffer.pixels_mut(),
                            &(rect.0 + 1, rect.1 + 1, rect.2 - 2, rect.3 - 2),
                            stride,
                            &color,
                        );

                        if let Some(font) = &ctx.ui.font {
                            ctx.draw.text_rect_blend(
                                self.buffer.pixels_mut(),
                                &rect,
                                stride,
                                font,
                                13.0,
                                &text,
                                &text_color,
                                TheHorizontalAlign::Center,
                                TheVerticalAlign::Center,
                            );

                            match atom {
                                TheAtom::Variable(_) => {
                                    if x == 0 {
                                        ctx.draw.text_rect_blend(
                                            self.buffer.pixels_mut(),
                                            &(rect.0, rect.1, rect.2 - 10, rect.3 - 2),
                                            stride,
                                            font,
                                            13.0,
                                            &"=",
                                            &text_color,
                                            TheHorizontalAlign::Right,
                                            TheVerticalAlign::Bottom,
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else {
                        ctx.draw.rect_outline_border(
                            self.buffer.pixels_mut(),
                            &rect,
                            stride,
                            &color,
                            0,
                        );
                    }
                }
            }

            self.code_is_dirty = false;
        }

        // ---

        pub fn _mix_color(a: &[u8; 4], b: &[u8; 4], v: f32) -> [u8; 4] {
            [
                (((1.0 - v) * (a[0] as f32 / 255.0) + b[0] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[1] as f32 / 255.0) + b[1] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[2] as f32 / 255.0) + b[2] as f32 / 255.0 * v) * 255.0) as u8,
                255,
            ]
        }

        let stride: usize = buffer.stride();

        if !self.buffer.is_valid() {
            ctx.draw.rect(
                buffer.pixels_mut(),
                &self.dim.to_buffer_utuple(),
                stride,
                &self.background,
            );
            return;
        }

        let target = buffer;

        let src_width = self.buffer.dim().width as f32;
        let src_height = self.buffer.dim().height as f32;
        let target_width = self.dim().width as f32;
        let target_height = self.dim().height as f32;

        // Calculate the scaled dimensions of the source image
        let scaled_width = src_width * self.zoom;
        let scaled_height = src_height * self.zoom;

        // Calculate the offset to center the image
        let offset_x = if scaled_width < target_width {
            0.0 //(target_width - scaled_width) / 2.0
        } else {
            -self.scroll_offset.x as f32
        };

        let offset_y = if scaled_height < target_height {
            (target_height - scaled_height) / 2.0
        } else {
            -self.scroll_offset.y as f32
        };

        // Loop over every pixel in the target buffer
        for target_y in 0..self.dim.height {
            for target_x in 0..self.dim.width {
                // Calculate the corresponding source coordinates with the offset
                let src_x = (target_x as f32 - offset_x) / self.zoom;
                let src_y = (target_y as f32 - offset_y) / self.zoom;

                // Calculate the index for the destination pixel
                let target_index = ((self.dim.buffer_y + target_y) * target.dim().width
                    + target_x
                    + self.dim.buffer_x) as usize
                    * 4;

                if let Some(grid) = self.grid {
                    if src_x as i32 % grid == 0 || src_y as i32 % grid == 0 {
                        target.pixels_mut()[target_index..target_index + 4]
                            .copy_from_slice(&self.grid_color);
                        continue;
                    }
                }

                if src_x >= 0.0 && src_x < src_width && src_y >= 0.0 && src_y < src_height {
                    // Perform nearest neighbor interpolation
                    let src_x = src_x as i32;
                    let src_y = src_y as i32;
                    let src_index = (src_y * self.buffer.stride() as i32 + src_x) as usize * 4;

                    // Copy the pixel from the source buffer to the target buffer
                    target.pixels_mut()[target_index..target_index + 4]
                        .copy_from_slice(&self.buffer.pixels()[src_index..src_index + 4]);
                } else {
                    // Set the pixel to black if it's out of the source bounds
                    target.pixels_mut()[target_index..target_index + 4].fill(0);
                }
            }
        }

        self.is_dirty = false;
    }

    fn as_code_view(&mut self) -> Option<&mut dyn TheCodeViewTrait> {
        Some(self)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheCodeViewTrait {
    fn adjust_buffer_to_grid(&mut self);

    fn buffer(&self) -> &TheRGBABuffer;
    fn buffer_mut(&mut self) -> &mut TheRGBABuffer;
    fn code_grid(&self) -> TheCodeGrid;
    fn set_code_grid(&mut self, code_grid: TheCodeGrid);
    fn set_background(&mut self, color: RGBA);
    fn zoom(&self) -> f32;
    fn set_zoom(&mut self, zoom: f32);
    fn set_scroll_offset(&mut self, offset: Vec2i);
    fn set_grid(&mut self, grid: Option<i32>);
    fn set_grid_color(&mut self, color: RGBA);
    fn set_selection_color(&mut self, color: RGBA);

    fn set_associated_layout(&mut self, id: TheId);

    fn selected(&self) -> Option<(u32, u32)>;

    fn set_atom_value(&mut self, coord: (u32, u32), name: String, value: TheValue);
    fn set_grid_atom(&mut self, coord: (u32, u32), atom: TheAtom);
    fn get_code_grid_offset(&self, coord: Vec2i) -> Option<(u32, u32)>;
}

impl TheCodeViewTrait for TheCodeView {
    fn adjust_buffer_to_grid(&mut self) {
        let grid_x = 10;
        let grid_y = 10;

        let d = self.buffer().dim();
        if d.width != grid_x * self.grid_size || d.height != grid_y * self.grid_size {
            let b = TheRGBABuffer::new(TheDim::new(
                0,
                0,
                grid_x * self.grid_size,
                grid_y * self.grid_size,
            ));
            self.buffer = b;
        }
    }
    fn buffer(&self) -> &TheRGBABuffer {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut TheRGBABuffer {
        &mut self.buffer
    }
    fn code_grid(&self) -> TheCodeGrid {
        self.code_grid.clone()
    }
    fn set_code_grid(&mut self, code_grid: TheCodeGrid) {
        self.code_grid = code_grid;
        self.code_is_dirty = true;
        self.is_dirty = true;
    }
    fn set_background(&mut self, color: RGBA) {
        self.background = color;
    }
    fn zoom(&self) -> f32 {
        self.zoom
    }
    fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }
    fn set_scroll_offset(&mut self, offset: Vec2i) {
        self.scroll_offset = offset;
    }
    fn set_associated_layout(&mut self, layout_id: TheId) {
        self.layout_id = layout_id;
    }
    fn set_grid(&mut self, grid: Option<i32>) {
        self.grid = grid;
    }
    fn set_grid_color(&mut self, color: RGBA) {
        self.grid_color = color;
    }
    fn set_selection_color(&mut self, color: RGBA) {
        self.selection_color = color;
        self.is_dirty = true;
    }
    fn selected(&self) -> Option<(u32, u32)> {
        self.selected
    }
    fn set_atom_value(&mut self, coord: (u32, u32), name: String, value: TheValue) {
        if let Some(atom) = self.code_grid.code.get_mut(&coord) {
            atom.process_value_change(name, value);
            self.code_is_dirty = true;
            self.is_dirty = true;
        }
    }
    fn set_grid_atom(&mut self, coord: (u32, u32), atom: TheAtom) {
        self.code_grid.code.insert(coord, atom);
        self.code_is_dirty = true;
        self.is_dirty = true;
    }
    fn get_code_grid_offset(&self, coord: Vec2i) -> Option<(u32, u32)> {
        let centered_offset_x = 0.0;
        // if (self.zoom * self.buffer.dim().width as f32) < self.dim.width as f32 {
        //     (self.dim.width as f32 - self.zoom * self.buffer.dim().width as f32) / 2.0
        // } else {
        //     0.0
        // };
        let centered_offset_y =
            if (self.zoom * self.buffer.dim().height as f32) < self.dim.height as f32 {
                (self.dim.height as f32 - self.zoom * self.buffer.dim().height as f32) / 2.0
            } else {
                0.0
            };

        let source_x = ((coord.x as f32 - centered_offset_x) / self.zoom
            + self.scroll_offset.x as f32)
            .round() as i32;
        let source_y = ((coord.y as f32 - centered_offset_y) / self.zoom
            + self.scroll_offset.y as f32)
            .round() as i32;

        if source_x >= 0
            && source_x < self.buffer.dim().width
            && source_y >= 0
            && source_y < self.buffer.dim().height
        {
            let grid_x = source_x / self.grid_size;
            let grid_y = source_y / self.grid_size;

            if grid_x * self.grid_size < self.buffer.dim().width
                && grid_y * self.grid_size < self.buffer.dim().height
            {
                return Some((grid_x as u32, grid_y as u32));
            }
        }
        None
    }
}
