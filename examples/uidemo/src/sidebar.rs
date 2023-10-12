use crate::prelude::*;

pub struct Sidebar {
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        let mut canvas: Box<TheCanvas> = Box::new(TheCanvas::new());
        canvas.limiter.set_max_width(300);

        let mut red_color = Box::new(TheColorButton::new("Red".to_string()));
        red_color.set_color([255, 0, 0, 255]);
        canvas.widget = Some(red_color);


        let mut header: Box<TheCanvas> = Box::new(TheCanvas::new());
        header.limiter.set_max_height(21);
        let mut black_color = Box::new(TheColorButton::new("Black".to_string()));
        black_color.set_color([0, 255, 255, 255]);
        header.widget = Some(black_color);

        canvas.top = Some(header);

        ui.canvas.right = Some(canvas);
    }
}
