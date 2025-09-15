use crate::prelude::*;
use web_time::{Duration, Instant};

#[cfg(feature = "cpu_render")]
pub fn translate_coord_to_local(x: u32, y: u32, scale_factor: f32) -> (u32, u32) {
    (
        (x as f32 / scale_factor) as u32,
        (y as f32 / scale_factor) as u32,
    )
}

/// TheApp class handles running an application on the current backend.
pub struct TheApp {
    #[cfg(feature = "ui")]
    pub ui: TheUI,
    pub args: Option<Vec<String>>,
}

impl Default for TheApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TheApp {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "ui")]
            ui: TheUI::new(),
            args: None,
        }
    }

    /// Optionally set the command line arguments of the app.
    pub fn set_cmd_line_args(&mut self, args: Vec<String>) {
        self.args = Some(args);
    }

    /// Runs the app
    pub fn run(self, app: Box<dyn crate::TheTrait>) {
        #[cfg(feature = "log")]
        setup_logger();

        #[cfg(all(feature = "winit_app", not(target_arch = "wasm32")))]
        futures::executor::block_on(run_app(self, app));

        #[cfg(all(feature = "winit_app", target_arch = "wasm32"))]
        wasm_bindgen_futures::spawn_local(run_app(self, app));
    }
}

#[allow(unused_assignments)]
#[allow(unused_variables)]
#[cfg(feature = "winit_app")]
async fn run_app(mut framework: TheApp, mut app: Box<dyn crate::TheTrait>) {
    #[cfg(feature = "cpu_render")]
    use std::num::NonZeroU32;
    use std::sync::Arc;

    use log::error;
    #[cfg(feature = "pixels_render")]
    use pixels::{Pixels, SurfaceTexture};
    use winit::dpi::LogicalSize;
    use winit::{
        event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        keyboard::{Key, NamedKey},
        window::{Icon, WindowBuilder},
    };
    use winit_input_helper::WinitInputHelper;

    let (mut width, mut height) = app.default_window_size();

    let window_title = app.window_title();
    let mut icon: Option<Icon> = None;
    if let Some(window_icon) = app.window_icon() {
        icon = Icon::from_rgba(window_icon.0, window_icon.1, window_icon.2).ok();
    }

    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();

    let size = LogicalSize::new(width as f64, height as f64);

    let builder = WindowBuilder::new()
        .with_title(window_title)
        .with_inner_size(size)
        .with_min_inner_size(size)
        .with_window_icon(icon); //TODO on Windows
    #[cfg(target_arch = "wasm32")]
    let builder = {
        use winit::platform::web::WindowBuilderExtWebSys;
        builder.with_append(true)
    };

    let window = builder.build(&event_loop).unwrap();
    let window = Arc::new(window);

    #[cfg(feature = "gpu_winit")]
    let (gpu, texture_renderer, ui_layer) = {
        let mut gpu = TheGpuContext::new().await.unwrap();
        let surface = gpu.create_surface(window.clone()).unwrap();
        gpu.set_surface(
            width as u32,
            height as u32,
            window.scale_factor() as f32,
            surface,
        );

        let mut texture_renderer = TheTextureRenderPass::new(gpu.device());
        let ui_layer = texture_renderer.add_layer();

        (gpu, texture_renderer, ui_layer)
    };

    #[cfg(all(feature = "gpu", not(feature = "gpu_winit")))]
    panic!("No suitable gpu backend was set.");

    #[cfg(all(feature = "pixels_winit", not(target_arch = "wasm32")))]
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.clone());
        Pixels::new(width as u32, height as u32, surface_texture).unwrap()
    };

    #[cfg(all(feature = "pixels_winit", target_arch = "wasm32"))]
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(
            window_size.width.max(1),
            window_size.height.max(1),
            window.clone(),
        );
        Pixels::new_async(
            (width as u32).max(1),
            (height as u32).max(1),
            surface_texture,
        )
        .await
        .unwrap()
    };

    #[cfg(any(feature = "gpu", feature = "cpu_render"))]
    let mut ui_frame = vec![0; width * height * 4];

    #[cfg(feature = "gpu")]
    let mut ctx = TheContext::new(width, height, gpu, texture_renderer);
    #[cfg(feature = "cpu_render")]
    let mut ctx = TheContext::new(width, height, window.clone());
    #[cfg(not(any(feature = "gpu", feature = "cpu_render")))]
    let mut ctx = TheContext::new(width, height);

    #[cfg(feature = "ui")]
    let mut ui = TheUI::new();
    #[cfg(feature = "ui")]
    ui.init(&mut ctx);

    app.init(&mut ctx);

    // If available set the command line arguments to the trait.
    if let Some(args) = framework.args.take() {
        app.set_cmd_line_args(args, &mut ctx);
    }

    #[cfg(feature = "ui")]
    {
        ui.canvas.root = true;
        ui.canvas
            .set_dim(TheDim::new(0, 0, width as i32, height as i32), &mut ctx);

        app.init_ui(&mut ui, &mut ctx);
        ui.canvas.layout(width as i32, height as i32, &mut ctx);
    }

    // Setup the target frame time
    let target_frame_time = Duration::from_secs_f64(1.0 / app.target_fps());
    let mut last_frame_time = Instant::now();

    // Loop
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| {
            match &event {
                // Try to maintain the target frame time
                Event::AboutToWait => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_frame_time);

                    if elapsed >= target_frame_time {
                        last_frame_time = now;
                        window.request_redraw();
                    } else {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            std::thread::sleep(target_frame_time - elapsed);
                        }
                    }

                    elwt.set_control_flow(ControlFlow::Poll);
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        if app.closing() {
                            elwt.exit();
                            return;
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        match delta {
                            MouseScrollDelta::LineDelta(x, y) => {
                                // println!("mouse wheel Line Delta: ({},{})", x, y);

                                #[cfg(feature = "ui")]
                                if ui.mouse_wheel((*x as i32 * 10, *y as i32 * 10), &mut ctx) {
                                    window.request_redraw();
                                }

                                if app.mouse_wheel((*x as isize * 10, *y as isize * 10), &mut ctx) {
                                    window.request_redraw();
                                    //mouse_wheel_ongoing = true;
                                }

                                if *x == 0.0 && *y == 0.0 {
                                    // mouse_wheel_ongoing = false;
                                }
                            }
                            MouseScrollDelta::PixelDelta(p) => {
                                // println!("mouse wheel Pixel Delta: ({},{})", p.x, p.y);
                                #[cfg(feature = "ui")]
                                if ui.mouse_wheel((p.x as i32, p.y as i32), &mut ctx) {
                                    window.request_redraw();
                                }

                                if app.mouse_wheel((p.x as isize, p.y as isize), &mut ctx) {
                                    window.request_redraw();
                                    //mouse_wheel_ongoing = true;
                                }

                                if p.x == 0.0 && p.y == 0.0 {
                                    // mouse_wheel_ongoing = false;
                                }
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        #[cfg(feature = "gpu_winit")]
                        if let Err(err) = ctx.gpu.begin_frame() {
                            error!("Failed to begin next frame: {} ", err);
                            elwt.exit();
                            return;
                        }

                        #[cfg(feature = "pixels_winit")]
                        let mut ui_frame = pixels.frame_mut();

                        #[cfg(feature = "ui")]
                        app.pre_ui(&mut ctx);

                        #[cfg(feature = "ui")]
                        ui.draw(&mut ui_frame, &mut ctx);

                        #[cfg(not(feature = "ui"))]
                        app.draw(&mut ui_frame, &mut ctx);

                        #[cfg(feature = "gpu_winit")]
                        let ui_texture = {
                            let ui_texture = ctx.texture_renderer.load_texture(
                                ctx.gpu.device(),
                                ctx.gpu.queue(),
                                width as u32,
                                height as u32,
                                &ui_frame,
                            );
                            ctx.texture_renderer.place_texture(
                                ui_layer,
                                ui_texture,
                                Vec2::new(0.0, 0.0),
                            );

                            ui_texture
                        };

                        #[cfg(feature = "gpu")]
                        if ctx
                            .gpu
                            .draw(&ctx.texture_renderer)
                            .map_err(|e| error!("render failed: {}", e))
                            .is_err()
                        {
                            elwt.exit();
                            return;
                        }

                        #[cfg(feature = "pixels_winit")]
                        if pixels
                            .render()
                            .map_err(|e| error!("pixels.render() failed: {}", e))
                            .is_err()
                        {
                            elwt.exit();
                            return;
                        }

                        #[cfg(feature = "cpu_render")]
                        {
                            let mut buffer = ctx.surface.buffer_mut().unwrap();
                            let scale_factor = ctx.scale_factor as usize;

                            if scale_factor == 1 {
                                for i in 0..(width * height) {
                                    let index = i * 4;
                                    let red = ui_frame[index] as u32;
                                    let green = ui_frame[index + 1] as u32;
                                    let blue = ui_frame[index + 2] as u32;

                                    buffer[i] = blue | (green << 8) | (red << 16);
                                }
                            } else {
                                let dest_width = width * scale_factor;
                                // let dest_height = height * scale_factor;
                                for y in 0..height {
                                    for x in 0..width {
                                        let src_index = (y * width + x) * 4;
                                        let red = ui_frame[src_index] as u32;
                                        let green = ui_frame[src_index + 1] as u32;
                                        let blue = ui_frame[src_index + 2] as u32;

                                        // Write the pixel into the scaled region
                                        for y2 in 0..scale_factor as usize {
                                            for x2 in 0..scale_factor as usize {
                                                let dest_x = x * scale_factor as usize + x2;
                                                let dest_y = y * scale_factor as usize + y2;

                                                let dest_index = dest_y * dest_width + dest_x;
                                                buffer[dest_index] =
                                                    blue | (green << 8) | (red << 16);
                                            }
                                        }
                                    }
                                }
                            }

                            if buffer
                                .present()
                                .map_err(|e| error!("render failed: {}", e))
                                .is_err()
                            {
                                elwt.exit();
                                return;
                            }
                        }

                        #[cfg(feature = "ui")]
                        app.post_ui(&mut ctx);

                        #[cfg(feature = "gpu_winit")]
                        {
                            ctx.texture_renderer.unload_texture(ui_texture);
                            if let Some(layer) = ctx.texture_renderer.layer_mut(ui_layer) {
                                layer.clear();
                            }
                        }

                        #[cfg(feature = "gpu_winit")]
                        if let Err(err) = ctx.gpu.end_frame() {
                            error!("Failed to end current frame: {} ", err);
                        }
                    }
                    WindowEvent::DroppedFile(path) => {
                        app.dropped_file(path.to_str().unwrap().to_string());
                        window.request_redraw();
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        let state = modifiers.state();

                        #[cfg(feature = "ui")]
                        if ui.modifier_changed(
                            state.shift_key(),
                            state.control_key(),
                            state.alt_key(),
                            state.super_key(),
                            &mut ctx,
                        ) {
                            window.request_redraw();
                        }
                        if app.modifier_changed(
                            state.shift_key(),
                            state.control_key(),
                            state.alt_key(),
                            state.super_key(),
                        ) {
                            window.request_redraw();
                        }
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed {
                            let key = match &event.logical_key {
                                Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                                    Some(TheKeyCode::Delete)
                                }
                                Key::Named(NamedKey::ArrowUp) => Some(TheKeyCode::Up),
                                Key::Named(NamedKey::ArrowRight) => Some(TheKeyCode::Right),
                                Key::Named(NamedKey::ArrowDown) => Some(TheKeyCode::Down),
                                Key::Named(NamedKey::ArrowLeft) => Some(TheKeyCode::Left),
                                Key::Named(NamedKey::Space) => Some(TheKeyCode::Space),
                                Key::Named(NamedKey::Tab) => Some(TheKeyCode::Tab),
                                Key::Named(NamedKey::Enter) => Some(TheKeyCode::Return),
                                Key::Named(NamedKey::Escape) => Some(TheKeyCode::Escape),
                                Key::Character(str) => {
                                    if str.is_ascii() {
                                        for ch in str.chars() {
                                            #[cfg(feature = "ui")]
                                            if ui.key_down(Some(ch), None, &mut ctx) {
                                                window.request_redraw();
                                            }
                                            if app.key_down(Some(ch), None, &mut ctx) {
                                                window.request_redraw();
                                            }
                                        }
                                    }
                                    None
                                }
                                _ => None,
                            };
                            if key.is_some() {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, key.clone(), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, key, &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                        }
                        if event.state == ElementState::Released {
                            let key = match &event.logical_key {
                                Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                                    Some(TheKeyCode::Delete)
                                }
                                Key::Named(NamedKey::ArrowUp) => Some(TheKeyCode::Up),
                                Key::Named(NamedKey::ArrowRight) => Some(TheKeyCode::Right),
                                Key::Named(NamedKey::ArrowDown) => Some(TheKeyCode::Down),
                                Key::Named(NamedKey::ArrowLeft) => Some(TheKeyCode::Left),
                                Key::Named(NamedKey::Space) => Some(TheKeyCode::Space),
                                Key::Named(NamedKey::Tab) => Some(TheKeyCode::Tab),
                                Key::Named(NamedKey::Enter) => Some(TheKeyCode::Return),
                                Key::Named(NamedKey::Escape) => Some(TheKeyCode::Escape),
                                Key::Character(str) => {
                                    if str.is_ascii() {
                                        for ch in str.chars() {
                                            #[cfg(feature = "ui")]
                                            if ui.key_up(Some(ch), None, &mut ctx) {
                                                window.request_redraw();
                                            }
                                            if app.key_up(Some(ch), None, &mut ctx) {
                                                window.request_redraw();
                                            }
                                        }
                                    }
                                    None
                                }
                                _ => None,
                            };
                            if key.is_some() {
                                // #[cfg(feature = "ui")]
                                // if ui.key_down(None, key.clone(), &mut ctx) {
                                //     window.request_redraw();
                                // }
                                if app.key_up(None, key, &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                        }
                    }
                    _ => (),
                },

                /*
                Event::DeviceEvent { event, .. } => match event {
                    // DeviceEvent::Text { codepoint } => {
                    //     println!("text: ({})", codepoint);
                    // }
                    DeviceEvent::MouseWheel { delta: _ } => {}
                    DeviceEvent::Added => {}
                    DeviceEvent::Removed => {}
                    DeviceEvent::MouseMotion { delta: _ } => {}
                    DeviceEvent::Motion { axis: _, value: _ } => {}
                    DeviceEvent::Button {
                        button: _,
                        state: _,
                    } => {}
                    DeviceEvent::Key(_) => {}
                },*/
                _ => (),
            }

            // Handle input events
            if input.update(&event) {
                // Close events
                if
                /*input.key_pressed(VirtualKeyCode::Escape) ||*/
                input.close_requested() {
                    elwt.exit();
                    return;
                }

                if input.mouse_pressed(MouseButton::Left) {
                    if let Some(coords) = input.cursor() {
                        let (x, y) = (coords.0 as u32, coords.1 as u32);
                        #[cfg(feature = "gpu")]
                        let (x, y) = ctx.gpu.translate_coord_to_local(x, y);

                        #[cfg(feature = "cpu_render")]
                        let (x, y) = translate_coord_to_local(x, y, ctx.scale_factor);

                        #[cfg(feature = "pixels_winit")]
                        let (x, y) = pixels
                            .window_pos_to_pixel(coords)
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        #[cfg(feature = "ui")]
                        if ui.touch_down(x as f32, y as f32, &mut ctx) {
                            window.request_redraw();
                        }

                        if app.touch_down(x as f32, y as f32, &mut ctx) {
                            window.request_redraw();
                        }
                    }
                }

                if input.mouse_pressed(MouseButton::Right) {
                    if let Some(coords) = input.cursor() {
                        let (x, y) = (coords.0 as u32, coords.1 as u32);
                        #[cfg(feature = "gpu")]
                        let (x, y) = ctx.gpu.translate_coord_to_local(x, y);

                        #[cfg(feature = "cpu_render")]
                        let (x, y) = translate_coord_to_local(x, y, ctx.scale_factor);

                        #[cfg(feature = "pixels_winit")]
                        let (x, y) = pixels
                            .window_pos_to_pixel(coords)
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        #[cfg(feature = "ui")]
                        if ui.context(x as f32, y as f32, &mut ctx) {
                            window.request_redraw();
                        }

                        if app.touch_down(x as f32, y as f32, &mut ctx) {
                            window.request_redraw();
                        }
                    }
                }

                if input.mouse_released(MouseButton::Left) {
                    if let Some(coords) = input.cursor() {
                        let (x, y) = (coords.0 as u32, coords.1 as u32);
                        #[cfg(feature = "gpu")]
                        let (x, y) = ctx.gpu.translate_coord_to_local(x, y);

                        #[cfg(feature = "cpu_render")]
                        let (x, y) = translate_coord_to_local(x, y, ctx.scale_factor);

                        #[cfg(feature = "pixels_winit")]
                        let (x, y) = pixels
                            .window_pos_to_pixel(coords)
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        #[cfg(feature = "ui")]
                        if ui.touch_up(x as f32, y as f32, &mut ctx) {
                            window.request_redraw();
                        }

                        if app.touch_up(x as f32, y as f32, &mut ctx) {
                            window.request_redraw();
                        }
                    }
                }

                if input.mouse_held(MouseButton::Left) {
                    let diff = input.mouse_diff();
                    if diff.0 != 0.0 || diff.1 != 0.0 {
                        if let Some(coords) = input.cursor() {
                            let (x, y) = (coords.0 as u32, coords.1 as u32);
                            #[cfg(feature = "gpu")]
                            let (x, y) = ctx.gpu.translate_coord_to_local(x, y);

                            #[cfg(feature = "cpu_render")]
                            let (x, y) = translate_coord_to_local(x, y, ctx.scale_factor);

                            #[cfg(feature = "pixels_winit")]
                            let (x, y) = pixels
                                .window_pos_to_pixel(coords)
                                .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                            #[cfg(feature = "ui")]
                            if ui.touch_dragged(x as f32, y as f32, &mut ctx) {
                                window.request_redraw();
                            }

                            if app.touch_dragged(x as f32, y as f32, &mut ctx) {
                                window.request_redraw();
                            }
                        }
                    }
                } else {
                    let diff = input.mouse_diff();
                    if diff.0 != 0.0 || diff.1 != 0.0 {
                        if let Some(coords) = input.cursor() {
                            let (x, y) = (coords.0 as u32, coords.1 as u32);
                            #[cfg(feature = "gpu")]
                            let (x, y) = ctx.gpu.translate_coord_to_local(x, y);

                            #[cfg(feature = "cpu_render")]
                            let (x, y) = translate_coord_to_local(x, y, ctx.scale_factor);

                            #[cfg(feature = "pixels_winit")]
                            let (x, y) = pixels
                                .window_pos_to_pixel(coords)
                                .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                            #[cfg(feature = "ui")]
                            if ui.hover(x as f32, y as f32, &mut ctx) {
                                window.request_redraw();
                            }

                            if app.hover(x as f32, y as f32, &mut ctx) {
                                window.request_redraw();
                            }
                        }
                    }
                }

                // Resize the window
                if let Some(size) = input.window_resized() {
                    if size.width != 0 && size.height != 0 {
                        let scale = window.scale_factor() as f32;

                        #[cfg(feature = "gpu")]
                        {
                            ctx.gpu.resize(size.width, size.height);
                            ctx.gpu.set_scale_factor(scale);
                        }

                        #[cfg(feature = "cpu_render")]
                        ctx.surface
                            .resize(
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            )
                            .unwrap();

                        #[cfg(feature = "pixels_render")]
                        {
                            let _rc = pixels.resize_surface(size.width, size.height);
                            let _rc = pixels.resize_buffer(
                                (size.width as f32 / scale) as u32,
                                (size.height as f32 / scale) as u32,
                            );
                        }

                        width = (size.width as f32 / scale) as usize;
                        height = (size.height as f32 / scale) as usize;
                        ctx.width = width;
                        ctx.height = height;
                        ctx.scale_factor = scale;

                        #[cfg(any(feature = "gpu", feature = "cpu_render"))]
                        ui_frame.resize(width * height * 4, 0);

                        #[cfg(feature = "ui")]
                        ui.canvas
                            .set_dim(TheDim::new(0, 0, width as i32, height as i32), &mut ctx);
                        #[cfg(feature = "ui")]
                        ctx.ui.send(TheEvent::Resize);

                        window.request_redraw();
                    }
                }

                #[cfg(feature = "ui")]
                if ui.update(&mut ctx) {
                    window.request_redraw();
                }

                #[cfg(feature = "ui")]
                // Test if the app needs an update
                if app.update_ui(&mut ui, &mut ctx) {
                    window.request_redraw();
                }

                // Test if the app needs an update
                if app.update(&mut ctx) {
                    window.request_redraw();
                }
            }
        })
        .unwrap();
}
