use crate::prelude::*;

pub struct TheCodeEditor {
    code_list_selection: Option<TheId>,
    grid_selection: Option<(u16, u16)>,

    undo: Option<TheUndo>,
}

impl Default for TheCodeEditor {
    fn default() -> Self {
        TheCodeEditor::new()
    }
}

impl TheCodeEditor {
    pub fn new() -> Self {
        Self {
            code_list_selection: None,
            grid_selection: None,

            undo: None,
        }
    }

    pub fn handle_event(&mut self, event: &TheEvent, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        match event {
            /*
            TheEvent::CodeEditorApply(_id) => {
                let mut atom: Option<TheCodeAtom> = None;

                if let Some(code_list_selection) = &self.code_list_selection {
                    if let Some(widget) = ui.get_widget_id(code_list_selection.uuid) {
                        if let Some(name) = widget.value().to_string() {
                            atom = Some(self.create_atom(name.as_str()));
                        }
                    }
                }

                if let Some(atom) = atom {
                    self.set_selected_atom(ui, atom);
                    self.set_grid_selection_ui(ui, ctx);
                    redraw = true;
                }
            }*/
            TheEvent::DragStarted(id) => {
                if id.name == "Code List Item" {
                    if let Some(code_list_selection) = &self.code_list_selection {
                        if let Some(widget) = ui.get_widget_id(code_list_selection.uuid) {
                            if let Some(name) = widget.value().to_string() {
                                if let Some(atom) = Some(self.create_atom(name.as_str())) {
                                    let mut drop = TheDrop::new(TheId::named("Code Editor Atom"));
                                    drop.set_data(atom.to_json());
                                    drop.set_title(name);
                                    ui.style.create_drop_image(&mut drop, ctx);
                                    ctx.ui.set_drop(drop);
                                }
                            }
                        }
                    }
                }
            }
            // TheEvent::CodeEditorDelete(_id) => {
            //     if let Some(selection) = self.grid_selection {
            //         if let Some(layout) = ui.get_code_layout("Code Editor") {
            //             if let Some(code_view) = layout.code_view_mut().as_code_view() {
            //                 code_view.codegrid_mut().code.remove_entry(&selection);
            //             }
            //         }
            //     }

            //     self.set_grid_selection_ui(ui, ctx);
            //     self.set_grid_status_message(ui, ctx);
            //     redraw = true;
            // }
            TheEvent::CodeEditorSelectionChanged(_id, selection) => {
                self.grid_selection = *selection;

                self.set_grid_selection_ui(ui, ctx);
                self.set_grid_status_message(ui, ctx);
                redraw = true;
            }
            TheEvent::StateChanged(id, _state) => {
                if id.name == "Code List Item" {
                    self.code_list_selection = Some(id.clone());

                    /*
                    let mut atom: Option<TheCodeAtom> = None;

                    if let Some(widget) = ui.get_widget_id(id.uuid) {
                        if let Some(name) = widget.value().to_string() {
                            atom = Some(self.create_atom(name.as_str()));
                        }
                    }

                    if let Some(atom) = atom {
                        if let Some(grid_selection) = &self.grid_selection {
                            if let Some(layout) = ui.get_code_layout("Code Editor") {
                                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                                    code_view.set_grid_atom(*grid_selection, atom);
                                    self.set_grid_selection_ui(ui, ctx);
                                    // ctx.ui.send(TheEvent::CodeEditorSelectionChanged(
                                    //     id.clone(),
                                    //     Some(*grid_selection),
                                    // ));
                                }
                            }
                        }
                    }*/
                }

                redraw = true;
            }
            TheEvent::ValueChanged(id, value) => {
                if id.name == "Code Zoom" {
                    if let Some(v) = value.to_f32() {
                        if let Some(layout) = ui.get_code_layout("Code Editor") {
                            if let Some(code_view) = layout.code_view_mut().as_code_view() {
                                code_view.set_zoom(v);
                                ctx.ui.relayout = true;
                            }
                        }
                    }
                } else if id.name == "Atom Func Def" {
                    if let Some(name) = value.to_string() {
                        if !name.is_empty() {
                            self.set_selected_atom(ui, TheCodeAtom::FuncDef(name));
                        }
                    }
                }  else if id.name == "Atom Func Arg" {
                    if let Some(name) = value.to_string() {
                        if !name.is_empty() {
                            self.set_selected_atom(ui, TheCodeAtom::FuncArg(name));
                        }
                    }
                }  else if id.name == "Atom Func Call" {
                    if let Some(name) = value.to_string() {
                        if !name.is_empty() {
                            self.set_selected_atom(ui, TheCodeAtom::FuncCall(name));
                        }
                    }
                } else if id.name == "Atom Local Get" {
                    if let Some(name) = value.to_string() {
                        if !name.is_empty() {
                            self.set_selected_atom(ui, TheCodeAtom::LocalGet(name));
                        }
                    }
                } else if id.name == "Atom Local Set" {
                    if let Some(name) = value.to_string() {
                        if !name.is_empty() {
                            self.set_selected_atom(ui, TheCodeAtom::LocalSet(name));
                        }
                    }
                } else if id.name == "Atom Integer" {
                    if let Some(v) = value.to_i32() {
                        self.start_undo(ui);
                        self.set_selected_atom(ui, TheCodeAtom::Value(TheValue::Int(v)));
                        self.finish_undo(ui, ctx);
                    }
                }
                redraw = true;
            }
            _ => {}
        }

        redraw
    }

