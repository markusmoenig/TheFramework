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

        let mut yellow_color = TheDropdownMenu::new("DropDown".to_string());
        yellow_color.add_option("Option #1".to_string());
        yellow_color.add_option("Option #2".to_string());

        let mut hlayout = TheHLayout::new("Menu Layout".to_string());
        hlayout.set_background_color(None);
        hlayout.add_widget(Box::new(yellow_color));

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

    fn needs_update(&mut self, ctx: &mut TheContext) -> bool {
        self.sidebar.needs_update(ctx)
    }
}
