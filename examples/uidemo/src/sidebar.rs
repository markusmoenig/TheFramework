use theframework::theui::thewidget::switchbar::TheSectionHeaderTrait;

use crate::prelude::*;

pub struct Sidebar {}

impl Sidebar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        let mut vertical_canvas = TheCanvas::new();
        vertical_canvas.limiter.set_max_width(90);

        let mut white_color = TheColorButton::new("Box".to_string());
        white_color.set_color([255, 255, 255, 255]);

        let mut vlayout = TheVLayout::new("Context Buttons".to_string());
        vlayout.add_widget(Box::new(white_color));
        vertical_canvas.set_layout(vlayout);

        let mut canvas = TheCanvas::new();
        canvas.limiter.set_max_width(360);

        let mut red_color = TheColorButton::new("Red".to_string());
        red_color.set_color([255, 0, 0, 255]);
        canvas.set_widget(red_color);

        let mut header = TheCanvas::new();
        header.limiter.set_max_height(21);
        let mut section_header = TheSwitchbar::new("Section Header".to_string());
        section_header.set_text("Section Header".to_string());
        header.set_widget(section_header);

        canvas.set_top(header);
        canvas.set_right(vertical_canvas);
        canvas.top_is_expanding = false;

        ui.canvas.set_right(canvas);
    }
}
