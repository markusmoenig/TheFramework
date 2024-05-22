use crate::prelude::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TheNodeUIImages {
    NormalTopLeft,
    NormalTopMiddle,
    NormalTopRight,
    SelectedTopLeft,
    SelectedTopMiddle,
    SelectedTopRight,
    NormalBottomLeft,
    NormalBottomMiddle,
    NormalBottomRight,
    SelectedBottomLeft,
    SelectedBottomMiddle,
    SelectedBottomRight,
}

use TheNodeUIImages::*;

pub struct TheNodeCanvasView {
    id: TheId,
    limiter: TheSizeLimiter,
    state: TheWidgetState,

    render_buffer: TheRGBABuffer,
    wheel_scale: f32,
    accumulated_wheel_delta: Vec2f,

    canvas: TheNodeCanvas,
    node_rects: Vec<TheDim>,

    dim: TheDim,

    is_dirty: bool,

    node_ui_images: FxHashMap<TheNodeUIImages, TheRGBABuffer>,
}

impl TheWidget for TheNodeCanvasView {
    fn new(id: TheId) -> Self
    where
        Self: Sized,
    {
        let mut limiter = TheSizeLimiter::new();
        limiter.set_max_size(vec2i(20, 20));
        Self {
            id,
            limiter,
            state: TheWidgetState::None,

            render_buffer: TheRGBABuffer::new(TheDim::new(0, 0, 20, 20)),
            wheel_scale: -0.4,
            accumulated_wheel_delta: Vec2f::zero(),

            canvas: TheNodeCanvas::default(),
            node_rects: Vec::new(),

            dim: TheDim::zero(),

            is_dirty: false,

            node_ui_images: FxHashMap::default(),
        }
    }

    fn id(&self) -> &TheId {
        &self.id
    }

    #[allow(clippy::single_match)]
    fn on_event(&mut self, event: &TheEvent, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        // println!("event ({}): {:?}", self.widget_id.name, event);
        match event {
            TheEvent::MouseDown(coord) => {
                if self.state == TheWidgetState::Selected {
                    self.state = TheWidgetState::None;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                } else if self.state != TheWidgetState::Selected {
                    self.state = TheWidgetState::Selected;
                    ctx.ui.send_widget_state_changed(self.id(), self.state);
                }
                ctx.ui.set_focus(self.id());

                let selected = self.node_index_at(coord);
                if selected.is_some() && selected != self.canvas.selected_node {
                    self.canvas.selected_node = selected;
                    self.is_dirty = true;
                    redraw = true;

                    ctx.ui.send(TheEvent::NodeSelectedIndexChanged(
                        self.id().clone(),
                        selected,
                    ));
                }
            }
            TheEvent::MouseDragged(coord) => {
                ctx.ui
                    .send(TheEvent::RenderViewDragged(self.id().clone(), *coord));
            }
            TheEvent::Hover(coord) => {
                if !self.id().equals(&ctx.ui.hover) {
                    self.is_dirty = true;
                    ctx.ui.set_hover(self.id());
                    redraw = true;
                }

                ctx.ui
                    .send(TheEvent::RenderViewHoverChanged(self.id().clone(), *coord));
            }
            TheEvent::LostHover(_) => {
                ctx.ui
                    .send(TheEvent::RenderViewLostHover(self.id().clone()));
            }
            TheEvent::MouseWheel(delta) => {
                let scale_factor = self.wheel_scale; // * 1.0 / (self.zoom.powf(0.5));

                let aspect_ratio = self.dim().width as f32 / self.dim().height as f32;

                let scale_x = if aspect_ratio > 1.0 {
                    1.0 / aspect_ratio
                } else {
                    1.0
                };
                let scale_y = if aspect_ratio < 1.0 {
                    aspect_ratio
                } else {
                    1.0
                };

                // Update accumulated deltas
                self.accumulated_wheel_delta.x += delta.x as f32 * scale_factor * scale_x;
                self.accumulated_wheel_delta.y += delta.y as f32 * scale_factor * scale_y;

                let minimum_delta_threshold = 2.0;

                // Check if accumulated deltas exceed the threshold
                if self.accumulated_wheel_delta.x.abs() > minimum_delta_threshold
                    || self.accumulated_wheel_delta.y.abs() > minimum_delta_threshold
                {
                    // Convert accumulated deltas to integer and reset
                    let d = vec2i(
                        self.accumulated_wheel_delta.x as i32,
                        self.accumulated_wheel_delta.y as i32,
                    );
                    self.accumulated_wheel_delta = Vec2f::zero();

                    //ctx.ui
                    //    .send(TheEvent::RenderViewScrollBy(self.id().clone(), d));
                    self.canvas.offset += d;

                    self.is_dirty = true;
                    redraw = true;
                }
            }
            _ => {}
        }
        redraw
    }

    fn dim(&self) -> &TheDim {
        &self.dim
    }

    fn dim_mut(&mut self) -> &mut TheDim {
        &mut self.dim
    }

    fn set_dim(&mut self, dim: TheDim) {
        if self.dim != dim {
            self.dim = dim;
            self.is_dirty = true;
            self.render_buffer.resize(dim.width, dim.height);
        }
    }

