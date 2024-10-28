use std::{cell::OnceCell, sync::mpsc::Receiver};

use theframework::prelude::*;

use crate::compute::Compute;

pub struct Demo {
    canvas_layer: usize,
    translate_x: f32,
    translate_y: f32,

    compute: OnceCell<Compute>,
    compute_enable: bool,
    compute_output_layer: usize,
    compute_texture: Option<TheTextureId>,

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

            compute: OnceCell::new(),
            compute_enable: false,
            compute_output_layer: 0,
            compute_texture: None,

            event_receiver: None,
        }
    }

    fn init(&mut self, ctx: &mut TheContext) {
        self.canvas_layer = ctx.texture_renderer.add_layer();
        // Set zindex < 0 so that the ui layer can always be on top
        ctx.texture_renderer.set_layer_zindex(self.canvas_layer, -1);

        let _ = self
            .compute
            .set(Compute::new(ctx.gpu.device(), ctx.gpu.queue()));
        self.compute_output_layer = ctx.texture_renderer.add_layer();
        if let Some(layer) = ctx.texture_renderer.layer_mut(self.compute_output_layer) {
            layer.scale(0.2);
        }
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        let sidebar_width: i32 = 400;

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

        let mut enable_compute = TheCheckButton::new(TheId::named("EnableCompute"));
        if self.compute_enable {
            enable_compute.set_state(TheWidgetState::Selected);
        }
        layout.add_pair(
            "Enable Compute Shader".to_string(),
            Box::new(enable_compute),
        );

        layout.set_background_color(Some(SectionbarBackground));

        let mut sidebar = TheCanvas::new();
        sidebar.set_layout(layout);

        ui.canvas.set_right(sidebar);

        self.event_receiver = Some(ui.add_state_listener("Main".into()));
    }

    fn post_captured(&mut self, texture: Vec<u8>, width: u32, height: u32) {
        // Handle screen buffer here.
    }

    fn post_ui(&mut self, ctx: &mut TheContext) {
        if let Some(compute_texture) = self.compute_texture.take() {
            ctx.texture_renderer.unload_texture(compute_texture);
            if let Some(layer) = ctx.texture_renderer.layer_mut(self.compute_output_layer) {
                layer.clear();
            }
        }
    }

    fn pre_ui(&mut self, ctx: &mut TheContext) {
        if self.compute_enable {
            let compute = self.compute.get().unwrap();
            ctx.gpu.compute(compute).unwrap();

            let device = ctx.gpu.device();
            let buffer = compute.buffer(device);

            let width = compute.width();
            let height = compute.height();
            let compute_texture =
                ctx.texture_renderer
                    .load_texture(device, ctx.gpu.queue(), width, height, &buffer);
            ctx.texture_renderer.place_texture(
                self.compute_output_layer,
                compute_texture,
                Vec2::zero(),
            );

            self.compute_texture = Some(compute_texture);
        }
    }

    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;

        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    TheEvent::StateChanged(id, state) => {
                        if id.name == "EnableCompute" {
                            self.compute_enable = state == TheWidgetState::Selected;
                            redraw = true;
                        }
                    }
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
