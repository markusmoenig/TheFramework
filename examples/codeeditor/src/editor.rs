use crate::{browser::Browser, prelude::*};
use std::sync::mpsc::Receiver;
use theframework::prelude::*;

pub struct CodeEditor {
    sidebar: Sidebar,
    browser: Browser,

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
            browser: Browser::new(),

            editor: TheCodeEditor::default(),

            event_receiver: None,
        }
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

        self.browser.init_ui(ui, ctx);

        //

        ui.canvas.set_top(top_canvas);

        ui.canvas.set_center(self.editor.build_canvas(ctx));

        self.event_receiver = Some(ui.add_state_listener("Main Receiver".into()));
    }

    #[allow(clippy::single_match)]
    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                redraw = self.editor.handle_event(&event, ui, ctx);
                match event {
                    TheEvent::ValueChanged(id, value) => {
                        if id.name.starts_with("Atom") {
                            // Set an edited value to the code grid
                            if let Some(layout) = ui.get_code_layout("Code Editor") {
                                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                                    if let Some(selection) = self.sidebar.editor_selection {
                                        code_view.set_atom_value(selection, id.name, value);
                                    }
                                }
                            }
                        }
                    }
                    TheEvent::StateChanged(id, _state) => {
                        //println!("app Widget State changed {:?}: {:?}", id, state);

                        if id.name == "Compile" {
                            if let Some(layout) = ui.get_code_layout("Code Editor") {
                                if let Some(code_view) = layout.code_view_mut().as_code_view() {
                                    let grid = code_view.code_grid();

                                    /*
                                    code_ctx
                                        .code
                                        .insert((0, 0), TheAtom::Value(TheValue::Int(2)));
                                    code_ctx.code.insert((1, 0), TheAtom::Add());
                                    code_ctx
                                        .code
                                        .insert((2, 0), TheAtom::Value(TheValue::Int(5)));
                                    code_ctx.code.insert((3, 0), TheAtom::Multiply());
                                    code_ctx
                                        .code
                                        .insert((4, 0), TheAtom::Value(TheValue::Int(5)));
                                    */

                                    let mut compiler = TheCompiler::new();
                                    let rc = compiler.compile(grid);

                                    if let Ok(mut pipe) = rc {
                                        pipe.execute();
                                    }

                                    //println!("Size of MyEnum: {} bytes", std::mem::size_of::<TheAtom>());
                                }
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
