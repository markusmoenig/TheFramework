use crate::prelude::*;

pub struct Sidebar {
    pub editor_selection: Option<(u32, u32)>,
    code_list_selection: Option<TheId>,
}

#[allow(clippy::new_without_default)]
impl Sidebar {
    pub fn new() -> Self {
        Self {
            editor_selection: None,
            code_list_selection: None,
        }
    }

    pub fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        let width = 320;

        let mut text_layout: TheTextLayout = TheTextLayout::new(TheId::named("Values Layout"));
        text_layout.limiter_mut().set_max_width(width);

        // let name_edit = TheTextLineEdit::new(TheId::named("Region Name Edit"));
        // text_layout.add_pair("Name".to_string(), Box::new(name_edit));

        // List

        /*
        let mut list_header = TheCanvas::new();
        let mut switchbar = TheSwitchbar::new(TheId::empty());
        switchbar.set_text("Available Codes List".to_string());
        list_header.set_widget(switchbar);

        let mut list_canvas = TheCanvas::new();

        let mut code_layout = ui.create_code_list(ctx);
        code_layout.limiter_mut().set_max_size(vec2i(width, 400));
        list_canvas.set_layout(code_layout);
        list_canvas.set_top(list_header);

        let mut apply_button = TheTraybarButton::new(TheId::named("Apply Code"));
        apply_button.set_disabled(true);
        apply_button.set_text("Apply Code".to_string());

        let mut toolbar_hlayout = TheHLayout::new(TheId::empty());
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
        toolbar_hlayout.add_widget(Box::new(apply_button));

        let mut toolbar_canvas = TheCanvas::default();
        toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));
        toolbar_canvas.set_layout(toolbar_hlayout);
        list_canvas.set_bottom(toolbar_canvas);
        */

        //

        let mut settings = TheCanvas::new();

        let mut settings_header = TheCanvas::new();
        let mut switchbar = TheSwitchbar::new(TheId::empty());
        switchbar.set_text("Code Settings".to_string());
        settings_header.set_widget(switchbar);

        settings.set_top(settings_header);
        settings.set_layout(TheTextLayout::new(TheId::empty()));

        let mut canvas: TheCanvas = TheCanvas::new();

        settings.limiter_mut().set_max_width(width);
        canvas.set_center(settings);
        //canvas.set_top(list_canvas);

        ui.canvas.set_right(canvas);
    }

    pub fn handle_event(&mut self, event: &TheEvent, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        match event {
            TheEvent::CodeEditorSelectionChanged(_id, selection) => {
                ui.set_widget_disabled_state(
                    "Apply Code",
                    ctx,
                    selection.is_none() || self.code_list_selection.is_none(),
                );
                self.editor_selection = *selection;

                /*
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
            TheEvent::StateChanged(id, state) => {
                //println!("app Widget State changed {:?}: {:?}", id, state);

                /*
                if id.name == "Apply Code" {
                    let mut atom: Option<TheAtom> = None;

                    if let Some(code_list_selection) = &self.code_list_selection {
                        if let Some(widget) = ui.get_widget_id(code_list_selection.uuid) {
                            if let Some(name) = widget.value().to_string() {
                                atom = Some(ui.create_code_atom(name.as_str()));
                            }
                        }
                    }

                    if let Some(atom) = atom {
                        if let Some(selection) = &self.editor_selection {
                            if let Some(layout) = ui.get_code_layout("Code Editor") {
                                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                                    code_view.set_grid_atom(*selection, atom);
                                    ctx.ui.send(TheEvent::CodeEditorSelectionChanged(
                                        id.clone(),
                                        Some(*selection),
                                    ));
                                }
                            }
                        }
                    }
                } else if id.name == "Code List Item" && *state == TheWidgetState::Selected {
                    self.code_list_selection = Some(id.clone());
                    ui.set_widget_disabled_state(
                        "Apply Code",
                        ctx,
                        self.editor_selection.is_none() || self.code_list_selection.is_none(),
                    );
                }*/

                redraw = true;
            }
            TheEvent::FileRequesterResult(id, paths) => {
                println!("FileRequester Result {:?} {:?}", id, paths);
            }
            _ => {}
        }

        redraw
    }
}
