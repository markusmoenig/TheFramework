use crate::prelude::*;
use std::sync::mpsc::Receiver;

pub struct Sidebar {
    state_receiver: Option<Receiver<TheEvent>>,
    stack_layout_id: TheWidgetId,
}

#[allow(clippy::new_without_default)]
impl Sidebar {
    pub fn new() -> Self {
        Self {
            state_receiver: None,
            stack_layout_id: TheWidgetId::new("".to_string()),
        }
    }

    pub fn init_ui(&mut self, ui: &mut TheUI, _ctx: &mut TheContext) {

        let mut sectionbar_canvas = TheCanvas::new();

        let mut section_bar_canvas = TheCanvas::new();
        section_bar_canvas
            .set_widget(TheSectionbar::new("Sectionbar".to_string()));
        sectionbar_canvas.set_top(section_bar_canvas);

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

        // Snapperbar

        let mut snapperbar = TheSnapperbar::new("Snapperbar Header".to_string());
        snapperbar.set_text("Snapperbar".to_string());

        // ---

        let mut text_layout = TheTextLayout::new("Text Layout".to_string());
        //text_layout.set_text_margin(50);

        let text_line_edit = TheTextLineEdit::new("Text Line Edit".to_string());
        text_layout.add_pair("Text Line Edit".to_string(), Box::new(text_line_edit));

        for i in 0..10 {
            let mut dropdown = TheDropdownMenu::new(format!("DropDown {}", i));
            dropdown.add_option("Option #1".to_string());
            dropdown.add_option("Option #2".to_string());
            text_layout.add_pair(format!("Item #{}", i), Box::new(dropdown));
        }

        let mut snapper_layout = TheSnapperLayout::new("Snapper Layout".to_string());
        snapper_layout.add_pair(Box::new(snapperbar), Box::new(text_layout));
        snapper_layout.limiter_mut().set_max_width(360);

        let mut canvas = TheCanvas::new();
        let mut stack_layout = TheStackLayout::new("Stack Layout".to_string());
        stack_layout.add_layout(Box::new(snapper_layout));

        let mut test_layout = TheVLayout::new("Dummy".to_string());
        test_layout.limiter_mut().set_max_width(360);
        let mut dummy_text = TheText::new("dummy text".to_string());
        dummy_text.set_text("Test".to_string());

        test_layout.add_widget(Box::new(dummy_text));
        stack_layout.add_layout(Box::new(test_layout));

        self.stack_layout_id = stack_layout.id().clone();
        stack_layout.set_index(0);
        canvas.set_layout(stack_layout);

        canvas.set_top(header);
        canvas.set_right(sectionbar_canvas);
        canvas.top_is_expanding = false;

        ui.canvas.set_right(canvas);

        self.state_receiver = Some(ui.add_state_listener("Sidebar".into()));
    }

    #[allow(clippy::single_match)]
    pub fn update_ui(&mut self, _ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        if let Some(receiver) = &mut self.state_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    TheEvent::StateChanged(id, _state) => {
                        //println!("app Widget State changed {:?}: {:?}", id, state);

                        if id.name == "Cube" {
                            ctx.ui
                                .set_widget_state("Sphere".to_string(), TheWidgetState::None);
                            ctx.ui.send(TheEvent::SetStackIndex(self.stack_layout_id.clone(), 0));
                        } else if id.name == "Sphere" {
                            ctx.ui
                                .set_widget_state("Cube".to_string(), TheWidgetState::None);
                            ctx.ui.send(TheEvent::SetStackIndex(self.stack_layout_id.clone(), 1));
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
