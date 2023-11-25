use crate::{browser::Browser, prelude::*};
use theframework::prelude::*;

pub struct CodeEditor {
    sidebar: Sidebar,
    browser: Browser,
}

impl TheTrait for CodeEditor {
    fn new() -> Self
    where
        Self: Sized,
    {
        let mut code_ctx = TheCodeContext::new();

        code_ctx
            .code
            .insert((0, 0), TheAtom::Value(TheValue::Int(2)));
        code_ctx.code.insert((2, 0), TheAtom::Add());
        code_ctx
            .code
            .insert((1, 0), TheAtom::Value(TheValue::Int(5)));

        let mut compiler = TheCompiler::new();
        let rc = compiler.compile(code_ctx);

        if let Ok(mut pipe) = rc {
            pipe.execute();
        }

        Self {
            sidebar: Sidebar::new(),
            browser: Browser::new(),
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
        self.sidebar.init_ui(ui, ctx);

        // Bottom

        self.browser.init_ui(ui, ctx);

        //

        ui.canvas.set_top(top_canvas);
        ui.canvas
            .set_widget(TheColorButton::new(TheId::named("White")));
    }

    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        //self.sidebar.update_ui(ui, ctx)
        true
    }
}
