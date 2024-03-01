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

    codegrid: TheCodeGrid,
    grid_size: i32,

    debug_module: TheDebugModule,

    buffer: TheRGBABuffer,

    scroll_offset: Vec2i,
    zoom: f32,

    selected: Option<(u16, u16)>,
    hover: Option<(u16, u16)>,
    drop: Option<(u16, u16)>,
    drop_atom: Option<TheCodeAtom>,

    mouse_down_pos: Vec2i,

    hscrollbar: TheId,
    vscrollbar: TheId,

    dim: TheDim,
    code_is_dirty: bool,
    is_dirty: bool,

    layout_id: TheId,

    drag_copy: bool,
    shift_is_down: bool,

    command: String,
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

            codegrid: TheCodeGrid::new(),
            grid_size: 70,

            debug_module: TheDebugModule::default(),

            scroll_offset: vec2i(0, 0),
            zoom: 1.0,

            selected: None,
            hover: None,
            drop: None,
            drop_atom: None,

            mouse_down_pos: Vec2i::zero(),

            hscrollbar: TheId::empty(),
            vscrollbar: TheId::empty(),

            dim: TheDim::zero(),
            code_is_dirty: true,
            is_dirty: true,

            layout_id: TheId::empty(),

            drag_copy: false,
            shift_is_down: false,

            command: "".to_string(),
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

                self.mouse_down_pos = *coord;
                redraw = true;
            }
            TheEvent::MouseDragged(coord) => {
                if let Some(selected) = &self.selected {
                    if ctx.ui.drop.is_none()
                        && distance(Vec2f::from(self.mouse_down_pos), Vec2f::from(*coord)) >= 5.0
                    {
                        if let Some(atom) = self.codegrid.code.get(selected) {
                            let mut drop = TheDrop::new(TheId::named("Code Editor Atom"));
                            drop.title = atom.describe();
                            drop.set_data(atom.to_json());
                            drop.start_position = Some(vec2i(selected.0 as i32, selected.1 as i32));
                            drop.operation = if self.drag_copy {
                                TheDropOperation::Copy
                            } else {
                                TheDropOperation::Move
                            };
                            ctx.ui.send(TheEvent::DragStartedWithNoImage(drop));
                        }
                    }
                }
            }
            TheEvent::DropPreview(coord, drop) => {
                if drop.id.name == "Code Editor Atom" {
                    let c = self.get_code_grid_offset(*coord);
                    self.selected = self.get_code_grid_offset(*coord);
                    if c != self.drop {
                        self.drop = c;
                        redraw = true;
                        self.drop_atom = Some(TheCodeAtom::from_json(&drop.data));
                        self.code_is_dirty = true;
                    }
                }
            }
            TheEvent::Drop(coord, drop) => {
                if drop.id.name == "Code Editor Atom" {
                    self.selected = self.get_code_grid_offset(*coord);
                    if let Some(c) = self.get_code_grid_offset(*coord) {
                        let mut atom = TheCodeAtom::from_json(&drop.data);

                        if let Some((x, y)) = &self.selected {
                            if (x % 2 == 1 || y % 2 == 1) && atom.uneven_slot()
                                || x % 2 == 0 && y % 2 == 0 && !atom.uneven_slot()
                            {
                                if atom.can_assign()
                                    && !self.codegrid.code.contains_key(&(x + 1, *y))
                                {
                                    self.codegrid.code.insert(
                                        (x + 1, *y),
                                        TheCodeAtom::Assignment(TheValueAssignment::Assign),
                                    );
                                }

                                if let TheCodeAtom::ExternalCall(_, _, _, arg_values, _) = &atom {
                                    for (index, value) in arg_values.iter().enumerate() {
                                        let off = x + (index + 1) as u16 * 2;

                                        self.codegrid
                                            .code
                                            .entry((off, *y))
                                            .or_insert_with(|| TheCodeAtom::Value(value.clone()));
                                    }
                                }

                                if self.shift_is_down {
                                    if let TheCodeAtom::LocalSet(name, _) = atom {
                                        atom = TheCodeAtom::LocalGet(name);
                                    } else if let TheCodeAtom::ObjectSet(object, name, _) = atom {
                                        atom = TheCodeAtom::ObjectGet(object, name);
                                    }
                                } else if drop.operation == TheDropOperation::Move {
                                    if let Some(sp) = drop.start_position {
                                        self.codegrid.code.remove(&(sp.x as u16, sp.y as u16));
                                    }
                                }

                                self.codegrid.code.insert(c, atom);
                                redraw = true;
                                self.code_is_dirty = true;
                                ctx.ui.send(TheEvent::CodeEditorChanged(
                                    self.id.clone(),
                                    self.codegrid.clone(),
                                ));
                                ctx.ui.send(TheEvent::CodeEditorSelectionChanged(
                                    self.id().clone(),
                                    self.selected,
                                ));
                            }
                        }
                        self.drop = None;
                        self.drop_atom = None;
                    }
                }
            }
            TheEvent::MouseUp(_coord) => {
                if self.drop.is_some() {
                    self.drop = None;
                    self.drop_atom = None;
                    redraw = true;
                    self.code_is_dirty = true;
                }
            }
            TheEvent::Hover(coord) => {
                if !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }

                let hover = self.get_code_grid_offset(*coord);
                if hover != self.hover {
                    self.code_is_dirty = true;
                    self.hover = hover;
                    redraw = true;
                    if let Some(hover) = hover {
                        if let Some(message) = self.codegrid.messages.get(&hover) {
                            if message.message_type == TheCodeGridMessageType::Error {
                                ctx.ui.send(TheEvent::SetStatusText(
                                    self.id().clone(),
                                    format!("Error: {}", message.message),
                                ));
                            }
                        } else if let Some(atom) = self.codegrid.code.get(&hover) {
                            let text = atom.help(); //format!("({}, {}) {}", hover.0, hover.1, atom.help());
                            ctx.ui
                                .send(TheEvent::SetStatusText(self.id().clone(), text));
                        } else {
                            ctx.ui
                                .send(TheEvent::SetStatusText(self.id().clone(), "".to_string()));
                        }
                    }
                }
            }
            TheEvent::ModifierChanged(shift, ctrl, alt, _logo) => {
                self.drag_copy = *ctrl || *alt;
                self.shift_is_down = *shift;
            }
            TheEvent::KeyCodeDown(code) => {
                if let Some(code) = code.to_key_code() {
                    if code == TheKeyCode::Return {
                        if let Some(selected) = &mut self.selected {
                            if !self.command.is_empty() {
                                self.process_command(self.command.clone(), ctx);
                                self.command.clear();
                            } else {
                                self.codegrid.move_one_line_down(*selected);
                                selected.1 += 2;
                                self.code_is_dirty = true;
                                redraw = true;
                                self.hover = None;
                                ctx.ui.send(TheEvent::CodeEditorChanged(
                                    self.id.clone(),
                                    self.codegrid.clone(),
                                ));
                            }
                        }
                    } else if code == TheKeyCode::Escape {
                        self.command.clear();
                    } else if code == TheKeyCode::Delete {
                        if let Some(selected) = self.selected {
                            self.selected = Some(self.codegrid.delete(selected));
                            // if self.codegrid.code.contains_key(&pos) {
                            //     self.selected = Some(pos);
                            // } else {
                            //     self.selected = None;
                            // }
                            self.code_is_dirty = true;
                            redraw = true;
                            self.hover = None;
                            ctx.ui.send(TheEvent::CodeEditorChanged(
                                self.id.clone(),
                                self.codegrid.clone(),
                            ));
                        }
                    } else if code == TheKeyCode::Space {
                        if let Some(selected) = &mut self.selected {
                            self.codegrid.insert_space(*selected);
                            selected.0 += 2;
                            self.code_is_dirty = true;
                            redraw = true;
                            ctx.ui.send(TheEvent::CodeEditorChanged(
                                self.id.clone(),
                                self.codegrid.clone(),
                            ));
                        }
                    } else if code == TheKeyCode::Left {
                        if self.selected.is_none() {
                            self.selected = Some((0, 0));
                        }

                        if let Some(selected) = &mut self.selected {
                            if selected.0 > 0 {
                                selected.0 -= 1;
                                self.code_is_dirty = true;
                                redraw = true;
                            }
                        }
                    } else if code == TheKeyCode::Up {
                        if self.selected.is_none() {
                            self.selected = Some((0, 0));
                        }

                        if let Some(selected) = &mut self.selected {
                            if selected.1 > 1 {
                                selected.1 -= 2;
                                self.code_is_dirty = true;
                                redraw = true;
                            }
                        }
                    } else if code == TheKeyCode::Right {
                        if self.selected.is_none() {
                            self.selected = Some((0, 0));
                        }
                        if let Some(selected) = &mut self.selected {
                            if let Some(minmax) = self.codegrid.max_xy() {
                                if selected.0 < minmax.0 + 2 {
                                    selected.0 += 1;
                                    self.code_is_dirty = true;
                                    redraw = true;
                                }
                            }
                        }
                    } else if code == TheKeyCode::Down {
                        if self.selected.is_none() {
                            self.selected = Some((0, 0));
                        }
                        if let Some(selected) = &mut self.selected {
                            if let Some(minmax) = self.codegrid.max_xy() {
                                if selected.1 < minmax.1 + 2 {
                                    selected.1 += 2;
                                    self.code_is_dirty = true;
                                    redraw = true;
                                }
                            }
                        }
                    }
                }
            }
            TheEvent::MouseWheel(delta) => {
                let d = vec2i(
                    (delta.x as f32 * -0.4) as i32,
                    (delta.y as f32 * -0.4) as i32,
                );
                ctx.ui.send(TheEvent::ScrollBy(self.hscrollbar.clone(), d));
                ctx.ui.send(TheEvent::ScrollBy(self.vscrollbar.clone(), d));
            }
            TheEvent::LostHover(_id) => {
                if self.hover.is_some() {
                    self.hover = None;
                    self.drop = None;
                    redraw = true;
                    self.is_dirty = true;
                    self.code_is_dirty = true;
                }
            }
            TheEvent::KeyDown(key) => {
                if let Some(key) = key.to_char() {
                    self.command.push(key);
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
            self.code_is_dirty = true;
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

    fn status_text(&self) -> Option<String> {
        Some("".to_string())
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

        /*
        let stride: usize = self.buffer.stride();
        let background: [u8; 4] = *style.theme().color(CodeGridBackground);
        let line: [u8; 4] = *style.theme().color(CodeGridDark);

        let offset_x = 0 as f32;//-self.scroll_offset.x as f32;
        let offset_y = -self.scroll_offset.y as f32;

        //let target = buffer;
        let grid = self.grid_size;

        for target_y in 0..self.dim.height {
            for target_x in 0..self.dim.width {
                // Calculate the corresponding source coordinates with the offset
                let src_x = (target_x as f32 - offset_x) / self.zoom;
                let src_y = (target_y as f32 - offset_y) / self.zoom;

                // Calculate the index for the destination pixel
                let target_index = ((self.dim.buffer_y + target_y) * buffer.dim().width
                    + target_x
                    + self.dim.buffer_x) as usize
                    * 4;

                let mut color = background;

                if src_x as i32 % grid == 0 || src_y as i32 % grid == 0 {
                    color = line;
                }

                buffer.pixels_mut()[target_index..target_index + 4]
                    .copy_from_slice(&color);
            }
        }*/

        // --- Draw the code grid into the buffer

        let background = *style.theme().color(CodeGridBackground);

        if self.code_is_dirty {
            let (grid_x, grid_y) = self.adjust_buffer_to_grid();
            let stride: usize = self.buffer.stride();

            let normal: [u8; 4] = *style.theme().color(CodeGridNormal);
            let dark: [u8; 4] = *style.theme().color(CodeGridDark);
            let selected = *style.theme().color(CodeGridSelected);
            let text_color = *style.theme().color(CodeGridText); //[242, 242, 242, 255];
            let ok = *style.theme().color(Green);
            let error = *style.theme().color(Red);
            let mut hover = *style.theme().color(CodeGridHover);
            hover[3] = 200;

            let utuple = self.buffer.dim().to_buffer_utuple();
            ctx.draw
                .rect(self.buffer.pixels_mut(), &utuple, stride, &background);

            // let border_color = dark;
            let font_size = 12.5_f32 * self.zoom;

            let grid_size = ceil(self.grid_size as f32 * self.zoom) as i32;
            //let rounding = 10.0 * self.zoom;
            //let border_size = 1.5 * self.zoom;

            let zoom = self.zoom;
            fn zoom_const(v: usize, zoom: f32) -> usize {
                (v as f32 * zoom) as usize
            }

            let mut canvas = TheSDFCanvas::new();
            canvas.background = crate::thecolor::TheColor::from_u8_array(background);
            canvas.highlight = crate::thecolor::TheColor::from_u8_array(selected);
            canvas.hover_highlight =
                crate::thecolor::TheColor::from_u8_array(*style.theme().color(CodeGridHover));

            let pattern_normal = ThePattern::SolidWithBorder(
                crate::thecolor::TheColor::from_u8_array(normal),
                crate::thecolor::TheColor::from_u8_array(dark),
                1.5 * zoom,
            );

            // let pattern_selected = ThePattern::SolidWithBorder(
            //     crate::thecolor::TheColor::from_u8_array(selected),
            //     crate::thecolor::TheColor::from_u8_array(dark),
            //     1.5 * zoom,
            // );

            // let pattern_hover = ThePattern::SolidWithBorder(
            //     crate::thecolor::TheColor::from_u8_array(hover),
            //     crate::thecolor::TheColor::from_u8_array(dark),
            //     1.5 * zoom,
            // );

            // fn check_selection(x: u16, y: u16, canvas: &mut TheSDFCanvas) {
            //     // if Some((x, y)) == self.selected {
            //     //     pattern_selected.clone()
            //     // } else if Some((x, y)) == self.hover {
            //     //     pattern_hover.clone()
            //     // } else {
            //     //     pattern_normal.clone()
            //     // }

            //     if Some((x, y)) == self.selected {
            //         canvas.selected = Some(canvas.sdfs.len());
            //     } else if Some((x, y)) == self.hover {
            //         canvas.hover = Some(canvas.sdfs.len());
            //     }
            // };

            let mut func_args_hash: FxHashMap<(u16, u16), (String, bool)> = FxHashMap::default();

            for y in 0..grid_y {
                for x in 0..grid_x {
                    if x % 2 == 1 || y % 2 == 1 {
                        continue;
                    }

                    let rect = (
                        (x / 2 * grid_size as u16) as usize,
                        (y / 2 * grid_size as u16) as usize,
                        grid_size as usize,
                        grid_size as usize,
                    );

                    canvas.clear();
                    canvas.selected = None;

                    let dim = TheDim::sized(grid_size, grid_size);

                    // Main atom
                    if let Some(atom) = self.codegrid.code.get(&(x, y)) {
                        let mut sdf = atom.to_sdf(dim, zoom);

                        // let pattern = ThePattern::SolidWithBorder(
                        //     crate::thecolor::TheColor::from_u8_array(atom.to_color()),
                        //     crate::thecolor::TheColor::from_u8_array(dark),
                        //     1.5 * zoom,
                        // );

                        // Insert the functions arguments into the hash map.
                        if let TheCodeAtom::ExternalCall(_, _, arg_names, _, _) = &atom {
                            if !arg_names.is_empty() {
                                // Avoid the border
                                let mut d = dim;
                                d.width += 2;
                                sdf = TheSDF::RoundedRect(d, (0.0, 0.0, 10.0 * zoom, 10.0 * zoom));
                            }
                            for (index, name) in arg_names.iter().enumerate() {
                                let off = x + (index + 1) as u16 * 2;

                                func_args_hash
                                    .insert((off, y), (name.clone(), index == arg_names.len() - 1));
                            }
                        } else if let Some((_, at_end)) = func_args_hash.get(&(x, y)) {
                            // Avoid the border
                            let mut d = dim;
                            d.x -= 2;
                            d.width += 2;
                            if *at_end {
                                sdf = TheSDF::RoundedRect(d, (10.0 * zoom, 10.0 * zoom, 0.0, 0.0));
                            } else {
                                sdf = TheSDF::RoundedRect(d, (0.0, 0.0, 0.0, 0.0));
                            }
                        }

                        canvas.add(sdf, pattern_normal.clone());

                        if Some((x, y)) == self.selected {
                            canvas.selected = Some(0);
                        } else if Some((x, y)) == self.hover {
                            canvas.hover = Some(0);
                        }
                        if let Some(message) = self.codegrid.messages.get(&(x, y)) {
                            if message.message_type == TheCodeGridMessageType::Error {
                                canvas.error = Some(0);
                            }
                        }
                    }

                    if x > 0 {
                        // Minor to the left
                        if let Some(atom) = self.codegrid.code.get(&(x - 1, y)) {
                            let dim = TheDim::new(
                                -grid_size / 4,
                                grid_size / 4,
                                grid_size / 2,
                                grid_size / 2,
                            );

                            let sdf = atom.to_sdf(dim, zoom);
                            canvas.add(sdf, pattern_normal.clone());

                            if Some((x - 1, y)) == self.selected {
                                canvas.selected = Some(canvas.sdfs.len() - 1);
                            } else if Some((x - 1, y)) == self.hover {
                                canvas.hover = Some(canvas.sdfs.len() - 1);
                            }
                        }
                    }

                    // Minor to the right
                    if let Some(atom) = self.codegrid.code.get(&(x + 1, y)) {
                        let dim = TheDim::new(
                            grid_size - grid_size / 4,
                            grid_size / 4,
                            grid_size / 2,
                            grid_size / 2,
                        );

                        let sdf = atom.to_sdf(dim, zoom);
                        canvas.add(sdf, pattern_normal.clone());

                        if Some((x + 1, y)) == self.selected {
                            canvas.selected = Some(canvas.sdfs.len() - 1);
                        } else if Some((x + 1, y)) == self.hover {
                            canvas.hover = Some(canvas.sdfs.len() - 1);
                        }
                    }

                    if !canvas.is_empty() {
                        let mut b = TheRGBABuffer::new(dim);
                        canvas.render(&mut b);

                        self.buffer.copy_into(rect.0 as i32, rect.1 as i32, &b);
                    }
                }
            }

            for y in 0..grid_y {
                for x in 0..grid_x {
                    let mut rect = (
                        (x / 2 * grid_size as u16) as usize,
                        (y / 2 * grid_size as u16) as usize,
                        grid_size as usize,
                        grid_size as usize,
                    );

                    if x % 2 == 1 {
                        rect.0 += rect.2 - grid_size as usize / 4;
                        rect.1 += grid_size as usize / 4;
                        rect.2 /= 2;
                        rect.3 /= 2;
                    }

                    if y % 2 == 1 {
                        rect.0 += grid_size as usize / 4;
                        rect.1 += rect.3 - grid_size as usize / 4;
                        rect.2 /= 2;
                        rect.3 /= 2;
                    }

                    if Some((x, y)) == self.drop {
                        if let Some(drop_atom) = &self.drop_atom {
                            if (x % 2 == 1 || y % 2 == 1) && drop_atom.uneven_slot()
                                || x % 2 == 0 && y % 2 == 0 && !drop_atom.uneven_slot()
                            {
                                ctx.draw
                                    .rect_outline(self.buffer.pixels_mut(), &rect, stride, &ok);
                            } else {
                                ctx.draw.rect_outline(
                                    self.buffer.pixels_mut(),
                                    &rect,
                                    stride,
                                    &error,
                                );
                            }
                        }
                    } /*else if Some((x, y)) == self.hover {
                          ctx.draw
                              .blend_rect(self.buffer.pixels_mut(), &rect, stride, &hover);
                      }*/

                    // let mut color = if Some((x, y)) == self.selected {
                    //     selected
                    // } else {
                    //     normal
                    // };

                    if let Some(message) = self.codegrid.messages.get(&(x, y)) {
                        if message.message_type == TheCodeGridMessageType::Error {
                            //color = error;
                            if !self.codegrid.code.contains_key(&(x, y)) {
                                ctx.draw
                                    .rect(self.buffer.pixels_mut(), &rect, stride, &error);
                            }
                        }
                    }

                    if let Some(atom) = self.codegrid.code.get(&(x, y)) {
                        if let Some(font) = &ctx.ui.font {
                            match atom {
                                TheCodeAtom::ObjectGet(object, var)
                                | TheCodeAtom::ObjectSet(object, var, _) => {
                                    ctx.draw.text_rect_blend(
                                        self.buffer.pixels_mut(),
                                        &(rect.0, rect.1 + 8, rect.2, rect.3 - 16),
                                        stride,
                                        font,
                                        font_size,
                                        object,
                                        &text_color,
                                        TheHorizontalAlign::Center,
                                        TheVerticalAlign::Top,
                                    );
                                    ctx.draw.text_rect_blend(
                                        self.buffer.pixels_mut(),
                                        &(rect.0 + 4, rect.1, rect.2 - 8, rect.3),
                                        stride,
                                        font,
                                        font_size,
                                        var,
                                        &text_color,
                                        TheHorizontalAlign::Center,
                                        TheVerticalAlign::Center,
                                    );
                                }
                                TheCodeAtom::Get(path) | TheCodeAtom::Set(path, _) => {
                                    let parts: Vec<String> =
                                        path.split('.').map(|s| s.to_string()).collect();

                                    if parts.len() == 1 {
                                        ctx.draw.text_rect_blend(
                                            self.buffer.pixels_mut(),
                                            &(rect.0 + 4, rect.1, rect.2 - 8, rect.3),
                                            stride,
                                            font,
                                            font_size,
                                            &parts[0],
                                            &text_color,
                                            TheHorizontalAlign::Center,
                                            TheVerticalAlign::Center,
                                        );
                                    } else if parts.len() == 2 {
                                        ctx.draw.text_rect_blend(
                                            self.buffer.pixels_mut(),
                                            &(rect.0, rect.1 + 8, rect.2, rect.3 - 16),
                                            stride,
                                            font,
                                            font_size,
                                            &parts[0],
                                            &WHITE,
                                            TheHorizontalAlign::Center,
                                            TheVerticalAlign::Top,
                                        );
                                        ctx.draw.text_rect_blend(
                                            self.buffer.pixels_mut(),
                                            &(rect.0 + 4, rect.1, rect.2 - 8, rect.3),
                                            stride,
                                            font,
                                            font_size,
                                            &parts[1],
                                            &text_color,
                                            TheHorizontalAlign::Center,
                                            TheVerticalAlign::Center,
                                        );
                                    } else {
                                        for (index, part) in parts.iter().enumerate() {
                                            ctx.draw.text_rect_blend(
                                                self.buffer.pixels_mut(),
                                                &(
                                                    rect.0 + 2,
                                                    rect.1 + zoom_const(4 + index * 16, zoom),
                                                    rect.2 - 4,
                                                    zoom_const(16, zoom),
                                                ),
                                                stride,
                                                font,
                                                font_size,
                                                part,
                                                if index != 0 { &text_color } else { &WHITE },
                                                TheHorizontalAlign::Center,
                                                TheVerticalAlign::Center,
                                            );
                                        }
                                    }
                                }
                                TheCodeAtom::ExternalCall(_, _, _, _, _)
                                | TheCodeAtom::ModuleCall(_, _, _, _) => {
                                    let executed = self.debug_module.executed.contains(&(x, y));
                                    let c = if executed { &WHITE } else { &text_color };
                                    ctx.draw.text_rect_blend(
                                        self.buffer.pixels_mut(),
                                        &(rect.0 + 2, rect.1, rect.2 - 4, rect.3),
                                        stride,
                                        font,
                                        font_size,
                                        &atom.describe(),
                                        c,
                                        TheHorizontalAlign::Center,
                                        TheVerticalAlign::Center,
                                    );
                                }
                                _ => {
                                    let executed = self.debug_module.executed.contains(&(x, y));
                                    let c = if executed { &WHITE } else { &text_color };
                                    ctx.draw.text_rect_blend(
                                        self.buffer.pixels_mut(),
                                        &(rect.0 + 2, rect.1, rect.2 - 4, rect.3),
                                        stride,
                                        font,
                                        font_size,
                                        &atom.describe(),
                                        c,
                                        TheHorizontalAlign::Center,
                                        TheVerticalAlign::Center,
                                    );
                                }
                            }

                            if let TheCodeAtom::Value(TheValue::ColorObject(color, _)) = atom {
                                let off = zoom_const(4, zoom);
                                ctx.draw.rounded_rect(
                                    self.buffer.pixels_mut(),
                                    &(
                                        rect.0 + 4 * off,
                                        rect.1 + rect.3 - 7 * off,
                                        rect.2 - 8 * off,
                                        3 * off,
                                    ),
                                    stride,
                                    &color.to_u8_array(),
                                    &(2.0 * zoom, 2.0 * zoom, 2.0 * zoom, 2.0 * zoom),
                                );
                            }

                            if let Some((name, _)) = func_args_hash.get(&(x, y)) {
                                ctx.draw.text_rect_blend(
                                    self.buffer.pixels_mut(),
                                    &(rect.0, rect.1, rect.2, rect.3 - zoom_const(5, zoom)),
                                    stride,
                                    font,
                                    font_size,
                                    name,
                                    &text_color,
                                    TheHorizontalAlign::Center,
                                    TheVerticalAlign::Bottom,
                                );
                            }

                            if let Some(v) = self.debug_module.values.get(&(x, y)) {
                                if let Some(top) = &v.0 {
                                    ctx.draw.text_rect_blend(
                                        self.buffer.pixels_mut(),
                                        &(
                                            rect.0,
                                            rect.1 + zoom_const(5, zoom),
                                            rect.2,
                                            rect.3 - zoom_const(10, zoom),
                                        ),
                                        stride,
                                        font,
                                        font_size,
                                        &top.describe(),
                                        &WHITE,
                                        TheHorizontalAlign::Center,
                                        TheVerticalAlign::Top,
                                    );
                                }
                                ctx.draw.text_rect_blend(
                                    self.buffer.pixels_mut(),
                                    &(rect.0, rect.1, rect.2, rect.3 - zoom_const(5, zoom)),
                                    stride,
                                    font,
                                    font_size,
                                    &v.1.describe(),
                                    &WHITE,
                                    TheHorizontalAlign::Center,
                                    TheVerticalAlign::Bottom,
                                );
                            }
                        }
                    } else if Some((x, y)) == self.selected && self.drop_atom.is_none() {
                        ctx.draw
                            .rect_outline(self.buffer.pixels_mut(), &rect, stride, &selected);
                    } else if Some((x, y)) == self.hover {
                        ctx.draw
                            .blend_rect(self.buffer.pixels_mut(), &rect, stride, &hover);
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

        let zoom = 1.0;

        // Calculate the scaled dimensions of the source image
        let scaled_width = src_width * zoom;
        let scaled_height = src_height * zoom;

        // Calculate the offset to center the image
        let offset_x = if scaled_width < target_width {
            0.0 //(target_width - scaled_width) / 2.0
        } else {
            -self.scroll_offset.x as f32
        };

        let offset_y = if scaled_height < target_height {
            0.0 //(target_height - scaled_height) / 2.0
        } else {
            -self.scroll_offset.y as f32
        };

        // Loop over every pixel in the target buffer
        for target_y in 0..self.dim.height {
            for target_x in 0..self.dim.width {
                // Calculate the corresponding source coordinates with the offset
                let src_x = (target_x as f32 - offset_x) / zoom;
                let src_y = (target_y as f32 - offset_y) / zoom;

                // Calculate the index for the destination pixel
                let target_index = ((self.dim.buffer_y + target_y) * target.dim().width
                    + target_x
                    + self.dim.buffer_x) as usize
                    * 4;

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
                    target.pixels_mut()[target_index..target_index + 4]
                        .copy_from_slice(&background);
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

/// `TheCodeViewTrait` trait defines a set of functionalities specifically for `TheCodeView` widget.
pub trait TheCodeViewTrait: TheWidget {
    /// Processes a command string for the current location.
    fn process_command(&mut self, command: String, ctx: &mut TheContext);

    /// Adjusts the buffer size to match the size of the code grid. This ensures
    /// that the buffer is correctly sized to fit the grid layout.
    fn adjust_buffer_to_grid(&mut self) -> (u16, u16);

    /// Provides a reference to the RGBA buffer used by the code view.
    /// This buffer is where the code view's visual representation is drawn.
    fn buffer(&self) -> &TheRGBABuffer;

    /// Provides a mutable reference to the RGBA buffer.
    /// This allows for modifications to the buffer, such as drawing or clearing operations.
    fn buffer_mut(&mut self) -> &mut TheRGBABuffer;

    /// Returns a reference to the `TheCodeGrid`, which holds the logical structure
    /// of the code grid, such as the arrangement and types of code elements.
    fn codegrid(&self) -> &TheCodeGrid;

    /// Returns a mutable reference to the `TheCodeGrid`.
    /// This is used for updating or modifying the code grid's structure.
    fn codegrid_mut(&mut self) -> &mut TheCodeGrid;

    /// Sets the code grid to a new `TheCodeGrid`.
    /// This function is used when the entire grid needs to be replaced or reset.
    fn set_codegrid(&mut self, code_grid: TheCodeGrid);

    /// Sets the debug values environment for the code view.
    fn set_debug_module(&mut self, module: TheDebugModule);

    /// Sets the background color of the code view.
    /// This affects the visual appearance of the widget's background.
    fn set_background(&mut self, color: RGBA);

    /// Gets the current zoom level of the code view.
    /// This determines how much the content is scaled visually.
    fn zoom(&self) -> f32;

    /// Sets the zoom level for the code view.
    /// This affects the scale at which the content is displayed.
    fn set_zoom(&mut self, zoom: f32);

    /// Sets the scroll offset for the code view.
    /// This affects the portion of the content that is visible in the view.
    fn set_scroll_offset(&mut self, offset: Vec2i);

    /// Sets the IDs of the horizontal and vertical scrollbars.
    /// These IDs are used to interact with the scrollbars in the UI.
    fn set_scrollbar_ids(&mut self, hscrollbar: TheId, vscrollbar: TheId);

    /// Associates a layout ID with the code view.
    /// This can be used to link the code view with a specific layout in the UI.
    fn set_associated_layout(&mut self, id: TheId);

    /// Returns the currently selected coordinates in the grid, if any.
    /// This is useful for operations that depend on the user's selection.
    fn selected(&self) -> Option<(u16, u16)>;

    // / Sets the value of an atom in the code grid.
    // / This is used for updating the content or behavior of a specific grid element.
    //fn set_atom_value(&mut self, coord: (u16, u16), name: String, value: TheValue);

    /// Sets a specific `TheCodeAtom` at given grid coordinates.
    /// This allows for modifying the type or properties of a grid element.
    fn set_grid_atom(&mut self, coord: (u16, u16), atom: TheCodeAtom);

    /// Calculates and returns the grid coordinates corresponding to a given screen position.
    /// This is used for translating screen interactions into grid operations.
    fn get_code_grid_offset(&self, coord: Vec2i) -> Option<(u16, u16)>;
}

impl TheCodeViewTrait for TheCodeView {
    fn process_command(&mut self, command: String, ctx: &mut TheContext) {
        println!("Command: {}", command);

        let mut atom: Option<TheCodeAtom> = None;
        if let Some(selection) = self.selected {
            if command == "=" && selection.0 % 2 == 1 {
                atom = Some(TheCodeAtom::Assignment(TheValueAssignment::Assign));
            }
        }

        if let Some(atom) = atom {
            self.set_grid_atom(self.selected.unwrap(), atom);
            ctx.ui.send(TheEvent::CodeEditorChanged(
                self.id.clone(),
                self.codegrid.clone(),
            ));
        }
    }

    fn adjust_buffer_to_grid(&mut self) -> (u16, u16) {
        let size = self.codegrid.max_xy();

        let grid_x;
        let grid_y;

        if let Some(size) = size {
            grid_x = size.0 as i32 + 3;
            grid_y = size.1 as i32 + 3;
        } else {
            grid_x = 2;
            grid_y = 2;
        }

        let grid_size = ceil(self.grid_size as f32 * self.zoom) as i32;

        let d = self.buffer().dim();
        if d.width != grid_x * grid_size || d.height != grid_y * grid_size {
            let width = grid_x * grid_size / 2 + grid_size / 2;
            let height = grid_y * grid_size / 2 + grid_size / 2;
            let b = TheRGBABuffer::new(TheDim::new(0, 0, width, height));
            self.buffer = b;
        }

        (grid_x as u16, grid_y as u16)
    }
    fn buffer(&self) -> &TheRGBABuffer {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut TheRGBABuffer {
        &mut self.buffer
    }
    fn codegrid(&self) -> &TheCodeGrid {
        &self.codegrid
    }
    fn codegrid_mut(&mut self) -> &mut TheCodeGrid {
        self.is_dirty = true;
        self.code_is_dirty = true;
        &mut self.codegrid
    }
    fn set_codegrid(&mut self, code_grid: TheCodeGrid) {
        self.codegrid = code_grid;
        self.code_is_dirty = true;
        self.is_dirty = true;
    }
    fn set_debug_module(&mut self, module: TheDebugModule) {
        self.debug_module = module;
        self.code_is_dirty = true;
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
        self.is_dirty = true;
        self.code_is_dirty = true;
        self.hover = None;
    }
    fn set_scroll_offset(&mut self, offset: Vec2i) {
        self.scroll_offset = offset;
    }
    fn set_associated_layout(&mut self, layout_id: TheId) {
        self.layout_id = layout_id;
    }
    fn selected(&self) -> Option<(u16, u16)> {
        self.selected
    }
    // fn set_atom_value(&mut self, coord: (u16, u16), name: String, value: TheValue) {
    //     if let Some(atom) = self.codegrid.code.get_mut(&coord) {
    //         atom.process_value_change(name, value);
    //         self.code_is_dirty = true;
    //         self.is_dirty = true;
    //     }
    // }
    fn set_grid_atom(&mut self, coord: (u16, u16), atom: TheCodeAtom) {
        self.codegrid.code.insert(coord, atom);
        self.code_is_dirty = true;
        self.is_dirty = true;
    }
    fn get_code_grid_offset(&self, coord: Vec2i) -> Option<(u16, u16)> {
        let centered_offset_x = 0.0;
        // if (self.zoom * self.buffer.dim().width as f32) < self.dim.width as f32 {
        //     (self.dim.width as f32 - self.zoom * self.buffer.dim().width as f32) / 2.0
        // } else {
        //     0.0
        // };
        let centered_offset_y = 0.0;
        // if (self.zoom * self.buffer.dim().height as f32) < self.dim.height as f32 {
        //     (self.dim.height as f32 - self.zoom * self.buffer.dim().height as f32) / 2.0
        // } else {
        //     0.0
        // };

        let grid_size = ceil(self.grid_size as f32 * self.zoom) as i32;

        let source_x = ((coord.x as f32 - centered_offset_x) / 1.0//self.zoom
            + self.scroll_offset.x as f32)
            .round() as i32;
        let source_y = ((coord.y as f32 - centered_offset_y) / 1.0//self.zoom
            + self.scroll_offset.y as f32)
            .round() as i32;

        if source_x >= 0
            //&& source_x < self.buffer.dim().width
            && source_y >= 0
        //&& source_y < self.buffer.dim().height
        {
            let quarter_grid_size = grid_size / 4;

            let mut grid_x = source_x / grid_size;
            let mut grid_y = source_y / grid_size;

            let grid_off_x = source_x % grid_size;
            let grid_off_y = source_y % grid_size;

            // Check x coordinate
            if grid_off_x > (grid_size - quarter_grid_size)
                && grid_off_y > (grid_size / 2 - quarter_grid_size)
                && grid_off_y < (grid_size / 2 + quarter_grid_size)
            {
                grid_x = grid_x * 2 + 1;
            } else if (source_x % grid_size) < (quarter_grid_size)
                && grid_off_y > (grid_size / 2 - quarter_grid_size)
                && grid_off_y < (grid_size / 2 + quarter_grid_size)
            {
                grid_x = grid_x * 2 - 1;
            } else {
                grid_x *= 2;
            }

            // Check y coordinate
            if grid_off_y > (grid_size - quarter_grid_size)
                && grid_off_x > (grid_size / 2 - quarter_grid_size)
                && grid_off_x < (grid_size / 2 + quarter_grid_size)
                && grid_y % 2 == 1
            {
                grid_y = grid_y * 2 + 1;
            } else if (source_y % grid_size) < (quarter_grid_size)
                && grid_off_x > (grid_size / 2 - quarter_grid_size)
                && grid_off_x < (grid_size / 2 + quarter_grid_size)
                && grid_y % 2 == 0
            {
                grid_y = grid_y * 2 - 1;
            } else {
                grid_y *= 2;
            }

            //if grid_x * grid_size < self.buffer.dim().width
            //     && grid_y * grid_size < self.buffer.dim().height
            // {
            return Some((grid_x as u16, grid_y as u16));
            // }
        }
        None
    }
}