    /// Gets the codegrid from the editor
    pub fn get_codegrid(&mut self, ui: &mut TheUI) -> TheCodeGrid {
        if let Some(layout) = ui.get_code_layout("Code Editor") {
            if let Some(code_view) = layout.code_view_mut().as_code_view() {
                return code_view.codegrid().clone();
            }
        }
        TheCodeGrid::new()
    }

    /// Sets the codegrid to the editor
    pub fn set_codegrid(&mut self, codegrid: TheCodeGrid, ui: &mut TheUI) {
        if let Some(layout) = ui.get_code_layout("Code Editor") {
            if let Some(code_view) = layout.code_view_mut().as_code_view() {
                code_view.set_codegrid(codegrid);
            }
        }
    }

    /// Sets the UI of the currently selected atom into the top toolbar.
    pub fn set_grid_selection_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        if let Some(atom) = self.get_selected_atom(ui) {
            if let Some(layout) = ui.get_hlayout("Code Top Toolbar") {
                layout.clear();
                atom.to_layout(layout);
                layout.relayout(ctx);
                ctx.ui.redraw_all = true;
            }
        } else if let Some(layout) = ui.get_hlayout("Code Top Toolbar") {
            layout.clear();
            ctx.ui.redraw_all = true;
        }
    }

    /// Returns a clone of the currently selected atom (if any).
    pub fn get_selected_atom(&mut self, ui: &mut TheUI) -> Option<TheCodeAtom> {
        if let Some(grid_selection) = self.grid_selection {
            if let Some(layout) = ui.get_code_layout("Code Editor") {
                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                    let grid = code_view.codegrid();

                    if let Some(atom) = grid.code.get(&grid_selection) {
                        return Some(atom.clone());
                    }
                }
            }
        }
        None
    }

    /// Set the atom at the current position.
    pub fn set_selected_atom(&mut self, ui: &mut TheUI, atom: TheCodeAtom) {
        if let Some(grid_selection) = self.grid_selection {
            if let Some(layout) = ui.get_code_layout("Code Editor") {
                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                    code_view.set_grid_atom(grid_selection, atom);
                }
            }
        }
    }

    /// Set grid status message
    pub fn set_grid_status_message(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        let mut message: Option<TheCodeGridMessage> = None;
        if let Some(grid_selection) = self.grid_selection {
            if let Some(layout) = ui.get_code_layout("Code Editor") {
                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                    message = code_view.codegrid().message(grid_selection);
                }
            }
        }

        if let Some(text) = ui.get_text("Code Grid Status") {
            if let Some(message) = message {
                text.set_text(message.message);
            } else {
                text.set_text("".to_string());
            }
            ctx.ui.relayout = true;
        }
    }

    /// Start undo by setting the undo data.
    pub fn start_undo(&mut self, ui: &mut TheUI) {
        let mut undo = TheUndo::new(TheId::named("Code Editor"));
        undo.set_undo_data(self.get_codegrid_json(ui));
        self.undo = Some(undo);
    }

    /// Finish undo by adding the redo data and add to undo stack.
    pub fn finish_undo(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        if self.undo.is_none() {
            return;
        }

        let mut undo = self.undo.take().unwrap();
        undo.set_redo_data(self.get_codegrid_json(ui));

        ctx.ui.undo_stack.add(undo);
    }

    /// Get the codegrid as json
    pub fn get_codegrid_json(&mut self, ui: &mut TheUI) -> String {
        if let Some(layout) = ui.get_code_layout("Code Editor") {
            if let Some(code_view) = layout.code_view_mut().as_code_view() {
                return code_view.codegrid().to_json();
            }
        }
        "".to_string()
    }

    /// Set the codegrid from json
    pub fn set_codegrid_json(&mut self, json: String, ui: &mut TheUI) {
        if let Some(layout) = ui.get_code_layout("Code Editor") {
            if let Some(code_view) = layout.code_view_mut().as_code_view() {
                code_view.set_codegrid(TheCodeGrid::from_json(json.as_str()));
            }
        }
    }

    /// Create an atom for the given name.
    pub fn create_atom(&self, name: &str) -> TheCodeAtom {
        match name {
            "Function" => TheCodeAtom::FuncDef("Name".to_string()),
            "Function Argument" => TheCodeAtom::FuncArg("Name".to_string()),
            "Function Call" => TheCodeAtom::FuncCall("Name".to_string()),
            "Return" => TheCodeAtom::Return,
            "Local Get" => TheCodeAtom::LocalGet("Name".to_string()),
            "Local Set" => TheCodeAtom::LocalSet("Name".to_string()),
            "Integer" => TheCodeAtom::Value(TheValue::Int(1)),
            "Float" => TheCodeAtom::Value(TheValue::Float(1.0)),
            "Add" => TheCodeAtom::Add,
            "Multiply" => TheCodeAtom::Multiply,
            _ => TheCodeAtom::EndOfCode,
        }
    }

    /// Builds the UI canvas
    pub fn build_canvas(&self, ctx: &mut TheContext) -> TheCanvas {
        let mut canvas: TheCanvas = TheCanvas::new();

        // Left code list

        let mut list_canvas: TheCanvas = TheCanvas::new();

        let mut code_layout = TheListLayout::new(TheId::named("Code List"));
        code_layout.limiter_mut().set_max_width(150);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Function".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Function Argument".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Function Call".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Return".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Local Get".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Local Set".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Integer".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Float".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Add".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Multiply".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        code_layout.select_first_item(ctx);
        list_canvas.set_layout(code_layout);

        // ---

        let mut list_toolbar_canvas = TheCanvas::new();

        let mut toolbar_hlayout = TheHLayout::new(TheId::empty());

        let icon_color = [85, 81, 85, 255];
        let icon_border_color = [174, 174, 174, 255];

        let mut syntax_icon = TheTraybarButton::new(TheId::named("Code List Control"));
        syntax_icon.set_status_text("Show all keywords.");
        syntax_icon.limiter_mut().set_max_size(vec2i(24, 24));
        let mut buffer = TheRGBABuffer::new(TheDim::new(0, 0, 22, 22));
        buffer.pixels_mut().fill(0);
        let icon_stride = buffer.stride();
        let icon_rect = buffer.dim().to_buffer_utuple();
        ctx.draw.circle_with_border(buffer.pixels_mut(), &icon_rect, icon_stride, &icon_border_color, 10.5, &icon_border_color, 0.0);
        syntax_icon.set_icon(buffer);
        toolbar_hlayout.add_widget(Box::new(syntax_icon));

        let mut values_icon = TheTraybarButton::new(TheId::named("Code List Types"));
        values_icon.set_status_text("Show all value types.");
        values_icon.limiter_mut().set_max_size(vec2i(24, 24));
        buffer = TheRGBABuffer::new(TheDim::new(0, 0, 22, 22));
        buffer.pixels_mut().fill(0);
        let icon_stride = buffer.stride();
        let icon_rect = buffer.dim().to_buffer_utuple();
        ctx.draw.hexagon_with_border(buffer.pixels_mut(), &icon_rect, icon_stride, &icon_color, &icon_border_color, 0.0);
        values_icon.set_icon(buffer);
        toolbar_hlayout.add_widget(Box::new(values_icon));

        let mut operators_icon = TheTraybarButton::new(TheId::named("Code List Operators"));
        operators_icon.set_status_text("Show all operators.");
        operators_icon.limiter_mut().set_max_size(vec2i(24, 24));
        buffer = TheRGBABuffer::new(TheDim::new(0, 0, 22, 22));
        buffer.pixels_mut().fill(0);
        let icon_stride = buffer.stride();
        let icon_rect = buffer.dim().to_buffer_utuple();
        ctx.draw.rhombus_with_border (buffer.pixels_mut(), &icon_rect, icon_stride, &icon_color, &icon_border_color, 0.0);
        operators_icon.set_icon(buffer);
        toolbar_hlayout.add_widget(Box::new(operators_icon));

        let mut functions_icon = TheTraybarButton::new(TheId::named("Code List Functions"));
        functions_icon.set_status_text("Show all available functions.");
        functions_icon.limiter_mut().set_max_size(vec2i(24, 24));
        buffer = TheRGBABuffer::new(TheDim::new(0, 0, 22, 22));
        buffer.pixels_mut().fill(0);
        let icon_stride = buffer.stride();
        let mut icon_rect = buffer.dim().to_buffer_utuple();
        icon_rect.1 += 4;
        icon_rect.3 -= 6;
        ctx.draw.rounded_rect_with_border (buffer.pixels_mut(), &icon_rect, icon_stride, &icon_color, &(5.0, 5.0, 5.0, 5.0), &icon_border_color, 0.0);
        functions_icon.set_icon(buffer);
        toolbar_hlayout.add_widget(Box::new(functions_icon));

        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
        toolbar_hlayout.set_padding(10);
        list_toolbar_canvas.set_layout(toolbar_hlayout);
        list_toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));
        list_canvas.set_top(list_toolbar_canvas);

        // Top Toolbar
        let mut top_toolbar_canvas = TheCanvas::new();
        let mut toolbar_hlayout = TheHLayout::new(TheId::named("Code Top Toolbar"));
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
        //toolbar_hlayout.limiter_mut().set_max_height(27);
        top_toolbar_canvas.set_layout(toolbar_hlayout);
        top_toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));

        // Bottom Toolbar
        let mut bottom_toolbar_canvas = TheCanvas::new();

        let mut compile_button = TheTraybarButton::new(TheId::named("Compile"));
        compile_button.set_text("Compile".to_string());

        let mut text = TheText::new(TheId::empty());
        text.set_text("Zoom".to_string());

        let mut zoom = TheSlider::new(TheId::named("Code Zoom"));
        zoom.set_value(TheValue::Float(1.0));
        zoom.set_range(TheValue::RangeF32(0.3..=3.0));
        zoom.set_continuous(true);
        zoom.limiter_mut().set_max_width(120);

        let mut status_text = TheText::new(TheId::named("Code Grid Status"));
        status_text.set_text("".to_string());

        let divider1 = TheHDivider::new(TheId::empty());
        let divider2 = TheHDivider::new(TheId::empty());

        let mut toolbar_hlayout = TheHLayout::new(TheId::empty());
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
        toolbar_hlayout.add_widget(Box::new(compile_button));
        toolbar_hlayout.add_widget(Box::new(divider1));
        toolbar_hlayout.add_widget(Box::new(text));
        toolbar_hlayout.add_widget(Box::new(zoom));
        toolbar_hlayout.add_widget(Box::new(divider2));
        toolbar_hlayout.add_widget(Box::new(status_text));
        toolbar_hlayout.limiter_mut().set_max_height(27);

        bottom_toolbar_canvas.set_layout(toolbar_hlayout);
        bottom_toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));

        // ---

        let code_layout = TheCodeLayout::new(TheId::named("Code Editor"));

        canvas.set_layout(code_layout);
        canvas.set_left(list_canvas);
        canvas.set_top(top_toolbar_canvas);
        canvas.set_bottom(bottom_toolbar_canvas);

        canvas
    }
}