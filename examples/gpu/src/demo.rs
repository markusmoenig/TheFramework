use std::sync::mpsc::Receiver;

use theframework::prelude::*;

pub struct Demo {
    canvas_layer: usize,
    translate_x: f32,
    translate_y: f32,

    event_receiver: Option<Receiver<TheEvent>>,
}

impl TheTrait for Demo {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            canvas_layer: 0,
            translate_x: 0.0,
            translate_y: 0.0,

            event_receiver: None,
        }
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        let sidebar_width: i32 = 400;

        self.canvas_layer = ctx.texture_renderer.add_layer();
        // Set zindex < 0 so that the ui layer can always be on top
        ctx.texture_renderer.set_layer_zindex(self.canvas_layer, -1);

        // Request screen capture. We can get the buffer later in `post_captured` callback.
        // ctx.gpu.request_capture(true);

        let textures = (0..7)
            .map(|i| {
                create_buffer(
                    [
                        (i as f32 / 6.0 * 255.0) as u8,
                        (i as f32 / 6.0 * 255.0) as u8,
                        (i as f32 / 6.0 * 255.0) as u8,
                        255,
                    ],
                    100,
                    100,
                )
            })
            .map(|buffer| {
                ctx.texture_renderer.load_texture(
                    ctx.gpu.device(),
                    ctx.gpu.queue(),
                    100,
                    100,
                    &buffer,
                )
            })
            .collect::<Vec<TheTextureId>>();
        for i in 0..1600 {
            ctx.texture_renderer.place_texture(
                self.canvas_layer,
                textures[i % textures.len()],
                Vec2::new(100.0 * (i % 40) as f32, 100.0 * (i / 40) as f32),
            );
        }

        let mut layout = TheTextLayout::new(TheId::named("Sidebar Layout"));
        layout.limiter_mut().set_max_width(sidebar_width);

        let mut scale = TheSlider::new(TheId::named("Scale"));
        scale.set_value(TheValue::Float(1.0));
        scale.set_range(TheValue::RangeF32(0.0..=10.0));
        layout.add_pair("Scale".to_string(), Box::new(scale));

        let mut translate_x = TheSlider::new(TheId::named("TranslateX"));
        translate_x.set_value(TheValue::Float(self.translate_x));
        translate_x.set_range(TheValue::RangeF32(0.0..=1000.0));
        layout.add_pair("Translate X".to_string(), Box::new(translate_x));

        let mut translate_y = TheSlider::new(TheId::named("TranslateY"));
        translate_y.set_value(TheValue::Float(self.translate_y));
        translate_y.set_range(TheValue::RangeF32(0.0..=1000.0));
        layout.add_pair("Translate Y".to_string(), Box::new(translate_y));

        layout.set_background_color(Some(SectionbarBackground));

        let mut sidebar = TheCanvas::new();
        sidebar.set_layout(layout);

        ui.canvas.set_right(sidebar);

        self.event_receiver = Some(ui.add_state_listener("Main".into()));
    }

    fn post_captured(&mut self, texture: Vec<u8>, width: u32, height: u32) {
        // Handle screen buffer here.
    }

    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    TheEvent::ValueChanged(id, value) => {
                        if id.name == "Scale" {
                            if let TheValue::Float(scale) = value {
                                if let Some(layer) =
                                    ctx.texture_renderer.layer_mut(self.canvas_layer)
                                {
                                    layer.scale(scale);
                                    redraw = true;
                                }
                            }
                        } else if id.name == "TranslateX" {
                            if let TheValue::Float(translate) = value {
                                if let Some(layer) =
                                    ctx.texture_renderer.layer_mut(self.canvas_layer)
                                {
                                    self.translate_x = translate;
                                    layer.translate(self.translate_x, self.translate_y);
                                    redraw = true;
                                }
                            }
                        } else if id.name == "TranslateY" {
                            if let TheValue::Float(translate) = value {
                                if let Some(layer) =
                                    ctx.texture_renderer.layer_mut(self.canvas_layer)
                                {
                                    self.translate_y = translate;
                                    layer.translate(self.translate_x, self.translate_y);
                                    redraw = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        redraw
    }
}

fn create_buffer(color: [u8; 4], w: usize, h: usize) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(w * h * 4);

    buffer.extend((0..w * h).flat_map(|_| color));

    buffer
}
