use theframework::theui::thewidget::switchbar::TheSectionHeaderTrait;

use crate::prelude::*;
use std::sync::mpsc::Receiver;

pub struct Sidebar {
    state_receiver: Option<Receiver<TheEvent>>,

}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            state_receiver: None,
        }
    }

    pub fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        let mut vertical_canvas = TheCanvas::new();

        let mut cube_sectionbar_button = TheSectionbarButton::new("Cube".to_string());
        cube_sectionbar_button.set_text("Cube".to_string());

        let mut sphere_sectionbar_button = TheSectionbarButton::new("Sphere".to_string());
        sphere_sectionbar_button.set_text("Sphere".to_string());

        let mut vlayout = TheVLayout::new("Context Buttons".to_string());
        vlayout.add_widget(Box::new(cube_sectionbar_button));
        vlayout.add_widget(Box::new(sphere_sectionbar_button));
        vlayout.set_fixed_content_size(vec2i(81, 47));
        vlayout.set_margin(vec4i(5, 10, 5, 10));
        vlayout.set_padding(4);
        vlayout.set_background_color(SectionbarBackground);
        vlayout.limiter_mut().set_max_width(90);
        vertical_canvas.set_layout(vlayout);

        let mut canvas = TheCanvas::new();

        let mut red_color = TheColorButton::new("Red".to_string());
        red_color.set_color([255, 0, 0, 255]);
        red_color.limiter_mut().set_max_width(360);
        canvas.set_widget(red_color);

        let mut header = TheCanvas::new();
        let mut switchbar = TheSwitchbar::new("Section Header".to_string());
        switchbar.set_text("Section Header".to_string());
        header.set_widget(switchbar);

        canvas.set_top(header);
        canvas.set_right(vertical_canvas);
        canvas.top_is_expanding = false;

        ui.canvas.set_right(canvas);

        self.state_receiver = Some(ui.add_state_listener("Sidebar".into()));
    }

    #[allow(clippy::single_match)]
    pub fn needs_update(&mut self, ctx: &mut TheContext) -> bool {

        let mut redraw = false;

        if let Some(receiver) = &mut self.state_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    TheEvent::StateChanged(id, state) => {
                        println!("app Widget State changed {:?}: {:?}", id, state);

                        if id.name == "Cube" {
                            ctx.ui.set_widget_state("Sphere".to_string(), TheWidgetState::None);
                        } else if id.name == "Sphere" {
                            ctx.ui.set_widget_state("Cube".to_string(), TheWidgetState::None);
                        }

                        redraw = true;
                    },
                    _ => {}
                }
            }
        }
        redraw
    }
}
