use crate::prelude::*;
use std::sync::mpsc::Receiver;

pub struct Sidebar {
    state_receiver: Option<Receiver<TheEvent>>,
    stack_layout_id: TheId,
}

#[allow(clippy::new_without_default)]
impl Sidebar {
    pub fn new() -> Self {
        Self {
            state_receiver: None,
            stack_layout_id: TheId::empty(),
        }
    }

    pub fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        let mut sectionbar_canvas = TheCanvas::new();

        let mut section_bar_canvas = TheCanvas::new();
        section_bar_canvas.set_widget(TheSectionbar::new(TheId::named("Sectionbar")));
        sectionbar_canvas.set_top(section_bar_canvas);

        let mut cube_sectionbar_button = TheSectionbarButton::new(TheId::named("Layout #1"));
        cube_sectionbar_button.set_text("Layout 1".to_string());

        let mut sphere_sectionbar_button = TheSectionbarButton::new(TheId::named("Layout #2"));
        sphere_sectionbar_button.set_text("Layout 2".to_string());

        let mut vlayout = TheVLayout::new(TheId::named("Context Buttons"));
        vlayout.add_widget(Box::new(cube_sectionbar_button));
        vlayout.add_widget(Box::new(sphere_sectionbar_button));
        vlayout.set_margin(vec4i(5, 10, 5, 10));
        vlayout.set_padding(4);
        vlayout.set_background_color(Some(SectionbarBackground));
        vlayout.limiter_mut().set_max_width(90);
        sectionbar_canvas.set_layout(vlayout);

        // Switchbar
        let mut header = TheCanvas::new();
        let mut switchbar = TheSwitchbar::new(TheId::named("Switchbar Header"));
        switchbar.set_text("Section Header".to_string());
        header.set_widget(switchbar);

        // ListLayout
        let mut list_canvas = TheCanvas::new();
        let mut list_layout = TheListLayout::new(TheId::named("List Layout"));
        list_layout.limiter_mut().set_max_size(vec2i(360, 200));
        list_canvas.set_top(header);

        for i in 0..25 {
            let mut list_item: TheListItem =
                TheListItem::new(TheId::named(format!("List Item {}", i).as_str()));
            list_item.set_text(format!("Item #{}", i));
            list_layout.add_item(list_item, ctx);
        }

        let mut toolbar_canvas = TheCanvas::new();
        let toolbar_widget = TheToolbar::new(TheId::named("Toolbar"));

        let mut add_button = TheToolbarButton::new(TheId::named("Add"));
        add_button.set_icon_name("icon_role_add".to_string());

        let mut remove_button = TheToolbarButton::new(TheId::named("Remove"));
        remove_button.set_icon_name("icon_role_remove".to_string());

        let mut toolbar_hlayout = TheHLayout::new(TheId::named("Toolbar Layout"));
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 0));
        toolbar_hlayout.add_widget(Box::new(add_button));
        toolbar_hlayout.add_widget(Box::new(remove_button));

        toolbar_canvas.set_layout(toolbar_hlayout);
        toolbar_canvas.set_widget(toolbar_widget);

        list_canvas.set_bottom(toolbar_canvas);
        list_canvas.set_layout(list_layout);

        // Snapperbar

        let mut snapperbar = TheSnapperbar::new(TheId::named("Snapperbar Header"));
        snapperbar.set_text("Snapperbar".to_string());

        // ---

        let mut text_layout = TheTextLayout::new(TheId::named("Text Layout"));
        //text_layout.set_text_margin(50);

        let text_line_edit = TheTextLineEdit::new(TheId::named("Text Line Edit"));
        text_layout.add_pair("Text Line Edit".to_string(), Box::new(text_line_edit));

        let slider = TheSlider::new(TheId::named("Slider"));
        text_layout.add_pair("Slider".to_string(), Box::new(slider));

        for i in 0..10 {
            let mut dropdown =
                TheDropdownMenu::new(TheId::named(format!("DropDown {}", i).as_str()));
            dropdown.add_option("Option #1".to_string());
            dropdown.add_option("Option #2".to_string());
            text_layout.add_pair(format!("Item #{}", i), Box::new(dropdown));
        }

        let mut snapper_canvas = TheCanvas::new();
        let mut snapper_layout = TheSnapperLayout::new(TheId::named("Snapper Layout"));
        snapper_layout.add_pair(Box::new(snapperbar), Box::new(text_layout));
        snapper_layout.limiter_mut().set_max_width(360);
        snapper_canvas.set_layout(snapper_layout);

        let mut canvas = TheCanvas::new();
        let mut stack_layout = TheStackLayout::new(TheId::named("Stack Layout"));
        stack_layout.add_canvas(snapper_canvas);

        let mut test_canvas = TheCanvas::new();
        let mut test_layout = TheVLayout::new(TheId::named("Dummy"));
        test_layout.limiter_mut().set_max_width(360);
        let mut dummy_text = TheText::new(TheId::empty());
        dummy_text.set_text("Test".to_string());
        test_layout.add_widget(Box::new(dummy_text));
        test_canvas.set_layout(test_layout);
        stack_layout.add_canvas(test_canvas);

        self.stack_layout_id = stack_layout.id().clone();
        stack_layout.set_index(0);
        canvas.set_layout(stack_layout);

        canvas.set_top(list_canvas);
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

                        if id.name == "Open" {
                            #[cfg(not(target_arch = "wasm32"))]
                            ctx.ui.open_file_requester(
                                TheId::named("MyID"),
                                "Open".into(),
                                TheFileExtension::new("PNG".into(), vec!["png".to_string()]),
                            );
                            ctx.ui
                                .set_widget_state("Open".to_string(), TheWidgetState::None);
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
                    }
                    _ => {}
                }
            }
        }
        redraw
    }
}
