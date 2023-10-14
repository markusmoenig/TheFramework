use crate::prelude::*;
use theframework::prelude::*;

pub struct UIDemo {}

impl TheTrait for UIDemo {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        // Left
        let mut left_canvas = TheCanvas::new();
        left_canvas.limiter.set_max_width(60);

        let mut blue_color = TheColorButton::new("Blue".to_string());
        blue_color.set_color([0, 0, 255, 255]);
        left_canvas.set_widget(blue_color);

        // Top
        let mut top_canvas = TheCanvas::new();
        top_canvas.limiter.set_max_height(80);

        let mut green_color = TheColorButton::new("Green".to_string());
        green_color.set_color([0, 255, 0, 255]);
        top_canvas.set_widget(green_color);

        // Right
        let mut sidebar = Sidebar::new();
        sidebar.init_ui(ui, ctx);

        // Bottom

        let mut bottom_canvas = TheCanvas::new();
        bottom_canvas.limiter.set_max_height(200);

        let mut yellow_color = TheColorButton::new("Yellow".to_string());
        yellow_color.set_color([255, 255, 0, 255]);
        bottom_canvas.set_widget(yellow_color);

        //

        ui.canvas.set_left(left_canvas);
        ui.canvas.set_top(top_canvas);
        ui.canvas.set_bottom(bottom_canvas);
        ui.canvas
            .set_widget(TheColorButton::new("White".to_string()));
    }
}
