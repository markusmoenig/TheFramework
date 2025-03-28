use crate::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use theframework::prelude::*;

pub struct CodeEditor {
    project: Project,
    project_path: Option<PathBuf>,

    editor: TheCodeEditor,
    compiler: TheCompiler,
    right_width: i32,

    event_receiver: Option<Receiver<TheEvent>>,
}

impl TheTrait for CodeEditor {
    fn new() -> Self
    where
        Self: Sized,
    {
        let mut editor = TheCodeEditor::default();

        editor.add_external(TheExternalCode::new(
            "Pulse".to_string(),
            "Counts up to a certain value and returns true on completion. Then restarts."
                .to_string(),
            vec!["Count to".to_string()],
            vec![TheValue::Int(4)],
            Some(TheValue::Bool(false)),
        ));

        editor.add_external(TheExternalCode::new(
            "Doubler".to_string(),
            "Doubles the input value.".to_string(),
            vec!["Value".to_string()],
            vec![TheValue::Float(0.0)],
            Some(TheValue::Bool(false)),
        ));

        let mut compiler = TheCompiler::default();
        compiler.add_external_call(
            "Pulse".to_string(),
            |stack: &mut Vec<TheValue>,
             data: &mut TheCodeNodeData,
             sandbox: &mut TheCodeSandbox| {
                let count = data.values[0].to_i32().unwrap();
                let mut max_value = data.values[1].to_i32().unwrap();

                let stack_v = stack.pop();

                // If the max value is zero, this is the first call, compute it.
                if let Some(v) = &stack_v {
                    if max_value == 0 {
                        if let Some(int) = v.to_i32() {
                            max_value = int;
                        }
                    }
                }

                if count < max_value {
                    data.values[0] = TheValue::Int(count + 1);
                    if sandbox.debug_mode {
                        sandbox.set_debug_value(
                            data.location,
                            (
                                Some(TheValue::Text(format!("{} / {}", count, max_value))),
                                TheValue::Bool(false),
                            ),
                        );
                    }
                    stack.push(TheValue::Bool(false));
                    TheCodeNodeCallResult::Continue
                } else {
                    if sandbox.debug_mode {
                        sandbox.set_debug_value(
                            data.location,
                            (
                                Some(TheValue::Text(format!("{} / {}", count, max_value))),
                                TheValue::Bool(true),
                            ),
                        );
                    }
                    data.values[0] = TheValue::Int(0);
                    if let Some(stack_v) = stack_v {
                        if let Some(int) = stack_v.to_i32() {
                            data.values[1] = TheValue::Int(int);
                        }
                    }
                    if !data.sub_functions.is_empty() {
                        _ = data.sub_functions[0].execute(sandbox).pop();
                    }
                    stack.push(TheValue::Bool(true));
                    TheCodeNodeCallResult::Continue
                }
            },
            vec![TheValue::Int(0), TheValue::Int(0)],
        );

        compiler.add_external_call(
            "Doubler".to_string(),
            |stack: &mut Vec<TheValue>,
             _data: &mut TheCodeNodeData,
             _sandbox: &mut TheCodeSandbox| {
                let mut done = false;

                if let Some(stack_v) = stack.pop() {
                    if let Some(value) = stack_v.mul(&TheValue::Float(2.0)) {
                        stack.push(value);
                        done = true;
                    }
                }

                if !done {
                    // Runtime error, no value on the stack.
                    stack.push(TheValue::Float(0.0));
                }

                TheCodeNodeCallResult::Continue
            },
            vec![],
        );

        Self {
            project: Project::new(),
            project_path: None,

            right_width: 280,
            editor,

            compiler,

            event_receiver: None,
        }
    }

