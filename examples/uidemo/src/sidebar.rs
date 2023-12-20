use crate::prelude::*;
use std::sync::mpsc;

pub struct Sidebar {
    stack_layout_id: TheId,

    pub renderer_command: Option<mpsc::Sender<RendererMessage>>,
}

#[allow(clippy::new_without_default)]
impl Sidebar {
    pub fn new() -> Self {
        Self {
            stack_layout_id: TheId::empty(),

            renderer_command: None,
        }
    }

    pub fn init_ui(&mut self, ui: &mut TheUI, _ctx: &mut TheContext, project: &mut Project) {
        let width = 400;

        // Section Buttons

        let mut sectionbar_canvas = TheCanvas::new();

        let mut section_bar_canvas = TheCanvas::new();
        section_bar_canvas.set_widget(TheSectionbar::new(TheId::named("Sectionbar")));
        sectionbar_canvas.set_top(section_bar_canvas);

        let mut sphere_sectionbar_button = TheSectionbarButton::new(TheId::named("Sphere Section"));
        sphere_sectionbar_button.set_text("Sphere".to_string());
        sphere_sectionbar_button.set_state(TheWidgetState::Selected);

        let mut cube_sectionbar_button = TheSectionbarButton::new(TheId::named("Cube Section"));
        cube_sectionbar_button.set_text("Cube".to_string());

        let mut pyramid_sectionbar_button =
            TheSectionbarButton::new(TheId::named("Pyramid Section"));
        pyramid_sectionbar_button.set_text("Pyramid".to_string());

        let mut vlayout = TheVLayout::new(TheId::named("Section Buttons"));
        vlayout.add_widget(Box::new(sphere_sectionbar_button));
        vlayout.add_widget(Box::new(cube_sectionbar_button));
        vlayout.add_widget(Box::new(pyramid_sectionbar_button));
        vlayout.set_margin(vec4i(5, 10, 5, 10));
        vlayout.set_padding(4);
        vlayout.set_background_color(Some(SectionbarBackground));
        vlayout.limiter_mut().set_max_width(90);
        sectionbar_canvas.set_layout(vlayout);

        // Header

        let mut header: TheCanvas = TheCanvas::new();
        let mut switchbar = TheSwitchbar::new(TheId::empty());
        header.limiter_mut().set_max_width(310);
        switchbar.set_text("Material".to_string());
        header.set_widget(switchbar);

        // Stack Layout

        let mut stack_layout = TheStackLayout::new(TheId::named("Stack Layout"));
        stack_layout.limiter_mut().set_max_width(width);

        self.stack_layout_id = stack_layout.id().clone();

        // Material Canvas

        let mut material_canvas = TheCanvas::default();

        let mut text_layout = TheTextLayout::new(TheId::named("Material Layout"));
        text_layout.limiter_mut().set_max_width(width);

        let mut metallic = TheSlider::new(TheId::named("Metallic"));
        metallic.set_status_text("The metallic attribute of the material.");
        metallic.set_value(TheValue::Float(project.material.metallic));
        text_layout.add_pair("Metallic".to_string(), Box::new(metallic));

        let mut roughness = TheSlider::new(TheId::named("Roughness"));
        roughness.set_status_text("The roughness attribute of the material.");
        roughness.set_value(TheValue::Float(project.material.roughness));
        text_layout.add_pair("Roughness".to_string(), Box::new(roughness));

        let mut transmission = TheSlider::new(TheId::named("Transmission"));
        transmission.set_status_text("The transmission attribute of the material.");
        transmission.set_value(TheValue::Float(project.material.spec_trans));
        text_layout.add_pair("Transmission".to_string(), Box::new(transmission));

        material_canvas.set_layout(text_layout);
        material_canvas.top_is_expanding = false;

        stack_layout.add_canvas(material_canvas);

        // Put it all into the canvas

        let mut canvas = TheCanvas::new();
        canvas.set_top(header);
        stack_layout.set_index(0);
        canvas.top_is_expanding = false;
        canvas.set_layout(stack_layout);
        canvas.set_right(sectionbar_canvas);

        ui.canvas.set_right(canvas);
    }

    #[allow(clippy::single_match)]
    pub fn handle_event(
        &mut self,
        event: &TheEvent,
        _ui: &mut TheUI,
        ctx: &mut TheContext,
        project: &mut Project,
    ) -> bool {
        let mut redraw = false;

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
            TheEvent::ValueChanged(id, value) => {
                if id.name == "Metallic" {
                    if let TheValue::Float(metallic) = value {
                        project.material.metallic = *metallic;

                        if let Some(renderer_command) = &self.renderer_command {
                            renderer_command
                                .send(RendererMessage::Material(project.material.clone()))
                                .unwrap();
                        }
                    }
                } else if id.name == "Roughness" {
                    if let TheValue::Float(roughness) = value {
                        project.material.roughness = *roughness;

                        if let Some(renderer_command) = &self.renderer_command {
                            renderer_command
                                .send(RendererMessage::Material(project.material.clone()))
                                .unwrap();
                        }
                    }
                }
                if id.name == "Transmission" {
                    if let TheValue::Float(transmission) = value {
                        project.material.spec_trans = *transmission;

                        if let Some(renderer_command) = &self.renderer_command {
                            renderer_command
                                .send(RendererMessage::Material(project.material.clone()))
                                .unwrap();
                        }
                    }
                }
            }
            TheEvent::FileRequesterResult(id, paths) => {
                println!("FileRequester Result {:?} {:?}", id, paths);
            }
            _ => {}
        }
        redraw
    }
}
