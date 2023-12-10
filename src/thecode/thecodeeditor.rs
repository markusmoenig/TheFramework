use crate::prelude::*;

pub struct TheCodeEditor {
    code_list_selection: Option<TheId>,
    grid_selection: Option<(u32, u32)>,
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
        }
    }

    pub fn handle_event(&mut self, event: &TheEvent, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        match event {
            TheEvent::CodeEditorApply(_id) => {
                let mut atom: Option<TheAtom> = None;

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
            }
            TheEvent::CodeEditorSelectionChanged(_id, selection) => {
                self.grid_selection = *selection;
                self.set_grid_selection_ui(ui, ctx);
                self.set_grid_status_message(ui, ctx);
                redraw = true;
                /*
                ui.set_widget_disabled_state(
                    "Apply Code",
                    ctx,
                    selection.is_none() || self.code_list_selection.is_none(),
                );
                self.editor_selection = *selection;

                // Generate the Atom UI
                let mut text_layout = TheTextLayout::new(TheId::empty());
                if let Some(selection) = selection {
                    if let Some(layout) = ui.get_code_layout("Code Editor") {
                        if let Some(code_view) = layout.code_view_mut().as_code_view() {
                            let grid = code_view.code_grid();

                            if let Some(atom) = grid.code.get(selection) {
                                text_layout = atom.to_text_layout();
                            }
                        }
                    }
                }

                ui.canvas
                    .right
                    .as_mut()
                    .unwrap()
                    .center
                    .as_mut()
                    .unwrap()
                    .set_layout(text_layout);
                ctx.ui.relayout = true;
                */
            }
            TheEvent::StateChanged(id, _state) => {
                if id.name == "Code List Item" {
                    self.code_list_selection = Some(id.clone());

                    /*
                    let mut atom: Option<TheAtom> = None;

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
                            self.set_selected_atom(ui, TheAtom::FuncDef(name));
                        }
                    }
                } else if id.name == "Atom Local Set" {
                    if let Some(name) = value.to_string() {
                        if !name.is_empty() {
                            self.set_selected_atom(ui, TheAtom::LocalSet(name));
                        }
                    }
                } else if id.name == "Atom Integer" {
                    if let Some(v) = value.to_i32() {
                        self.set_selected_atom(ui, TheAtom::Value(TheValue::Int(v)));
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
    pub fn get_selected_atom(&mut self, ui: &mut TheUI) -> Option<TheAtom> {
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
    pub fn set_selected_atom(&mut self, ui: &mut TheUI, atom: TheAtom) {
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

    /// Create an atom for the given name.
    pub fn create_atom(&self, name: &str) -> TheAtom {
        match name {
            "Func Def" => TheAtom::FuncDef("Name".to_string()),
            "Func Call" => TheAtom::FuncCall("Name".to_string()),
            "Return" => TheAtom::Return,
            "Local Get" => TheAtom::LocalGet("Name".to_string()),
            "Local Set" => TheAtom::LocalSet("Name".to_string()),
            "Integer" => TheAtom::Value(TheValue::Int(1)),
            "Add" => TheAtom::Add,
            "Multiply" => TheAtom::Multiply,
            _ => TheAtom::EndOfCode,
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
        item.set_text("Func Def".to_string());
        item.set_associated_layout(code_layout.id().clone());
        code_layout.add_item(item, ctx);

        let mut item = TheListItem::new(TheId::named("Code List Item"));
        item.set_text("Func Call".to_string());
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
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
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