    fn set_state(&mut self, state: TheWidgetState) {
        self.state = state;
        self.is_dirty = true;
    }

    fn limiter(&self) -> &TheSizeLimiter {
        &self.limiter
    }

    fn limiter_mut(&mut self) -> &mut TheSizeLimiter {
        &mut self.limiter
    }

    fn needs_redraw(&mut self) -> bool {
        self.is_dirty
    }

    fn set_needs_redraw(&mut self, redraw: bool) {
        self.is_dirty = redraw;
    }

    fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        style: &mut Box<dyn TheStyle>,
        ctx: &mut TheContext,
    ) {
        if !self.dim().is_valid() {
            return;
        }

        if self.node_ui_images.is_empty() {
            self.fill_node_ui_images(ctx);
        }

        self.render_buffer.fill([128, 128, 128, 255]);

        let node_width = 60;
        self.node_rects.clear();
        let dest = Arc::new(Mutex::new(&mut self.render_buffer));
        let node_rects = Arc::new(Mutex::new(Vec::new()));

        // Draw a node
        let draw_node = |index: usize, node: &TheNode| {
            let max_terminals = max(node.inputs.len(), node.outputs.len()) as i32;
            let body_height = 7 + max_terminals * 10 + (max_terminals - 1) * 4 + 7;
            let node_height = 19 + body_height + 19;

            let dim = TheDim::new(
                node.position.x - self.canvas.offset.x,
                node.position.y - self.canvas.offset.y,
                node_width,
                node_height,
            );

            let mut nb = TheRGBABuffer::new(TheDim::sized(node_width, node_height));

            let is_selected = Some(index) == self.canvas.selected_node;

            // Header

            if is_selected {
                nb.copy_into(0, 0, self.node_ui_images.get(&SelectedTopLeft).unwrap());

                for i in 0..(node_width - 18) {
                    nb.copy_into(
                        9 + i,
                        0,
                        self.node_ui_images.get(&SelectedTopMiddle).unwrap(),
                    );
                }

                nb.copy_into(
                    node_width - 9,
                    0,
                    self.node_ui_images.get(&SelectedTopRight).unwrap(),
                );
            } else {
                nb.copy_into(2, 2, self.node_ui_images.get(&NormalTopLeft).unwrap());

                for i in 0..(node_width - 18) {
                    nb.copy_into(9 + i, 2, self.node_ui_images.get(&NormalTopMiddle).unwrap());
                }

                nb.copy_into(
                    node_width - 9,
                    2,
                    self.node_ui_images.get(&NormalTopRight).unwrap(),
                );
            }

            if let Some(font) = &ctx.ui.font {
                nb.draw_text(vec2i(12, 4), font, &node.name, 10.0, [188, 188, 188, 255]);
            }

            // Body
            for _y in 0..body_height {
                let y = _y + 19;
                for x in 0..node_width {
                    if x < 2 {
                        if is_selected {
                            if x == 0 {
                                nb.set_pixel(x, y, &[255, 255, 255, 55]);
                            } else {
                                nb.set_pixel(x, y, &[255, 255, 255, 166]);
                            }
                        }
                        continue;
                    } else if x >= node_width - 2 {
                        if is_selected {
                            if x == node_width - 1 {
                                nb.set_pixel(x, y, &[255, 255, 255, 55]);
                            } else {
                                nb.set_pixel(x, y, &[255, 255, 255, 166]);
                            }
                        }
                        continue;
                    }

                    if x == node_width - 3 || _y == body_height - 1 {
                        nb.set_pixel(x, y, &[44, 44, 44, 255]);
                    } else if x == node_width - 4 && _y > 1 {
                        nb.set_pixel(x, y, &[162, 162, 162, 255]);
                    } else if x == 2 {
                        nb.set_pixel(x, y, &[112, 112, 112, 255]);
                    } else if x == 3 && _y > 0 {
                        nb.set_pixel(x, y, &[137, 137, 137, 255]);
                    } else if _y == 0 {
                        nb.set_pixel(x, y, &[82, 82, 82, 255]);
                    } else if _y == 1 {
                        nb.set_pixel(x, y, &[137, 137, 137, 255]);
                    } else if _y == body_height - 2 {
                        nb.set_pixel(x, y, &[162, 162, 162, 255]);
                    } else {
                        nb.set_pixel(x, y, &[179, 179, 179, 255]);
                    }
                }
            }

            // Footer
            if is_selected {
                nb.copy_into(
                    0,
                    node_height - 19,
                    self.node_ui_images.get(&SelectedBottomLeft).unwrap(),
                );

                for i in 0..(node_width - 18) {
                    nb.copy_into(
                        9 + i,
                        node_height - 19,
                        self.node_ui_images.get(&SelectedBottomMiddle).unwrap(),
                    );
                }

                nb.copy_into(
                    node_width - 9,
                    node_height - 19,
                    self.node_ui_images.get(&SelectedBottomRight).unwrap(),
                );
            } else {
                nb.copy_into(
                    2,
                    node_height - 19,
                    self.node_ui_images.get(&NormalBottomLeft).unwrap(),
                );

                for i in 0..(node_width - 18) {
                    nb.copy_into(
                        9 + i,
                        node_height - 19,
                        self.node_ui_images.get(&NormalBottomMiddle).unwrap(),
                    );
                }

                nb.copy_into(
                    node_width - 9,
                    node_height - 19,
                    self.node_ui_images.get(&NormalBottomRight).unwrap(),
                );
            }

            // Finished write back to dest
            let mut dest = dest.lock().unwrap();
            dest.blend_into(dim.x, dim.y, &nb);

            let mut node_rects = node_rects.lock().unwrap();
            node_rects.push((index, dim));
        };

        // Parallel iteration over the nodes except the selected node
        self.canvas
            .nodes
            .par_iter()
            .enumerate()
            .for_each(|(index, node)| {
                if Some(index) != self.canvas.selected_node {
                    draw_node(index, node);
                }
            });

        // Draw the selected node afterwards to draw it is always on top
        if let Some(selected_index) = self.canvas.selected_node {
            if let Some(node) = self.canvas.nodes.get(selected_index) {
                draw_node(selected_index, node);
            }
        }

        // Copy and sort node_rects back to self.node_rects
        let mut sorted_node_rects = Arc::try_unwrap(node_rects).unwrap().into_inner().unwrap();
        sorted_node_rects.sort_by_key(|&(index, _)| index);
        self.node_rects = sorted_node_rects.into_iter().map(|(_, dim)| dim).collect();

        // Copy the render buffer to the main buffer
        buffer.copy_into(self.dim.buffer_x, self.dim.buffer_y, &self.render_buffer);

        // Draw the focus rectangle if necessary
        let stride = buffer.stride();
        if Some(self.id.clone()) == ctx.ui.focus {
            let tuple = self.dim().to_buffer_utuple();
            ctx.draw.rect_outline(
                buffer.pixels_mut(),
                &tuple,
                stride,
                style.theme().color(DefaultSelection),
            );
        }
        self.is_dirty = false;
    }

    fn supports_hover(&mut self) -> bool {
        true
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_node_canvas_view(&mut self) -> Option<&mut dyn TheNodeCanvasViewTrait> {
        Some(self)
    }
}

