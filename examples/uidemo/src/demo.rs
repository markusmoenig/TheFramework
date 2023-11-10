use crate::{browser::Browser, prelude::*};
use theframework::prelude::*;

pub struct UIDemo {
    sidebar: Sidebar,
    browser: Browser,
}

impl TheTrait for UIDemo {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            sidebar: Sidebar::new(),
            browser: Browser::new(),
        }
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        // Left
        let mut left_canvas = TheCanvas::new();

        let mut blue_color = TheColorButton::new(TheId::named("Blue"));
        blue_color.set_color([0, 0, 255, 255]);
        blue_color.limiter_mut().set_max_width(60);
        left_canvas.set_widget(blue_color);

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

        /*
        let mut bottom_canvas = TheCanvas::new();

        let mut yellow_color = TheColorButton::new("Yellow".to_string());
        yellow_color.set_color([255, 255, 0, 255]);
        yellow_color.limiter_mut().set_max_height(200);
        bottom_canvas.set_widget(yellow_color);*/

        //

        ui.canvas.set_left(left_canvas);
        ui.canvas.set_top(top_canvas);
        ui.canvas
            .set_widget(TheColorButton::new(TheId::named("White")));
    }

    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        self.sidebar.update_ui(ui, ctx)
    }
}
