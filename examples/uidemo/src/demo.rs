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
        let mut left_canvas: Box<TheCanvas> = Box::new(TheCanvas::new());
        left_canvas.limiter.set_max_width(60);

        let mut blue_color = Box::new(TheColorButton::new("Blue".to_string()));
        blue_color.set_color([0, 0, 255, 255]);
        left_canvas.widget = Some(blue_color);

        // Top
        let mut top_canvas: Box<TheCanvas> = Box::new(TheCanvas::new());
        top_canvas.limiter.set_max_height(80);

        let mut green_color = Box::new(TheColorButton::new("Green".to_string()));
        green_color.set_color([0, 255, 0, 255]);
        top_canvas.widget = Some(green_color);

        // Right
        let mut sidebar = Sidebar::new();
        sidebar.init_ui(ui, ctx);

        // Bottom

        let mut bottom_canvas: Box<TheCanvas> = Box::new(TheCanvas::new());
        bottom_canvas.limiter.set_max_height(200);

        let mut yellow_color: Box<TheColorButton> =
            Box::new(TheColorButton::new("Yellow".to_string()));
        yellow_color.set_color([255, 255, 0, 255]);
        bottom_canvas.widget = Some(yellow_color);

        //

        ui.canvas.left = Some(left_canvas);
        ui.canvas.top = Some(top_canvas);
        ui.canvas.bottom = Some(bottom_canvas);
        ui.canvas.widget = Some(Box::new(TheColorButton::new("White".to_string())));
    }
}