pub trait TheNodeCanvasViewTrait: TheWidget {
    fn set_canvas(&mut self, canvas: TheNodeCanvas);
    fn node_index_at(&self, coord: &Vec2i) -> Option<usize>;
    fn fill_node_ui_images(&mut self, ctx: &mut TheContext);
}

impl TheNodeCanvasViewTrait for TheNodeCanvasView {
    fn set_canvas(&mut self, canvas: TheNodeCanvas) {
        self.canvas = canvas;
        self.is_dirty = true;
    }
    fn fill_node_ui_images(&mut self, ctx: &mut TheContext) {
        self.node_ui_images.clear();
        self.node_ui_images.insert(
            SelectedTopLeft,
            ctx.ui.icon("dark_node_selected_topleft").unwrap().clone(),
        );
        self.node_ui_images.insert(
            SelectedTopMiddle,
            ctx.ui.icon("dark_node_selected_topmiddle").unwrap().clone(),
        );
        self.node_ui_images.insert(
            SelectedTopRight,
            ctx.ui.icon("dark_node_selected_topright").unwrap().clone(),
        );
        self.node_ui_images.insert(
            NormalTopLeft,
            ctx.ui.icon("dark_node_normal_topleft").unwrap().clone(),
        );
        self.node_ui_images.insert(
            NormalTopMiddle,
            ctx.ui.icon("dark_node_normal_topmiddle").unwrap().clone(),
        );
        self.node_ui_images.insert(
            NormalTopRight,
            ctx.ui.icon("dark_node_normal_topright").unwrap().clone(),
        );
        self.node_ui_images.insert(
            SelectedBottomLeft,
            ctx.ui
                .icon("dark_node_selected_bottomleft")
                .unwrap()
                .clone(),
        );
        self.node_ui_images.insert(
            SelectedBottomMiddle,
            ctx.ui
                .icon("dark_node_selected_bottommiddle")
                .unwrap()
                .clone(),
        );
        self.node_ui_images.insert(
            SelectedBottomRight,
            ctx.ui
                .icon("dark_node_selected_bottomright")
                .unwrap()
                .clone(),
        );
        self.node_ui_images.insert(
            NormalBottomLeft,
            ctx.ui.icon("dark_node_normal_bottomleft").unwrap().clone(),
        );
        self.node_ui_images.insert(
            NormalBottomMiddle,
            ctx.ui
                .icon("dark_node_normal_bottommiddle")
                .unwrap()
                .clone(),
        );
        self.node_ui_images.insert(
            NormalBottomRight,
            ctx.ui.icon("dark_node_normal_bottomright").unwrap().clone(),
        );
    }
    fn node_index_at(&self, coord: &Vec2i) -> Option<usize> {
        for (i, r) in self.node_rects.iter().enumerate() {
            if r.contains(*coord) {
                return Some(i);
            }
        }
        None
    }
}
