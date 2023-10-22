use crate::prelude::*;
use std::sync::mpsc::Receiver;

pub struct Sidebar {
    state_receiver: Option<Receiver<TheEvent>>,
}

#[allow(clippy::new_without_default)]
impl Sidebar {
    pub fn new() -> Self {
        Self {
            state_receiver: None,
        }
    }

    pub fn init_ui(&mut self, ui: &mut TheUI, _ctx: &mut TheContext) {
        let mut sectionbar_canvas = TheCanvas::new();

        let mut section_bar_header_canvas = TheCanvas::new();
        section_bar_header_canvas
            .set_widget(TheSectionbarHeader::new("Context Header".to_string()));
        sectionbar_canvas.set_top(section_bar_header_canvas);

        let mut cube_sectionbar_button = TheSectionbarButton::new("Cube".to_string());
        cube_sectionbar_button.set_text("Cube".to_string());

        let mut sphere_sectionbar_button = TheSectionbarButton::new("Sphere".to_string());
        sphere_sectionbar_button.set_text("Sphere".to_string());

        let mut vlayout = TheVLayout::new("Context Buttons".to_string());
        vlayout.add_widget(Box::new(cube_sectionbar_button));
        vlayout.add_widget(Box::new(sphere_sectionbar_button));
        vlayout.set_margin(vec4i(5, 10, 5, 10));
        vlayout.set_padding(4);
        vlayout.set_background_color(Some(SectionbarBackground));
        vlayout.limiter_mut().set_max_width(90);
        sectionbar_canvas.set_layout(vlayout);

        // Switchbar

        let mut header = TheCanvas::new();
        let mut switchbar = TheSwitchbar::new("Switchbar Header".to_string());
        switchbar.set_text("Section Header".to_string());
        header.set_widget(switchbar);


        let mut text_layout = TheTextLayout::new("Text Layout".to_string());
        text_layout.limiter_mut().set_max_width(360);
        //text_layout.set_text_margin(50);

        let mut text_line_edit = TheTextLineEdit::new("Text Line Edit".to_string());
        text_layout.add_pair("Text Line Edit".to_string(), Box::new(text_line_edit));

        for i in 0..10 {
            let mut dropdown = TheDropdownMenu::new(format!("DropDown {}", i));
            dropdown.add_option("Option #1".to_string());
            dropdown.add_option("Option #2".to_string());
            text_layout.add_pair(format!("Item #{}", i), Box::new(dropdown));
        }

        let mut canvas = TheCanvas::new();
        canvas.set_layout(text_layout);

        canvas.set_top(header);
        canvas.set_right(sectionbar_canvas);
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
                            ctx.ui
                                .set_widget_state("Sphere".to_string(), TheWidgetState::None);
                        } else if id.name == "Sphere" {
                            ctx.ui
                                .set_widget_state("Cube".to_string(), TheWidgetState::None);
                        }

                        redraw = true;
                    }
                    _ => {}
                }
            }
        }
        redraw
    }
}
