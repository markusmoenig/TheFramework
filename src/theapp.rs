// Platform-aware accelerator modifiers (AltGr-safe)
#[cfg(feature = "winit_app")]
#[inline]
fn is_accel_mods(m: &winit::keyboard::ModifiersState) -> bool {
    #[cfg(target_os = "macos")]
    {
        // macOS commonly uses ⌘ and ⌥ for accelerators
        m.super_key() || m.alt_key() || m.control_key()
    }
    #[cfg(not(target_os = "macos"))]
    {
        // On Windows/Linux, avoid treating Alt as an accelerator by default (AltGr = Ctrl+Alt)
        m.super_key() || m.control_key()
    }
}

// US-ANSI physical keycode to ascii (letters, digits, punctuation, space)
#[cfg(feature = "winit_app")]
fn accel_physical_to_ascii(code: winit::keyboard::KeyCode, shift: bool) -> Option<char> {
    use winit::keyboard::KeyCode;
    let (base, shifted) = match code {
        // Letters
        KeyCode::KeyA => ('a', 'A'),
        KeyCode::KeyB => ('b', 'B'),
        KeyCode::KeyC => ('c', 'C'),
        KeyCode::KeyD => ('d', 'D'),
        KeyCode::KeyE => ('e', 'E'),
        KeyCode::KeyF => ('f', 'F'),
        KeyCode::KeyG => ('g', 'G'),
        KeyCode::KeyH => ('h', 'H'),
        KeyCode::KeyI => ('i', 'I'),
        KeyCode::KeyJ => ('j', 'J'),
        KeyCode::KeyK => ('k', 'K'),
        KeyCode::KeyL => ('l', 'L'),
        KeyCode::KeyM => ('m', 'M'),
        KeyCode::KeyN => ('n', 'N'),
        KeyCode::KeyO => ('o', 'O'),
        KeyCode::KeyP => ('p', 'P'),
        KeyCode::KeyQ => ('q', 'Q'),
        KeyCode::KeyR => ('r', 'R'),
        KeyCode::KeyS => ('s', 'S'),
        KeyCode::KeyT => ('t', 'T'),
        KeyCode::KeyU => ('u', 'U'),
        KeyCode::KeyV => ('v', 'V'),
        KeyCode::KeyW => ('w', 'W'),
        KeyCode::KeyX => ('x', 'X'),
        KeyCode::KeyY => ('y', 'Y'),
        KeyCode::KeyZ => ('z', 'Z'),
        // Digits (US)
        KeyCode::Digit0 => ('0', ')'),
        KeyCode::Digit1 => ('1', '!'),
        KeyCode::Digit2 => ('2', '@'),
        KeyCode::Digit3 => ('3', '#'),
        KeyCode::Digit4 => ('4', '$'),
        KeyCode::Digit5 => ('5', '%'),
        KeyCode::Digit6 => ('6', '^'),
        KeyCode::Digit7 => ('7', '&'),
        KeyCode::Digit8 => ('8', '*'),
        KeyCode::Digit9 => ('9', '('),
        // Punctuation (US ANSI)
        KeyCode::Minus => ('-', '_'),
        KeyCode::Equal => ('=', '+'),
        KeyCode::BracketLeft => ('[', '{'),
        KeyCode::BracketRight => (']', '}'),
        KeyCode::Semicolon => (';', ':'),
        KeyCode::Quote => ('\'', '"'),
        KeyCode::Comma => (',', '<'),
        KeyCode::Period => ('.', '>'),
        KeyCode::Slash => ('/', '?'),
        KeyCode::Backquote => ('`', '~'),
        KeyCode::Backslash => ('\\', '|'),
        KeyCode::IntlBackslash => ('\\', '|'),
        // Space
        KeyCode::Space => (' ', ' '),
        _ => return None,
    };
    Some(if shift { shifted } else { base })
}
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
        event::{
            ElementState, Event, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent,
        },
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
    // Track modifiers for accelerator handling (for winit versions without event.modifiers)
    let mut mods: winit::keyboard::ModifiersState = winit::keyboard::ModifiersState::empty();

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
                    WindowEvent::Touch(Touch {
                        phase, location, ..
                    }) => {
                        let (x, y) = (location.x as u32, location.y as u32);

                        #[cfg(feature = "gpu")]
                        let (x, y) = ctx.gpu.translate_coord_to_local(x, y);

                        #[cfg(feature = "cpu_render")]
                        let (x, y) = translate_coord_to_local(x, y, ctx.scale_factor);

                        #[cfg(feature = "pixels_winit")]
                        let (x, y) = {
                            // Convert logical window coords to pixel coords similarly to mouse path
                            let logical = (location.x as f32, location.y as f32);
                            let pos = pixels
                                .window_pos_to_pixel(logical)
                                .unwrap_or_else(|p| pixels.clamp_pixel_pos(p));
                            (pos.0, pos.1)
                        };

                        match phase {
                            TouchPhase::Started => {
                                let mut redraw = false;
                                #[cfg(feature = "ui")]
                                {
                                    if ui.touch_down(x as f32, y as f32, &mut ctx) {
                                        redraw = true;
                                    }
                                }
                                if app.touch_down(x as f32, y as f32, &mut ctx) {
                                    redraw = true;
                                }

                                if redraw {
                                    window.request_redraw();
                                }
                            }
                            TouchPhase::Moved => {
                                let mut redraw = false;
                                #[cfg(feature = "ui")]
                                {
                                    if ui.touch_dragged(x as f32, y as f32, &mut ctx) {
                                        redraw = true;
                                    }
                                }
                                if app.touch_dragged(x as f32, y as f32, &mut ctx) {
                                    redraw = true;
                                }
                                if redraw {
                                    window.request_redraw();
                                }
                            }
                            TouchPhase::Ended | TouchPhase::Cancelled => {
                                let mut redraw = false;
                                #[cfg(feature = "ui")]
                                {
                                    if ui.touch_up(x as f32, y as f32, &mut ctx) {
                                        redraw = true;
                                    }
                                }
                                if app.touch_up(x as f32, y as f32, &mut ctx) {
                                    redraw = true;
                                }

                                if redraw {
                                    window.request_redraw();
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

                        // We always call this for apps who use the "ui" feature
                        // but do not use the UI API
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
                            let buffer_data = convert_rgba_to_softbuffer(
                                &ui_frame,
                                width,
                                height,
                                ctx.scale_factor as usize,
                            );
                            let mut buffer = ctx.surface.buffer_mut().unwrap();
                            buffer.copy_from_slice(&buffer_data);
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
                        // keep a copy of current modifiers for accelerator checks
                        mods = state;

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
                                    // Accelerator: use physical key with modifiers (ignore composed text like "å")
                                    if is_accel_mods(&mods) {
                                        if let winit::keyboard::PhysicalKey::Code(code) =
                                            event.physical_key
                                        {
                                            if let Some(ch) =
                                                accel_physical_to_ascii(code, mods.shift_key())
                                            {
                                                #[cfg(feature = "ui")]
                                                if ui.key_down(Some(ch), None, &mut ctx) {
                                                    window.request_redraw();
                                                }
                                                if app.key_down(Some(ch), None, &mut ctx) {
                                                    window.request_redraw();
                                                }
                                                return; // handled as accelerator
                                            }
                                        }
                                    }
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
                                    // Accelerator release: use physical key with modifiers (ignore composed text)
                                    if is_accel_mods(&mods) {
                                        if let winit::keyboard::PhysicalKey::Code(code) =
                                            event.physical_key
                                        {
                                            if let Some(ch) =
                                                accel_physical_to_ascii(code, mods.shift_key())
                                            {
                                                #[cfg(feature = "ui")]
                                                if ui.key_up(Some(ch), None, &mut ctx) {
                                                    window.request_redraw();
                                                }
                                                if app.key_up(Some(ch), None, &mut ctx) {
                                                    window.request_redraw();
                                                }
                                                return;
                                            }
                                        }
                                    }
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
                                #[cfg(feature = "ui")]
                                if ui.key_up(None, key.clone(), &mut ctx) {
                                    window.request_redraw();
                                }
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

#[cfg(feature = "cpu_render")]
fn convert_rgba_to_softbuffer(
    ui_frame: &[u8],
    width: usize,
    height: usize,
    scale_factor: usize,
) -> Vec<u32> {
    use rayon::prelude::*;

    let dest_width = width * scale_factor;
    let dest_height = height * scale_factor;

    if scale_factor == 1 {
        let mut buffer = vec![0u32; dest_width * dest_height];
        buffer.par_iter_mut().enumerate().for_each(|(i, px)| {
            let index = i * 4;
            let red = ui_frame[index] as u32;
            let green = ui_frame[index + 1] as u32;
            let blue = ui_frame[index + 2] as u32;
            *px = blue | (green << 8) | (red << 16);
        });
        buffer
    } else {
        let mut buffer = vec![0u32; dest_width * dest_height];

        // Each source row y maps to a contiguous chunk of `scale_factor` destination rows.
        let sf = scale_factor;
        buffer
            .par_chunks_mut(dest_width * sf)
            .enumerate()
            .for_each(|(y, chunk)| {
                for x in 0..width {
                    let src_index = (y * width + x) * 4;
                    let r = ui_frame[src_index] as u32;
                    let g = ui_frame[src_index + 1] as u32;
                    let b = ui_frame[src_index + 2] as u32;
                    let color = b | (g << 8) | (r << 16);

                    // Write a sf×sf block at (x*sf, 0..sf) across the `sf` destination rows for this source row.
                    for y2 in 0..sf {
                        let row = &mut chunk[y2 * dest_width..(y2 + 1) * dest_width];
                        let start = x * sf;
                        let end = start + sf;
                        row[start..end].fill(color);
                    }
                }
            });

        buffer
    }
}
