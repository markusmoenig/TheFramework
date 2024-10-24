use crate::prelude::*;

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
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "winit_app")]
    pub fn run(&mut self, mut app: Box<dyn crate::TheTrait>) {
        use std::sync::Arc;

        use log::error;
        use winit::{
            dpi::{LogicalSize, PhysicalSize},
            event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
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

        let window = {
            if cfg!(target_os = "macos") {
                let size = LogicalSize::new(width as f64, height as f64);
                WindowBuilder::new()
                    .with_title(window_title)
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    .with_window_icon(icon)
                    .build(&event_loop)
                    .unwrap()
            } else {
                let size = PhysicalSize::new(width as f64, height as f64);
                WindowBuilder::new()
                    .with_title(window_title)
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    .with_window_icon(icon) //TODO on Windows
                    .build(&event_loop)
                    .unwrap()
            }
        };
        let window = Arc::new(window);

        #[cfg(feature = "pixels_winit")]
        let gpu = ThePixelsContext::from_window(window.clone()).unwrap();

        #[cfg(feature = "wgpu_winit")]
        let (gpu, ui_layer) = {
            let mut gpu =
                futures::executor::block_on(TheWgpuContext::with_default_shaders()).unwrap();
            let surface = gpu.create_surface(window.clone()).unwrap();
            gpu.set_surface(
                width as u32,
                height as u32,
                window.scale_factor() as f32,
                surface,
            );

            let ui_layer = gpu.add_layer();

            (gpu, ui_layer)
        };

        #[cfg(all(
            feature = "gpu",
            not(any(feature = "pixels_winit", feature = "wgpu_winit"))
        ))]
        panic!("No suitable gpu backend was set.");

        let mut ui_frame = vec![0; width * height * 4];

        #[cfg(feature = "gpu")]
        let mut ctx = TheContext::new(width, height, gpu);
        #[cfg(not(feature = "gpu"))]
        let mut ctx = TheContext::new(width, height);
        #[cfg(feature = "ui")]
        let mut ui = TheUI::new();
        #[cfg(feature = "ui")]
        ui.init(&mut ctx);

        app.init(&mut ctx);

        // If available set the command line arguments to the trait.
        if let Some(args) = self.args.take() {
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

        // Loop
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop
            .run(move |event, elwt| {
                match &event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::RedrawRequested => {
                            #[cfg(feature = "ui")]
                            ui.draw(&mut ui_frame, &mut ctx);

                            #[cfg(not(feature = "ui"))]
                            app.draw(&mut ui_frame, &mut ctx);

                            #[cfg(feature = "pixels_winit")]
                            ctx.gpu
                                .layer_mut(0)
                                .unwrap()
                                .frame_mut()
                                .copy_from_slice(&ui_frame);

                            #[cfg(feature = "wgpu_winit")]
                            let ui_texture = {
                                let ui_texture =
                                    ctx.gpu.load_texture(width as u32, height as u32, &ui_frame);
                                ctx.gpu
                                    .place_texture(ui_layer, ui_texture, Vec2::new(0.0, 0.0));

                                ui_texture
                            };

                            #[cfg(feature = "gpu")]
                            match ctx.gpu.draw().map_err(|e| error!("render failed: {}", e)) {
                                Ok(texture) => {
                                    if let Some(texture) = texture {
                                        app.post_captured(texture, width as u32, height as u32);
                                    }
                                }
                                Err(_) => {
                                    elwt.exit();
                                    return;
                                }
                            }

                            #[cfg(feature = "wgpu_winit")]
                            {
                                ctx.gpu.unload_texture(ui_texture);
                                if let Some(layer) = ctx.gpu.layer_mut(ui_layer) {
                                    layer.clear();
                                }
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
                                    Key::Named(NamedKey::Delete)
                                    | Key::Named(NamedKey::Backspace) => Some(TheKeyCode::Delete),
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
                        }
                        _ => (),
                    },

                    Event::DeviceEvent { event, .. } => match event {
                        // DeviceEvent::Text { codepoint } => {
                        //     println!("text: ({})", codepoint);
                        // }
                        DeviceEvent::MouseWheel { delta } => match delta {
                            MouseScrollDelta::LineDelta(x, y) => {
                                //println!("mouse wheel Line Delta: ({},{})", x, y);

                                #[cfg(feature = "ui")]
                                if ui.mouse_wheel((*x as i32, *y as i32), &mut ctx) {
                                    window.request_redraw();
                                }

                                if app.mouse_wheel((*x as isize, *y as isize), &mut ctx) {
                                    window.request_redraw();
                                    //mouse_wheel_ongoing = true;
                                }

                                if *x == 0.0 && *y == 0.0 {
                                    // mouse_wheel_ongoing = false;
                                }
                            }
                            MouseScrollDelta::PixelDelta(p) => {
                                //println!("mouse wheel Pixel Delta: ({},{})", p.x, p.y);
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
                        },
                        DeviceEvent::Added => {}
                        DeviceEvent::Removed => {}
                        DeviceEvent::MouseMotion { delta: _ } => {}
                        DeviceEvent::Motion { axis: _, value: _ } => {}
                        DeviceEvent::Button {
                            button: _,
                            state: _,
                        } => {}
                        DeviceEvent::Key(_) => {}
                    },
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
                        let scale = window.scale_factor() as f32;

                        #[cfg(feature = "gpu")]
                        ctx.gpu.resize(size.width, size.height);
                        #[cfg(feature = "gpu")]
                        ctx.gpu.set_scale_factor(scale);

                        width = (size.width as f32 / scale) as usize;
                        height = (size.height as f32 / scale) as usize;
                        ctx.width = width;
                        ctx.height = height;
                        ctx.scale_factor = scale;

                        ui_frame.resize(width * height * 4, 0);

                        #[cfg(feature = "ui")]
                        ui.canvas
                            .set_dim(TheDim::new(0, 0, width as i32, height as i32), &mut ctx);
                        #[cfg(feature = "ui")]
                        ctx.ui.send(TheEvent::Resize);

                        window.request_redraw();
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

    // Run on WASM
    #[cfg(target_arch = "wasm32")]
    pub fn run(&mut self, mut app: Box<dyn crate::TheTrait>) {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(async move {
            use log::error;
            use pixels::{Pixels, SurfaceTexture};
            use wasm_bindgen::prelude::*;
            use winit::dpi::LogicalSize;
            use winit::event::KeyboardInput;
            use winit::event::{DeviceEvent, Event, WindowEvent};
            use winit::event::{ElementState, VirtualKeyCode};
            use winit::event_loop::{ControlFlow, EventLoop};
            use winit::platform::web::WindowExtWebSys;
            use winit::window::WindowBuilder;
            use winit_input_helper::WinitInputHelper;

            let width: usize = 1200;
            let height: usize = 700;

            fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
                error!("{method_name}() failed: {err}");
            }

            let mut ctx = TheContext::new(width, height);
            #[cfg(feature = "ui")]
            let mut ui = TheUI::new();
            #[cfg(feature = "ui")]
            ui.init(&mut ctx);

            app.init(&mut ctx);

            let window_title = app.window_title();

            let event_loop = EventLoop::new();
            let window = {
                let size = LogicalSize::new(width as f64, height as f64);
                WindowBuilder::new()
                    .with_title(window_title)
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    .build(&event_loop)
                    .expect("WindowBuilder error")
            };

            // Retrieve current width and height dimensions of browser client window
            let get_window_size = || {
                let client_window = web_sys::window().unwrap();
                LogicalSize::new(
                    client_window.inner_width().unwrap().as_f64().unwrap(),
                    client_window.inner_height().unwrap().as_f64().unwrap(),
                )
            };

            let window = Rc::new(window);

            // Initialize winit window with current dimensions of browser client
            window.set_inner_size(get_window_size());

            let client_window = web_sys::window().unwrap();

            // Attach winit canvas to body element
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");

            // Listen for resize event on browser client. Adjust winit window dimensions
            // on event trigger
            {
                let window = Rc::clone(&window);
                let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    let size = get_window_size();
                    window.set_inner_size(size)
                }) as Box<dyn FnMut(_)>);
                client_window
                    .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();
            }

            let mut input = WinitInputHelper::new();
            let mut pixels = {
                let window_size = window.inner_size();
                let surface_texture =
                    SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
                Pixels::new_async(width as u32, height as u32, surface_texture)
                    .await
                    .expect("Pixels error")
            };

            #[cfg(feature = "ui")]
            {
                ui.canvas.root = true;
                ui.canvas
                    .set_dim(TheDim::new(0, 0, width as i32, height as i32), &mut ctx);

                app.init_ui(&mut ui, &mut ctx);
                ui.canvas.layout(width as i32, height as i32, &mut ctx);
            }

            event_loop.run(move |event, _, control_flow| {
                // Draw the current frame
                if let Event::RedrawRequested(_) = event {
                    #[cfg(feature = "ui")]
                    ui.draw(pixels.frame_mut(), &mut ctx);

                    #[cfg(not(feature = "ui"))]
                    app.draw(pixels.frame_mut(), &mut ctx);

                    if let Err(err) = pixels.render() {
                        log_error("pixels.render", err);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }

                match &event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::DroppedFile(path) => match path {
                            _ => {
                                app.dropped_file(path.to_str().unwrap().to_string());
                                window.request_redraw();
                            }
                        },

                        WindowEvent::ReceivedCharacter(char) => match char {
                            _ => {
                                #[cfg(feature = "ui")]
                                if !char.is_ascii_control()
                                    && ui.key_down(Some(*char), None, &mut ctx)
                                {
                                    window.request_redraw();
                                }
                                if app.key_down(Some(*char), None, &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                        },

                        WindowEvent::ModifiersChanged(state) => match state {
                            _ => {
                                #[cfg(feature = "ui")]
                                if ui.modifier_changed(
                                    state.shift(),
                                    state.ctrl(),
                                    state.alt(),
                                    state.logo(),
                                    &mut ctx,
                                ) {
                                    window.request_redraw();
                                }
                                if app.modifier_changed(
                                    state.shift(),
                                    state.ctrl(),
                                    state.alt(),
                                    state.logo(),
                                ) {
                                    window.request_redraw();
                                }
                            }
                        },

                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(virtual_code),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => match virtual_code {
                            VirtualKeyCode::Delete => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Delete), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Delete), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Back => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Delete), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Delete), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Up => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Up), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Up), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Right => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Right), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Right), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Down => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Down), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Down), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Left => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Left), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Left), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Space => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Space), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Space), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Tab => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Tab), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Tab), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Return => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Return), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Return), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Escape => {
                                #[cfg(feature = "ui")]
                                if ui.key_down(None, Some(TheKeyCode::Escape), &mut ctx) {
                                    window.request_redraw();
                                }
                                if app.key_down(None, Some(TheKeyCode::Escape), &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                            _ => (),
                        },
                        _ => (),
                    },

                    Event::DeviceEvent { event, .. } => match event {
                        // DeviceEvent::Text { codepoint } => {
                        //     println!("text: ({})", codepoint);
                        // }
                        DeviceEvent::MouseWheel { delta } => match delta {
                            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                                //println!("mouse wheel Line Delta: ({},{})", x, y);

                                #[cfg(feature = "ui")]
                                if ui.mouse_wheel((*x as i32, *y as i32), &mut ctx) {
                                    window.request_redraw();
                                }

                                if app.mouse_wheel(
                                    ((*x * 100.0) as isize, (*y * 100.0) as isize),
                                    &mut ctx,
                                ) {
                                    window.request_redraw();
                                    //mouse_wheel_ongoing = true;
                                }

                                if *x == 0.0 && *y == 0.0 {
                                    // mouse_wheel_ongoing = false;
                                }
                            }
                            winit::event::MouseScrollDelta::PixelDelta(p) => {
                                //println!("mouse wheel Pixel Delta: ({},{})", p.x, p.y);

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
                        },
                        _ => (),
                    },
                    _ => (),
                }

                // Handle input events
                if input.update(&event) {
                    // Close events
                    if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    if input.mouse_pressed(0) {
                        if let Some(coords) = input.mouse() {
                            let pixel_pos: (usize, usize) = pixels
                                .window_pos_to_pixel(coords)
                                .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                            #[cfg(feature = "ui")]
                            if ui.touch_down(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                                window.request_redraw();
                            }

                            if app.touch_down(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                                window.request_redraw();
                            }
                        }
                    }

                    if input.mouse_pressed(1) {
                        if let Some(coords) = input.mouse() {
                            let pixel_pos: (usize, usize) = pixels
                                .window_pos_to_pixel(coords)
                                .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                            #[cfg(feature = "ui")]
                            if ui.context(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                                window.request_redraw();
                            }

                            if app.touch_down(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                                window.request_redraw();
                            }
                        }
                    }

                    if input.mouse_released(0) {
                        if let Some(coords) = input.mouse() {
                            let pixel_pos: (usize, usize) = pixels
                                .window_pos_to_pixel(coords)
                                .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                            #[cfg(feature = "ui")]
                            if ui.touch_up(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                                window.request_redraw();
                            }

                            if app.touch_up(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                                window.request_redraw();
                            }
                        }
                    }

                    if input.mouse_held(0) {
                        let diff = input.mouse_diff();
                        if diff.0 != 0.0 || diff.1 != 0.0 {
                            if let Some(coords) = input.mouse() {
                                let pixel_pos: (usize, usize) = pixels
                                    .window_pos_to_pixel(coords)
                                    .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                                #[cfg(feature = "ui")]
                                if ui.touch_dragged(
                                    pixel_pos.0 as f32,
                                    pixel_pos.1 as f32,
                                    &mut ctx,
                                ) {
                                    window.request_redraw();
                                }

                                if app.touch_dragged(
                                    pixel_pos.0 as f32,
                                    pixel_pos.1 as f32,
                                    &mut ctx,
                                ) {
                                    window.request_redraw();
                                }
                            }
                        }
                    } else {
                        let diff = input.mouse_diff();
                        if diff.0 != 0.0 || diff.1 != 0.0 {
                            if let Some(coords) = input.mouse() {
                                let pixel_pos: (usize, usize) = pixels
                                    .window_pos_to_pixel(coords)
                                    .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                                #[cfg(feature = "ui")]
                                if ui.hover(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                                    window.request_redraw();
                                }

                                if app.hover(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                                    window.request_redraw();
                                }
                            }
                        }
                    }

                    // Resize the window
                    if let Some(size) = input.window_resized() {
                        if let Err(err) = pixels.resize_surface(size.width, size.height) {
                            log_error("pixels.resize_surface", err);
                            *control_flow = ControlFlow::Exit;
                            return;
                        }

                        let scale = window.scale_factor() as u32;
                        let _rc = pixels.resize_buffer(size.width / scale, size.height / scale);
                        // editor.resize(size.width as usize / scale as usize, size.height as usize / scale as usize);
                        let width = size.width as usize / scale as usize;
                        let height = size.height as usize / scale as usize;
                        ctx.width = width;
                        ctx.height = height;

                        #[cfg(feature = "ui")]
                        ui.canvas
                            .set_dim(TheDim::new(0, 0, width as i32, height as i32), &mut ctx);

                        window.request_redraw();
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

                    // Update internal state and request a redraw
                    app.update(&mut ctx);
                    window.request_redraw();
                }
            });
        });
    }
}
