use crate::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum TheRGBAViewMode {
    Display,
    TileSelection,
    TileEditor,
}

pub struct TheRGBAView {
    id: TheId,
    limiter: TheSizeLimiter,

    state: TheWidgetState,
    background: RGBA,

    buffer: TheRGBABuffer,

    scroll_offset: Vec2i,
    zoom: f32,

    grid: Option<i32>,
    grid_color: RGBA,
    selected: FxHashSet<(i32, i32)>,
    selection_color: RGBA,
    hover_color: Option<RGBA>,
    hover: Option<(i32, i32)>,

    hscrollbar: TheId,
    vscrollbar: TheId,
    wheel_scale: f32,

    mode: TheRGBAViewMode,

    dim: TheDim,
    is_dirty: bool,

    layout_id: TheId,
}

impl TheWidget for TheRGBAView {
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
            scroll_offset: vec2i(0, 0),
            zoom: 1.0,

            grid: None,
            grid_color: [200, 200, 200, 255],
            selected: FxHashSet::default(),
            selection_color: [255, 255, 255, 180],
            hover_color: None,
            hover: None,

            hscrollbar: TheId::empty(),
            vscrollbar: TheId::empty(),
            wheel_scale: -0.4,

            mode: TheRGBAViewMode::Display,

            dim: TheDim::zero(),
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
                if self.state != TheWidgetState::Selected {
                    self.is_dirty = true;
                    self.state = TheWidgetState::Selected;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                    ctx.ui.send(TheEvent::NewListItemSelected(
                        self.id().clone(),
                        self.layout_id.clone(),
                    ));
                    ctx.ui.set_focus(self.id());
                    redraw = true;
                }