    fn window_title(&self) -> String {
        "CodeGrid Editor".to_string()
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        // Menu
        let mut menu_canvas = TheCanvas::new();
        let mut menu = TheMenu::new(TheId::named("Menu"));

        let mut file_menu = TheContextMenu::named(str!("File"));
        file_menu.add(TheContextMenuItem::new(
            str!("Open..."),
            TheId::named("Open"),
        ));
        file_menu.add(TheContextMenuItem::new(str!("Save"), TheId::named("Save")));
        file_menu.add(TheContextMenuItem::new(
            str!("Save As ..."),
            TheId::named("Save As"),
        ));
        let mut edit_menu = TheContextMenu::named(str!("Edit"));
        edit_menu.add(TheContextMenuItem::new(str!("Undo"), TheId::named("Undo")));
        edit_menu.add(TheContextMenuItem::new(str!("Redo"), TheId::named("Redo")));
        edit_menu.add_separator();
        edit_menu.add(TheContextMenuItem::new(str!("Cut"), TheId::named("Cut")));
        edit_menu.add(TheContextMenuItem::new(str!("Copy"), TheId::named("Copy")));
        edit_menu.add(TheContextMenuItem::new(
            str!("Paste"),
            TheId::named("Paste"),
        ));

        let mut code_menu = TheContextMenu::named(str!("Code"));
        code_menu.add(self.editor.create_keywords_context_menu_item());
        code_menu.add(self.editor.create_operators_context_menu_item());
        code_menu.add(self.editor.create_values_context_menu_item());
        code_menu.add(self.editor.create_functions_context_menu_item());

        menu.add_context_menu(file_menu);
        menu.add_context_menu(edit_menu);
        menu.add_context_menu(code_menu);

        menu_canvas.set_widget(menu);

        self.editor.init_menu_selection(ctx);

        // Top
        let mut top_canvas = TheCanvas::new();

        let mut menubar = TheMenubar::new(TheId::named("Menubar"));
        menubar.limiter_mut().set_max_height(43 + 22);

        let mut open_button = TheMenubarButton::new(TheId::named("Open"));
        open_button.set_icon_name("icon_role_load".to_string());

        let mut save_button = TheMenubarButton::new(TheId::named("Save"));
        save_button.set_icon_name("icon_role_save".to_string());

        let mut save_as_button = TheMenubarButton::new(TheId::named("Save As"));
        save_as_button.set_icon_name("icon_role_save_as".to_string());
        save_as_button.set_icon_offset(Vec2::new(2, -5));

        let mut undo_button = TheMenubarButton::new(TheId::named("Undo"));
        undo_button.set_icon_name("icon_role_undo".to_string());

        let mut redo_button = TheMenubarButton::new(TheId::named("Redo"));
        redo_button.set_icon_name("icon_role_redo".to_string());

        let mut hlayout = TheHLayout::new(TheId::named("Menu Layout"));
        hlayout.set_background_color(None);
        hlayout.set_margin(Vec4::new(40, 5, 20, 0));
        hlayout.add_widget(Box::new(open_button));
        hlayout.add_widget(Box::new(save_button));
        hlayout.add_widget(Box::new(save_as_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(undo_button));
        hlayout.add_widget(Box::new(redo_button));

        top_canvas.set_widget(menubar);
        top_canvas.set_layout(hlayout);
        top_canvas.set_top(menu_canvas);

        // Side

        let bundle_canvas =
            self.editor
                .set_bundle(self.project.bundle.clone(), ctx, self.right_width, None);
        ui.canvas.set_right(bundle_canvas);

        let mut status_canvas = TheCanvas::new();
        let mut statusbar = TheStatusbar::new(TheId::named("Statusbar"));
        statusbar.set_text("Welcome to TheFramework!".to_string());
        status_canvas.set_widget(statusbar);

        //

        ui.canvas.set_top(top_canvas);
        ui.canvas.set_bottom(status_canvas);
        ui.canvas.set_center(self.editor.build_canvas(ctx));
        ui.set_statusbar_name("Statusbar".to_string());

        ctx.ui.set_disabled("Save");
        ctx.ui.set_disabled("Save As");
        ctx.ui.set_disabled("Undo");
        ctx.ui.set_disabled("Redo");

        self.event_receiver = Some(ui.add_state_listener("Main Receiver".into()));
    }

    #[allow(clippy::single_match)]
    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                redraw = self.editor.handle_event(&event, ui, ctx);
                match event {
                    TheEvent::ContextMenuSelected(_, action) => {
                        if action.name.starts_with("Code") {
                            self.editor.insert_context_menu_id(action, ui, ctx);
                        }
                    }
                    TheEvent::FileRequesterResult(id, paths) => {
                        if id.name == "Open" {
                            for p in paths {
                                self.project_path = Some(p.clone());
                                let contents = std::fs::read_to_string(p).unwrap_or("".to_string());
                                self.project =
                                    serde_json::from_str(&contents).unwrap_or(Project::new());
                                ui.canvas.set_right(self.editor.set_bundle(
                                    self.project.bundle.clone(),
                                    ctx,
                                    self.right_width,
                                    None,
                                ));
                                redraw = true;
                                ctx.ui.send(TheEvent::SetStatusText(
                                    TheId::empty(),
                                    "Project loaded successfully.".to_string(),
                                ))
                            }
                        } else if id.name == "Save As" {
                            self.project.bundle = self.editor.get_bundle();
                            for p in paths {
                                let json = serde_json::to_string(&self.project);
                                if let Ok(json) = json {
                                    if std::fs::write(p, json).is_ok() {
                                        ctx.ui.send(TheEvent::SetStatusText(
                                            TheId::empty(),
                                            "Project saved successfully.".to_string(),
                                        ))
                                    } else {
                                        ctx.ui.send(TheEvent::SetStatusText(
                                            TheId::empty(),
                                            "Unable to save project!".to_string(),
                                        ))
                                    }
                                }
                            }
                        }
                    }
                    TheEvent::CodeBundleChanged(_, _) => {
                        redraw = true;
                    }
                    TheEvent::StateChanged(id, _state) => {
                        if id.name == "Open" {
                            #[cfg(not(target_arch = "wasm32"))]
                            ctx.ui.open_file_requester(
                                TheId::named_with_id(id.name.as_str(), Uuid::new_v4()),
                                "Open".into(),
                                TheFileExtension::new(
                                    "CodeGridFX".into(),
                                    vec!["codegridfx".to_string()],
                                ),
                            );
                            ctx.ui
                                .set_widget_state("Open".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            redraw = true;
                        } else if id.name == "Save" {
                            self.project.bundle = self.editor.get_bundle();
                            if let Some(path) = &self.project_path {
                                let json = serde_json::to_string(&self.project);
                                if let Ok(json) = json {
                                    if std::fs::write(path, json).is_ok() {
                                        ctx.ui.send(TheEvent::SetStatusText(
                                            TheId::empty(),
                                            "Project saved successfully.".to_string(),
                                        ))
                                    } else {
                                        ctx.ui.send(TheEvent::SetStatusText(
                                            TheId::empty(),
                                            "Unable to save project!".to_string(),
                                        ))
                                    }
                                }
                            }
                        } else if id.name == "Save As" {
                            #[cfg(not(target_arch = "wasm32"))]
                            ctx.ui.save_file_requester(
                                TheId::named_with_id(id.name.as_str(), Uuid::new_v4()),
                                "Save".into(),
                                TheFileExtension::new(
                                    "CodeGridFX".into(),
                                    vec!["codegridfx".to_string()],
                                ),
                            );
                            ctx.ui
                                .set_widget_state("Save".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            redraw = true;
                        } else if id.name == "Compile" {
                            if let Some(layout) = ui.get_code_layout("Code Editor") {
                                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                                    let grid = code_view.codegrid_mut();

                                    let rc = self.compiler.compile(grid);

                                    if let Ok(mut module) = rc {
                                        let mut sandbox = TheCodeSandbox::new();
                                        sandbox.debug_mode = true;

                                        // sandbox.add_global(
                                        //     "test",
                                        //     TheCodeNode::new(
                                        //         |_, data, _| {
                                        //             println!("inside test {:?}", data.location);
                                        //             if let Some(i) = data.values[0].to_i32() {
                                        //                 println!("i: {:?}", i);
                                        //                 data.values[0] = TheValue::Int(i + 1);
                                        //             }
                                        //             TheCodeNodeCallResult::Continue
                                        //         },
                                        //         TheCodeNodeData::values(vec![TheValue::Int(0)]),
                                        //     ),
                                        //     vec![TheCodeAtom::NamedValue("Count".to_string(), TheValue::Int(4))]
                                        // );

                                        //sandbox.insert_module(module.clone());
                                        module.execute(&mut sandbox);
                                        code_view.set_debug_module(
                                            sandbox.get_module_debug_module(module.id),
                                        );
                                    } else {
                                        code_view.set_debug_module(TheDebugModule::default());
                                    }
                                }
                            }
                        } else {
                            let mut data: Option<(TheId, String)> = None;
                            if id.name == "Undo" && ctx.ui.undo_stack.has_undo() {
                                data = Some(ctx.ui.undo_stack.undo());
                            } else if id.name == "Redo" && ctx.ui.undo_stack.has_redo() {
                                data = Some(ctx.ui.undo_stack.redo());
                            }

                            if let Some((id, json)) = data {
                                if id.name == "Code Editor" {
                                    self.editor.set_codegrid_json(json, ui);
                                    self.editor.set_grid_selection_ui(ui, ctx);
                                }
                                redraw = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        redraw
    }
}
