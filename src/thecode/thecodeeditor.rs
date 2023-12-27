use crate::prelude::*;

pub struct TheCodeEditor {
    code_list_selection: Option<TheId>,
    grid_selection: Option<(u16, u16)>,

    codegrid_selection: Option<TheId>,
    bundle: TheCodeBundle,

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
            codegrid_selection: None,

            bundle: TheCodeBundle::new(),

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
            TheEvent::SDFIndexChanged(_id, index) => {
                if let Some(code_list) = ui.get_list_layout("Code List") {
                    self.get_code_list_items(*index, code_list, ctx)
                }
            }
            TheEvent::DragStarted(id, text, offset) => {
                if id.name == "Code List Item" {
                    if let Some(atom) = Some(self.create_atom(text.as_str())) {
                        let mut drop = TheDrop::new(TheId::named("Code Editor Atom"));
                        drop.set_data(atom.to_json());
                        drop.set_title(text.clone());
                        drop.set_offset(*offset);
                        ui.style.create_drop_image(&mut drop, ctx);
                        ctx.ui.set_drop(drop);
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
            TheEvent::CodeEditorChanged(_id, codegrid) => {
                self.bundle.insert_grid(codegrid.clone());
                ctx.ui
                    .send(TheEvent::CodeBundleChanged(self.bundle.clone()));
            }
            TheEvent::CodeEditorSelectionChanged(_id, selection) => {
                self.grid_selection = *selection;

                self.set_grid_selection_ui(ui, ctx);
                self.set_grid_status_message(ui, ctx);
                redraw = true;
            }
            TheEvent::StateChanged(id, state) => {
                if id.name == "Code List Item" {
                    self.code_list_selection = Some(id.clone());
                } else if id.name == "CodeGrid List Add" {
                    if *state == TheWidgetState::Clicked {
                        let codegrid = TheCodeGrid::new();
                        self.bundle.insert_grid(codegrid.clone());

                        if let Some(code_list) = ui.get_list_layout("CodeGrid List") {
                            let item_id = TheId::named_with_id("CodeGrid List Item", codegrid.uuid);
                            let mut item = TheListItem::new(item_id.clone());
                            item.set_text(codegrid.name.clone());
                            item.set_associated_layout(code_list.id().clone());
                            item.set_state(TheWidgetState::Selected);
                            code_list.deselect_all();
                            code_list.add_item(item, ctx);

                            ctx.ui
                                .send_widget_state_changed(&item_id, TheWidgetState::Selected);
                        }

                        ui.set_widget_disabled_state("CodeGrid List Name", ctx, false);
                        ui.set_widget_disabled_state("CodeGrid List Remove", ctx, false);

                        ctx.ui
                            .send(TheEvent::CodeBundleChanged(self.bundle.clone()));

                        self.set_codegrid(codegrid.clone(), ui);
                        self.set_grid_selection_ui(ui, ctx);
                        self.set_grid_status_message(ui, ctx);
                    }
                } else if id.name == "CodeGrid List Remove" {
                    if *state == TheWidgetState::Clicked {
                        if let Some(codegrid_selection) = &self.codegrid_selection {
                            self.bundle.grids.remove(&codegrid_selection.uuid);
                            let mut disable = false;
                            if let Some(code_list) = ui.get_list_layout("CodeGrid List") {
                                code_list.remove(codegrid_selection.clone());
                                code_list.select_first_item(ctx);

                                disable = code_list.widgets().is_empty();
                            }

                            ui.set_widget_disabled_state("CodeGrid List Name", ctx, disable);
                            ui.set_widget_disabled_state("CodeGrid List Remove", ctx, disable);

                            self.codegrid_selection = None;

                            ctx.ui
                                .send(TheEvent::CodeBundleChanged(self.bundle.clone()));

                            self.set_grid_selection_ui(ui, ctx);
                            self.set_grid_status_message(ui, ctx);
                        }
                    }
                } else if id.name == "CodeGrid List Item" && *state == TheWidgetState::Selected {
                    if let Some(codegrid) = self.bundle.get_grid(&id.uuid) {
                        self.codegrid_selection = Some(id.clone());
                        if let Some(text_edit) = ui.get_text_line_edit("CodeGrid List Name") {
                            text_edit.set_text(codegrid.name.clone());
                        }

                        ui.set_widget_disabled_state("CodeGrid List Name", ctx, false);
                        ui.set_widget_disabled_state("CodeGrid List Remove", ctx, false);

                        self.set_codegrid(codegrid.clone(), ui);
                        self.set_grid_selection_ui(ui, ctx);
                        self.set_grid_status_message(ui, ctx);
                    }
                }

                redraw = true;
            }
            TheEvent::ValueChanged(id, value) => {
                if id.name == "CodeGrid List Name" {
                    if let Some(text) = value.to_string() {
                        if let Some(codegrid_selection) = &self.codegrid_selection {
                            if let Some(codegrid) =
                                self.bundle.get_grid_mut(&codegrid_selection.uuid)
                            {
                                if codegrid.name != text {
                                    if let Some(widget) = ui.get_widget_id(codegrid_selection.uuid)
                                    {
                                        widget.set_value(TheValue::Text(text.clone()));
                                        codegrid.name = text.clone();
                                        ctx.ui.relayout = true;
                                    }
                                }
                            }
                        }
                    }
                } else if id.name == "Code Zoom" {
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
                } else if id.name == "Atom Func Arg" {
                    if let Some(name) = value.to_string() {
                        if !name.is_empty() {
                            self.set_selected_atom(ui, TheCodeAtom::FuncArg(name));
                        }
                    }
                } else if id.name == "Atom Func Call" {
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
                } else if id.name == "Atom Object Set Object" {
                    if let Some(name) = value.to_string() {
                        if !name.is_empty() {
                            if let Some(TheCodeAtom::ObjectSet(_object, variable)) =
                                self.get_selected_atom(ui)
                            {
                                self.set_selected_atom(ui, TheCodeAtom::ObjectSet(name, variable));
                            }
                        }
                    }
                } else if id.name == "Atom Object Set Variable" {
                    if let Some(name) = value.to_string() {
                        if !name.is_empty() {
                            if let Some(TheCodeAtom::ObjectSet(object, _variable)) =
                                self.get_selected_atom(ui)
                            {
                                self.set_selected_atom(ui, TheCodeAtom::ObjectSet(object, name));
                            }
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
            "Assignment" => TheCodeAtom::Assignment("=".to_string()),
            "Function" => TheCodeAtom::FuncDef("name".to_string()),
            "Function Argument" => TheCodeAtom::FuncArg("name".to_string()),
            "Function Call" => TheCodeAtom::FuncCall("name".to_string()),
            "Return" => TheCodeAtom::Return,
            "Local Get" => TheCodeAtom::LocalGet("name".to_string()),
            "Local Set" => TheCodeAtom::LocalSet("name".to_string()),
            "Object Get" => TheCodeAtom::ObjectGet("self".to_string(), "name".to_string()),
            "Object Set" => TheCodeAtom::ObjectSet("self".to_string(), "name".to_string()),
            "Integer" => TheCodeAtom::Value(TheValue::Int(0)),
            "Float" => TheCodeAtom::Value(TheValue::Float(0.0)),
            "Float2" => TheCodeAtom::Value(TheValue::Float2(vec2f(0.0, 0.0))),
            "Float3" => TheCodeAtom::Value(TheValue::Float3(vec3f(0.0, 0.0, 0.0))),
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
        self.get_code_list_items(0, &mut code_layout, ctx);

        code_layout.select_first_item(ctx);
        list_canvas.set_layout(code_layout);

        // ---

        let mut list_toolbar_canvas = TheCanvas::new();

        let mut toolbar_hlayout = TheHLayout::new(TheId::empty());
        toolbar_hlayout.set_margin(vec4i(2, 2, 2, 2));
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_mode(TheHLayoutMode::SizeBased);

        let mut sdf_view = TheSDFView::new(TheId::named("Code List SDF View"));

        let mut sdf_canvas = TheSDFCanvas::new();
        sdf_canvas.background = crate::thecolor::TheColor::from_u8_array([118, 118, 118, 255]);
        sdf_canvas.selected = Some(0);
        sdf_canvas.add(
            TheSDF::Circle(TheDim::new(5, 2, 20, 20)),
            ThePattern::Solid(crate::thecolor::TheColor::from_u8(74, 74, 74, 255)),
        );
        sdf_view.set_status(0, "Show all keywords.".to_string());

        sdf_canvas.add(
            TheSDF::Hexagon(TheDim::new(65, 2, 20, 20)),
            ThePattern::Solid(crate::thecolor::TheColor::from_u8(74, 74, 74, 255)),
        );
        sdf_view.set_status(1, "Show all value types.".to_string());

        sdf_canvas.add(
            TheSDF::Rhombus(TheDim::new(125, 2, 20, 20)),
            ThePattern::Solid(crate::thecolor::TheColor::from_u8(74, 74, 74, 255)),
        );
        sdf_view.set_status(2, "Show all operators.".to_string());

        sdf_canvas.add(
            TheSDF::RoundedRect(TheDim::new(185, 2, 20, 20), (5.0, 5.0, 5.0, 5.0)),
            ThePattern::Solid(crate::thecolor::TheColor::from_u8(74, 74, 74, 255)),
        );
        sdf_view.set_status(3, "Show all available functions.".to_string());

        sdf_view.set_canvas(sdf_canvas);
        toolbar_hlayout.add_widget(Box::new(sdf_view));
        list_toolbar_canvas.set_layout(toolbar_hlayout);
        list_toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));
        list_canvas.set_top(list_toolbar_canvas);

        // Top Toolbar
        let mut top_toolbar_canvas = TheCanvas::new();
        let mut toolbar_hlayout = TheHLayout::new(TheId::named("Code Top Toolbar"));
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
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
        canvas.top_is_expanding = false;

        canvas
    }

    /// Sets the bundle and returns the list canvas for it.
    pub fn set_bundle(
        &mut self,
        bundle: TheCodeBundle,
        ctx: &mut TheContext,
        width: i32,
    ) -> TheCanvas {
        self.bundle = bundle;

        let mut canvas: TheCanvas = TheCanvas::new();

        let mut settings_header = TheCanvas::new();
        let mut switchbar = TheSwitchbar::new(TheId::empty());
        switchbar.set_text("Codes".to_string());
        settings_header.set_widget(switchbar);

        canvas.set_top(settings_header);

        // Grid list

        let mut list_canvas: TheCanvas = TheCanvas::new();
        list_canvas.limiter_mut().set_max_width(width);

        let mut code_layout = TheListLayout::new(TheId::named("CodeGrid List"));

        let keys = self.bundle.sorted();

        for key in &keys {
            if let Some(grid) = self.bundle.grids.get(key) {
                let mut item =
                    TheListItem::new(TheId::named_with_id("CodeGrid List Item", grid.uuid));
                item.set_text(grid.name.clone());
                item.set_associated_layout(code_layout.id().clone());
                code_layout.add_item(item, ctx);
            }
        }

        code_layout.select_first_item(ctx);
        list_canvas.set_layout(code_layout);

        canvas.set_center(list_canvas);

        // Toolbar

        let mut add_button = TheTraybarButton::new(TheId::named("CodeGrid List Add"));
        add_button.set_icon_name("icon_role_add".to_string());
        add_button.set_status_text("Add new code.");
        let mut remove_button = TheTraybarButton::new(TheId::named("CodeGrid List Remove"));
        remove_button.set_icon_name("icon_role_remove".to_string());
        remove_button.set_disabled(true);
        remove_button.set_status_text("Remove code.");

        let mut text_edit = TheTextLineEdit::new(TheId::named("CodeGrid List Name"));
        text_edit.limiter_mut().set_max_width(180);

        let mut toolbar_hlayout = TheHLayout::new(TheId::empty());
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
        toolbar_hlayout.add_widget(Box::new(add_button));
        toolbar_hlayout.add_widget(Box::new(remove_button));
        toolbar_hlayout.add_widget(Box::new(TheHDivider::new(TheId::empty())));
        toolbar_hlayout.add_widget(Box::new(text_edit));

        let mut toolbar_canvas = TheCanvas::default();
        toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));
        toolbar_canvas.set_layout(toolbar_hlayout);
        canvas.set_bottom(toolbar_canvas);

        canvas
    }

    pub fn get_code_list_items(&self, index: u32, code_layout: &mut dyn TheListLayoutTrait, ctx: &mut TheContext) {
        code_layout.clear();
        if index == 0 {
            let mut item = TheListItem::new(TheId::named("Code List Item"));
            item.set_text("Assignment".to_string());
            item.set_associated_layout(code_layout.id().clone());
            code_layout.add_item(item, ctx);

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
            item.set_text("Object Get".to_string());
            item.set_associated_layout(code_layout.id().clone());
            code_layout.add_item(item, ctx);

            let mut item = TheListItem::new(TheId::named("Code List Item"));
            item.set_text("Object Set".to_string());
            item.set_associated_layout(code_layout.id().clone());
            code_layout.add_item(item, ctx);
        }

        if index == 0 || index == 1 {
            let mut item = TheListItem::new(TheId::named("Code List Item"));
            item.set_text("Integer".to_string());
            item.set_associated_layout(code_layout.id().clone());
            code_layout.add_item(item, ctx);

            let mut item = TheListItem::new(TheId::named("Code List Item"));
            item.set_text("Float".to_string());
            item.set_associated_layout(code_layout.id().clone());
            code_layout.add_item(item, ctx);

            let mut item = TheListItem::new(TheId::named("Code List Item"));
            item.set_text("Float2".to_string());
            item.set_associated_layout(code_layout.id().clone());
            code_layout.add_item(item, ctx);

            let mut item = TheListItem::new(TheId::named("Code List Item"));
            item.set_text("Float3".to_string());
            item.set_associated_layout(code_layout.id().clone());
            code_layout.add_item(item, ctx);
        }

        if index == 0 || index == 2 {
            let mut item = TheListItem::new(TheId::named("Code List Item"));
            item.set_text("Add".to_string());
            item.set_associated_layout(code_layout.id().clone());
            code_layout.add_item(item, ctx);

            let mut item = TheListItem::new(TheId::named("Code List Item"));
            item.set_text("Multiply".to_string());
            item.set_associated_layout(code_layout.id().clone());
            code_layout.add_item(item, ctx);
        }
    }

    /// Returns the bundle
    pub fn get_bundle(&self) -> TheCodeBundle {
        self.bundle.clone()
    }
}
