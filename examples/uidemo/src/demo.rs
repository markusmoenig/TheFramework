use crate::prelude::*;
use theframework::prelude::*;

pub struct UIDemo {
    sidebar: Sidebar,
}

impl TheTrait for UIDemo {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            sidebar: Sidebar::new(),
        }
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        // Left
        let mut left_canvas = TheCanvas::new();

        let mut blue_color = TheColorButton::new("Blue".to_string());
        blue_color.set_color([0, 0, 255, 255]);
        blue_color.limiter_mut().set_max_width(60);
        left_canvas.set_widget(blue_color);

        // Top
        let mut top_canvas = TheCanvas::new();

        let menubar = TheMenubar::new("Menubar".to_string());

        let mut open_button = TheMenubarButton::new("Open".to_string());
        open_button.set_icon_name("icon_role_load".to_string());

        let mut save_button = TheMenubarButton::new("Save".to_string());
        save_button.set_icon_name("icon_role_save".to_string());

        let mut save_as_button = TheMenubarButton::new("Save As".to_string());
        save_as_button.set_icon_name("icon_role_save_as".to_string());
        save_as_button.set_icon_offset(vec2i(2, -5));

        let mut undo_button = TheMenubarButton::new("Undo".to_string());
        undo_button.set_icon_name("icon_role_undo".to_string());

        let mut redo_button = TheMenubarButton::new("Redo".to_string());
        redo_button.set_icon_name("icon_role_redo".to_string());

        let mut hlayout = TheHLayout::new("Menu Layout".to_string());
        hlayout.set_background_color(None);
        hlayout.set_margin(vec4i(40, 5, 20, 0));
        hlayout.add_widget(Box::new(open_button));
        hlayout.add_widget(Box::new(save_button));
        hlayout.add_widget(Box::new(save_as_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new("".to_string())));
        hlayout.add_widget(Box::new(undo_button));
        hlayout.add_widget(Box::new(redo_button));

        top_canvas.set_widget(menubar);
        top_canvas.set_layout(hlayout);

        // Right
        self.sidebar.init_ui(ui, ctx);

        // Bottom

        let mut bottom_canvas = TheCanvas::new();

        let mut yellow_color = TheColorButton::new("Yellow".to_string());
        yellow_color.set_color([255, 255, 0, 255]);
        yellow_color.limiter_mut().set_max_height(200);
        bottom_canvas.set_widget(yellow_color);

        //

        ui.canvas.set_left(left_canvas);
        ui.canvas.set_top(top_canvas);
        ui.canvas.set_bottom(bottom_canvas);
        ui.canvas
            .set_widget(TheColorButton::new("White".to_string()));
    }

    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        self.sidebar.update_ui(ui, ctx)
    }
}
