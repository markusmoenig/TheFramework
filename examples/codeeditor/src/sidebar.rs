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

        let width = 320;

        let mut text_layout: TheTextLayout = TheTextLayout::new(TheId::named("Values Layout"));
        text_layout
            .limiter_mut()
            .set_max_width(width);

        // let name_edit = TheTextLineEdit::new(TheId::named("Region Name Edit"));
        // text_layout.add_pair("Name".to_string(), Box::new(name_edit));

        // List

        let mut list_header = TheCanvas::new();
        let mut switchbar = TheSwitchbar::new(TheId::empty());
        switchbar.set_text("Available Codes List".to_string());
        list_header.set_widget(switchbar);

        let mut list_canvas = TheCanvas::new();

        let mut code_layout = ui.create_code_list(ctx);
        code_layout
            .limiter_mut()
            .set_max_size(vec2i(width, 400));
        list_canvas.set_layout(code_layout);
        list_canvas.set_top(list_header);

        let mut apply_button = TheTraybarButton::new(TheId::named("Apply"));
        apply_button.set_text("Apply Code".to_string());

        let mut toolbar_hlayout = TheHLayout::new(TheId::empty());
        toolbar_hlayout.set_background_color(None);
        toolbar_hlayout.set_margin(vec4i(5, 2, 5, 2));
        toolbar_hlayout.add_widget(Box::new(apply_button));

        let mut toolbar_canvas = TheCanvas::default();
        toolbar_canvas.set_widget(TheTraybar::new(TheId::empty()));
        toolbar_canvas.set_layout(toolbar_hlayout);
        list_canvas.set_bottom(toolbar_canvas);

        let mut text_layout: TheTextLayout = TheTextLayout::new(TheId::empty());
        text_layout.limiter_mut().set_max_width(width);
        let name_edit = TheTextLineEdit::new(TheId::named("Region Name Edit"));
        text_layout.add_pair("Name".to_string(), Box::new(name_edit));

        //

        let mut settings = TheCanvas::new();

        let mut settings_header = TheCanvas::new();
        let mut switchbar = TheSwitchbar::new(TheId::empty());
        switchbar.set_text("Code Settings".to_string());
        settings_header.set_widget(switchbar);

        settings.set_top(settings_header);
        settings.set_layout(text_layout);

        let mut canvas: TheCanvas = TheCanvas::new();

        settings.limiter_mut().set_max_width(width);
        canvas.set_center(settings);
        canvas.set_top(list_canvas);

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
