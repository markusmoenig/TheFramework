use crate::prelude::*;
use std::sync::mpsc::Receiver;
use theframework::{prelude::*, thecode::{thecodesandbox::TheDebugModule, thecodenode::TheCodeNodeData}};

pub struct CodeEditor {
    sidebar: Sidebar,
    // browser: Browser,
    project: Project,
    editor: TheCodeEditor,

    event_receiver: Option<Receiver<TheEvent>>,
}

impl TheTrait for CodeEditor {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            sidebar: Sidebar::new(),
            // browser: Browser::new(),
            project: Project::new(),
            editor: TheCodeEditor::default(),

            event_receiver: None,
        }
    }

    fn window_title(&mut self) -> String {
        "CodeGrid Editor".to_string()
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        // Top
        let mut top_canvas = TheCanvas::new();

        let menubar = TheMenubar::new(TheId::named("Menubar"));

        let mut open_button = TheMenubarButton::new(TheId::named("Open"));
        open_button.set_icon_name("icon_role_load".to_string());

        let mut save_button = TheMenubarButton::new(TheId::named("Save"));
        save_button.set_icon_name("icon_role_save".to_string());

        let mut save_as_button = TheMenubarButton::new(TheId::named("Save As"));
        save_as_button.set_icon_name("icon_role_save_as".to_string());
        save_as_button.set_icon_offset(vec2i(2, -5));

        let mut undo_button = TheMenubarButton::new(TheId::named("Undo"));
        undo_button.set_icon_name("icon_role_undo".to_string());

        let mut redo_button = TheMenubarButton::new(TheId::named("Redo"));
        redo_button.set_icon_name("icon_role_redo".to_string());

        let mut hlayout = TheHLayout::new(TheId::named("Menu Layout"));
        hlayout.set_background_color(None);
        hlayout.set_margin(vec4i(40, 5, 20, 0));
        hlayout.add_widget(Box::new(open_button));
        hlayout.add_widget(Box::new(save_button));
        hlayout.add_widget(Box::new(save_as_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(undo_button));
        hlayout.add_widget(Box::new(redo_button));

        top_canvas.set_widget(menubar);
        top_canvas.set_layout(hlayout);

        // Right
        //self.sidebar.init_ui(ui, ctx);

        // Bottom

        //self.browser.init_ui(ui, ctx);

        let mut status_canvas = TheCanvas::new();
        let mut statusbar = TheStatusbar::new(TheId::named("Statusbar"));
        statusbar.set_text("Welcome to TheFramework!".to_string());
        status_canvas.set_widget(statusbar);

        //

        ui.canvas.set_top(top_canvas);
        ui.canvas.set_bottom(status_canvas);
        ui.canvas.set_center(self.editor.build_canvas(ctx));
        ui.set_statusbar_name("Statusbar".to_string());

        self.event_receiver = Some(ui.add_state_listener("Main Receiver".into()));
    }

    #[allow(clippy::single_match)]
    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                redraw = self.editor.handle_event(&event, ui, ctx);
                match event {
                    TheEvent::FileRequesterResult(id, paths) => {
                        if id.name == "Open" {
                            for p in paths {
                                let contents = std::fs::read_to_string(p).unwrap_or("".to_string());
                                self.project =
                                    serde_json::from_str(&contents).unwrap_or(Project::new());
                                //self.sidebar.load_from_project(ui, ctx, &self.project);
                                //self.tileeditor.load_from_project(ui, ctx, &self.project);
                                self.editor.set_codegrid(self.project.codegrid.clone(), ui);
                                redraw = true;
                            }
                        } else if id.name == "Save" {
                            self.project.codegrid = self.editor.get_codegrid(ui);

                            for p in paths {
                                let json = serde_json::to_string(&self.project); //.unwrap();
                                                                                 //println!("{:?}", json.err());
                                if let Ok(json) = json {
                                    println!("{:?}", p);
                                    std::fs::write(p, json).expect("Unable to write file");
                                }
                            }
                        }
                    }
                    TheEvent::StateChanged(id, _state) => {
                        //println!("app Widget State changed {:?}: {:?}", id, state);

                        if id.name == "Open" {
                            ctx.ui.open_file_requester(
                                TheId::named_with_id(id.name.as_str(), Uuid::new_v4()),
                                "Open".into(),
                                TheFileExtension::new(
                                    "CodeGrid".into(),
                                    vec!["codegrid".to_string()],
                                ),
                            );
                            ctx.ui
                                .set_widget_state("Open".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            redraw = true;
                        } else if id.name == "Save" {
                            ctx.ui.save_file_requester(
                                TheId::named_with_id(id.name.as_str(), Uuid::new_v4()),
                                "Save".into(),
                                TheFileExtension::new(
                                    "CodeGrid".into(),
                                    vec!["codegrid".to_string()],
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

                                    let mut compiler = TheCompiler::new();
                                    let rc = compiler.compile(grid);

                                    if let Ok(mut module) = rc {
                                        let mut sandbox = TheCodeSandbox::new();
                                        sandbox.debug_mode = true;

                                        sandbox.add_global("test", TheCodeNode::new(|_, data, _| {
                                            println!("inside test");
                                            if let Some(i) = data.values[0].to_i32() {
                                                println!("i: {:?}", i);
                                                data.values[0] = TheValue::Int(i + 1);
                                            }
                                        }, TheCodeNodeData::values(vec![TheValue::Int(0)])));

                                        sandbox.insert_module(module.clone());
                                        module.execute(&mut sandbox);
                                        code_view.set_debug_module(
                                            sandbox.get_module_debug_module(module.uuid),
                                        );
                                    } else {
                                        code_view.set_debug_module(TheDebugModule::new());
                                    }

                                    self.editor.set_grid_status_message(ui, ctx);
                                    //println!("Size of MyEnum: {} bytes", std::mem::size_of::<TheCodeAtom>());
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
                                    self.editor.set_grid_status_message(ui, ctx);
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