                if self.mode != TheRGBAViewMode::Display {
                    if let Some(grid) = self.grid {
                        let centered_offset_x = if (self.zoom * self.buffer.dim().width as f32)
                            < self.dim.width as f32
                        {
                            (self.dim.width as f32 - self.zoom * self.buffer.dim().width as f32)
                                / 2.0
                        } else {
                            0.0
                        };
                        let centered_offset_y = if (self.zoom * self.buffer.dim().height as f32)
                            < self.dim.height as f32
                        {
                            (self.dim.height as f32 - self.zoom * self.buffer.dim().height as f32)
                                / 2.0
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
                            let grid_x = source_x / grid;
                            let grid_y = source_y / grid;

                            if grid_x * grid < self.buffer.dim().width
                                && grid_y * grid < self.buffer.dim().height
                            {
                                //println!("{} {}", grid_x, grid_y);
                                if self.mode == TheRGBAViewMode::TileSelection {
                                    if self.selected.contains(&(grid_x, grid_y)) {
                                        self.selected.remove(&(grid_x, grid_y));
                                    } else {
                                        self.selected.insert((grid_x, grid_y));
                                    }
                                    ctx.ui.send(TheEvent::TileSelectionChanged(self.id.clone()));
                                } else if self.mode == TheRGBAViewMode::TileEditor {
                                    ctx.ui.send(TheEvent::TileEditorClicked(
                                        self.id.clone(),
                                        vec2i(grid_x, grid_y),
                                    ));
                                }
                            }
                        }
                    }
                    redraw = true;
                }
            }
            TheEvent::LostHover(_id) => {
                if self.hover.is_some() {
                    self.hover = None;
                    redraw = true;
                }
            }
            TheEvent::Hover(coord) => {
                if !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }

                if self.mode == TheRGBAViewMode::TileEditor && self.hover_color.is_some() {
                    if let Some(grid) = self.grid {
                        let centered_offset_x = if (self.zoom * self.buffer.dim().width as f32)
                            < self.dim.width as f32
                        {
                            (self.dim.width as f32 - self.zoom * self.buffer.dim().width as f32)
                                / 2.0
                        } else {
                            0.0
                        };
                        let centered_offset_y = if (self.zoom * self.buffer.dim().height as f32)
                            < self.dim.height as f32
                        {
                            (self.dim.height as f32 - self.zoom * self.buffer.dim().height as f32)
                                / 2.0
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
                            let grid_x = source_x / grid;
                            let grid_y = source_y / grid;

                            if grid_x * grid < self.buffer.dim().width
                                && grid_y * grid < self.buffer.dim().height
                                && Some((grid_x, grid_y)) != self.hover
                            {
                                self.hover = Some((grid_x, grid_y));
                                ctx.ui.send(TheEvent::TileEditorHoverChanged(
                                    self.id.clone(),
                                    vec2i(grid_x, grid_y),
                                ));
                            }
                        }
                    }
                    redraw = true;
                }
            }
            TheEvent::MouseWheel(delta) => {
                let d = vec2i(
                    (delta.x as f32 * self.wheel_scale) as i32,
                    (delta.y as f32 * self.wheel_scale) as i32,
                );
                ctx.ui.send(TheEvent::ScrollBy(self.hscrollbar.clone(), d));
                ctx.ui.send(TheEvent::ScrollBy(self.vscrollbar.clone(), d));
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
        _style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        pub fn mix_color(a: &[u8; 4], b: &[u8; 4], v: f32) -> [u8; 4] {
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
            (target_width - scaled_width) / 2.0
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

                if self.mode == TheRGBAViewMode::TileSelection {
                    if let Some(grid) = self.grid {
                        if src_x as i32 % grid == 0 || src_y as i32 % grid == 0 {
                            target.pixels_mut()[target_index..target_index + 4]
                                .copy_from_slice(&self.grid_color);
                            continue;
                        }
                    }
                }

                if src_x >= 0.0 && src_x < src_width && src_y >= 0.0 && src_y < src_height {
                    // Perform nearest neighbor interpolation
                    let src_x = src_x as i32;
                    let src_y = src_y as i32;
                    let src_index = (src_y * self.buffer.stride() as i32 + src_x) as usize * 4;

                    let mut copy = true;
                    if let Some(grid) = self.grid {
                        if self.selected.contains(&(src_x / grid, src_y / grid)) {
                            let s = self.buffer.pixels();
                            let c = &[
                                s[src_index],
                                s[src_index + 1],
                                s[src_index + 2],
                                s[src_index + 3],
                            ];
                            let m = mix_color(
                                c,
                                &self.selection_color,
                                self.selection_color[3] as f32 / 255.0,
                            );
                            target.pixels_mut()[target_index..target_index + 4].copy_from_slice(&m);
                            copy = false;
                        } else if let Some(hover_color) = self.hover_color {
                            if self.hover == Some((src_x / grid, src_y / grid)) {
                                let s = self.buffer.pixels();
                                let c = &[
                                    s[src_index],
                                    s[src_index + 1],
                                    s[src_index + 2],
                                    s[src_index + 3],
                                ];
                                let m = mix_color(c, &hover_color, hover_color[3] as f32 / 255.0);
                                target.pixels_mut()[target_index..target_index + 4]
                                    .copy_from_slice(&m);
                                copy = false;
                            }
                        }
                    }

                    // Copy the pixel from the source buffer to the target buffer
                    if copy {
                        target.pixels_mut()[target_index..target_index + 4]
                            .copy_from_slice(&self.buffer.pixels()[src_index..src_index + 4]);
                    }
                } else {
                    // Set the pixel to black if it's out of the source bounds
                    target.pixels_mut()[target_index..target_index + 4].fill(0);
                }
            }
        }

        /*
        // Loop over every pixel in the target buffer
        for target_y in 0..self.dim.height {
            for target_x in 0..self.dim.width {
                // Calculate the corresponding source coordinates
                let src_x = (target_x as f32 / self.zoom) - self.scroll_offset.x as f32;
                let src_y = (target_y as f32 / self.zoom) + self.scroll_offset.y as f32;

                // Calculate the index for the destination pixel
                let target_index = (target_y * target.dim().width + target_x) as usize * 4;

                if src_x >= 0.0 && src_x < src_width && src_y >= 0.0 && src_y < src_height {
                    // Perform nearest neighbor interpolation for simplicity here
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
        }*/

        self.is_dirty = false;
    }

    fn as_rgba_view(&mut self) -> Option<&mut dyn TheRGBAViewTrait> {
        Some(self)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait TheRGBAViewTrait: TheWidget {
    fn buffer(&self) -> &TheRGBABuffer;
    fn buffer_mut(&mut self) -> &mut TheRGBABuffer;
    fn set_buffer(&mut self, buffer: TheRGBABuffer);
    fn set_background(&mut self, color: RGBA);
    fn zoom(&self) -> f32;
    fn set_zoom(&mut self, zoom: f32);
    fn set_scroll_offset(&mut self, offset: Vec2i);
    fn set_grid(&mut self, grid: Option<i32>);
    fn set_grid_color(&mut self, color: RGBA);
    fn set_selection_color(&mut self, color: RGBA);
    fn set_wheel_scale(&mut self, wheel_scale: f32);
    fn set_hover_color(&mut self, color: Option<RGBA>);
    fn set_scrollbar_ids(&mut self, hscrollbar: TheId, vscrollbar: TheId);

    fn set_associated_layout(&mut self, id: TheId);

    fn selection(&self) -> FxHashSet<(i32, i32)>;
    fn selection_as_dim(&self) -> TheDim;
    fn selection_sorted(&self) -> Vec<(i32, i32)>;
    fn selection_as_sequence(&self) -> TheRGBARegionSequence;
    fn selection_as_tile(&self) -> TheRGBATile;
    fn set_selection(&mut self, selection: FxHashSet<(i32, i32)>);
    fn set_mode(&mut self, mode: TheRGBAViewMode);
}

impl TheRGBAViewTrait for TheRGBAView {
    fn buffer(&self) -> &TheRGBABuffer {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut TheRGBABuffer {
        &mut self.buffer
    }
    fn set_buffer(&mut self, buffer: TheRGBABuffer) {
        self.buffer = buffer;
        self.is_dirty = true;
    }
    fn set_background(&mut self, color: RGBA) {
        self.background = color;
    }
    fn set_scrollbar_ids(&mut self, hscrollbar: TheId, vscrollbar: TheId) {
        self.hscrollbar = hscrollbar;
        self.vscrollbar = vscrollbar;
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
    fn set_wheel_scale(&mut self, wheel_scale: f32) {
        self.wheel_scale = wheel_scale;
    }
    fn set_hover_color(&mut self, color: Option<RGBA>) {
        self.hover_color = color;
        self.is_dirty = true;
    }
    fn selection(&self) -> FxHashSet<(i32, i32)> {
        self.selected.clone()
    }
    fn selection_as_dim(&self) -> TheDim {
        let (mut min_x, mut min_y) = (i32::MAX, i32::MAX);
        let (mut max_x, mut max_y) = (i32::MIN, i32::MIN);

        for &(x, y) in &self.selected {
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        }

        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;

        TheDim::new(min_x, min_y, width, height)
    }
    fn selection_sorted(&self) -> Vec<(i32, i32)> {
        let mut vec: Vec<(i32, i32)> = self.selected.clone().into_iter().collect();
        vec.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        vec
    }
    fn selection_as_sequence(&self) -> TheRGBARegionSequence {
        let mut sequence = TheRGBARegionSequence::new();
        let sorted = self.selection_sorted();
        if let Some(grid) = self.grid {
            for s in sorted {
                sequence.regions.push(TheRGBARegion::new(
                    (s.0 * grid) as usize,
                    (s.1 * grid) as usize,
                    grid as usize,
                    grid as usize,
                ))
            }
        }
        sequence
    }
    fn selection_as_tile(&self) -> TheRGBATile {
        let sequence = self.selection_as_sequence();
        let buffer = self.buffer.extract_sequence(&sequence);
        let mut tile = TheRGBATile::new();
        tile.buffer = buffer;
        tile
    }
    fn set_selection(&mut self, selection: FxHashSet<(i32, i32)>) {
        self.selected = selection;
        self.is_dirty = true;
    }
    fn set_mode(&mut self, mode: TheRGBAViewMode) {
        self.mode = mode;
    }
}
