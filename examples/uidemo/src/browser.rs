use crate::prelude::*;
use std::sync::mpsc::Receiver;

pub struct Browser {
    state_receiver: Option<Receiver<TheEvent>>,
}

#[allow(clippy::new_without_default)]
impl Browser {
    pub fn new() -> Self {
        Self {
            state_receiver: None,
        }
    }

    pub fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {

        let mut canvas = TheCanvas::new();

        let mut tab_bar = TheTabbar::new("Tabbar".to_string());

        tab_bar.add_tab("Red".to_string());
        tab_bar.add_tab("Yellow".to_string());

        canvas.set_widget(tab_bar);
        /*
        let mut yellow_color = TheColorButton::new("Yellow".to_string());
        yellow_color.set_color([255, 255, 0, 255]);

        yellow_color.limiter_mut().set_max_height(200);
        */

        ui.canvas.set_bottom(canvas);

        /*
        let mut sectionbar_canvas = TheCanvas::new();

        let mut section_bar_canvas = TheCanvas::new();
        section_bar_canvas.set_widget(TheSectionbar::new("Sectionbar".to_string()));
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

        // ListLayout
        let mut list_canvas = TheCanvas::new();
        let mut list_layout = TheListLayout::new("List Layout".to_string());
        list_layout.limiter_mut().set_max_size(vec2i(360, 200));
        list_canvas.set_top(header);

        for i in 0..25 {
            let mut list_item: TheListItem = TheListItem::new(format!("List Item {}", i));
            list_item.set_text(format!("Item #{}", i));
            list_layout.add_item(list_item, ctx);
        }

        let mut toolbar_canvas = TheCanvas::new();
        let toolbar_widget = TheToolbar::new("Toolbar".to_string());

        let mut add_button = TheToolbarButton::new("Add".to_string());
        add_button.set_icon_name("icon_role_add".to_string());

        let mut remove_button = TheToolbarButton::new("Remove".to_string());
        remove_button.set_icon_name("icon_role_remove".to_string());

        let mut toolbar_hlayout = TheHLayout::new("Toolbar Layout".to_string());
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 0));
        toolbar_hlayout.add_widget(Box::new(add_button));
        toolbar_hlayout.add_widget(Box::new(remove_button));

        toolbar_canvas.set_layout(toolbar_hlayout);
        toolbar_canvas.set_widget(toolbar_widget);

        list_canvas.set_bottom(toolbar_canvas);
        list_canvas.set_layout(list_layout);

        // Snapperbar

        let mut snapperbar = TheSnapperbar::new("Snapperbar Header".to_string());
        snapperbar.set_text("Snapperbar".to_string());

        // ---

        let mut text_layout = TheTextLayout::new("Text Layout".to_string());
        //text_layout.set_text_margin(50);

        let text_line_edit = TheTextLineEdit::new("Text Line Edit".to_string());
        text_layout.add_pair("Text Line Edit".to_string(), Box::new(text_line_edit));

        let slider = TheSlider::new("Slider".to_string());
        text_layout.add_pair("Slider".to_string(), Box::new(slider));

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

        canvas.set_top(list_canvas);
        canvas.set_right(sectionbar_canvas);
        canvas.top_is_expanding = false;

        ui.canvas.set_right(canvas);

        self.state_receiver = Some(ui.add_state_listener("Sidebar".into()));
        */
    }

    #[allow(clippy::single_match)]
    pub fn update_ui(&mut self, _ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        if let Some(receiver) = &mut self.state_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    /*
                    TheEvent::StateChanged(id, _state) => {
                        //println!("app Widget State changed {:?}: {:?}", id, state);

                        if id.name == "Open" {
                            ctx.ui.open_file_requester(TheId::new("MyID".into()), "Open".into(), vec![] );
                            ctx.ui.set_widget_state("Open".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                        } else if id.name == "Cube" {
                            ctx.ui
                                .set_widget_state("Sphere".to_string(), TheWidgetState::None);
                            ctx.ui
                                .send(TheEvent::SetStackIndex(self.stack_layout_id.clone(), 0));
                        } else if id.name == "Sphere" {
                            ctx.ui
                                .set_widget_state("Cube".to_string(), TheWidgetState::None);
                            ctx.ui
                                .send(TheEvent::SetStackIndex(self.stack_layout_id.clone(), 1));
                        }

                        redraw = true;
                    }
                    TheEvent::FileRequesterResult(id, paths) => {
                        println!("FileRequester Result {:?} {:?}", id, paths);
                    }*/
                    _ => {}
                }
            }
        }
        redraw
    }
}