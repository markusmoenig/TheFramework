use crate::{prelude::*, TheTrait};

/// TheApp class handles running an application on the current backend.
pub struct TheApp {
    #[cfg(feature = "ui")]
    pub ui: TheUI,
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
        }
    }

    /// Runs the app
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "pixels_winit")]
    pub fn run(&mut self, mut app: Box<dyn TheTrait>) -> Result<(), pixels::Error> {
        use log::error;

        use pixels::{Pixels, SurfaceTexture};
        use winit::dpi::{LogicalSize, PhysicalSize};
        use winit::event::KeyboardInput;
        use winit::event::{DeviceEvent, Event, WindowEvent};
        use winit::event_loop::{ControlFlow, EventLoop};
        use winit::window::WindowBuilder;
        use winit_input_helper::WinitInputHelper;

        let width: usize = 1200;
        let height: usize = 700;

        let event_loop = EventLoop::new();
        let mut input = WinitInputHelper::new();
        let window = {
            if cfg!(target_os = "macos") {
                let size = LogicalSize::new(width as f64, height as f64);
                WindowBuilder::new()
                    .with_title("The Framework")
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    .build(&event_loop)
                    .unwrap()
            } else {
                let size = PhysicalSize::new(width as f64, height as f64);
                WindowBuilder::new()
                    .with_title("The Framework")
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    .build(&event_loop)
                    .unwrap()
            }
        };

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(width as u32, height as u32, surface_texture)?
        };

        let mut ctx = TheContext::new(width, height);
        #[cfg(feature = "ui")]
        let mut ui = TheUI::new();
        #[cfg(feature = "ui")]
        ui.init(&mut ctx);

        app.init(&mut ctx);

        #[cfg(feature = "ui")]
        {
            ui.canvas.root = true;
            ui.canvas
                .set_dim(TheDim::new(0, 0, width as i32, height as i32), &mut ctx);

            app.init_ui(&mut ui, &mut ctx);
            ui.canvas.layout(width as i32, height as i32, &mut ctx);
        }

        // Loop
        event_loop.run(move |event, _, control_flow| {
            use winit::event::{ElementState, VirtualKeyCode};

            if let Event::RedrawRequested(_) = event {
                let frame = pixels.frame_mut();

                #[cfg(feature = "ui")]
                ui.draw(frame, &mut ctx);

                app.draw(frame, &mut ctx);
                if pixels
                    .render()
                    .map_err(|e| error!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            match &event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::DroppedFile(path) => {
                        app.dropped_file(path.to_str().unwrap().to_string());
                        window.request_redraw();
                    }

                    WindowEvent::ReceivedCharacter(char) => {
                        if app.key_down(Some(*char), None, &mut ctx) {
                            window.request_redraw();
                        }
                    }

                    WindowEvent::ModifiersChanged(state) => {
                        if app.modifier_changed(
                            state.shift(),
                            state.ctrl(),
                            state.alt(),
                            state.logo(),
                        ) {
                            window.request_redraw();
                        }
                    }

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
                            if app.key_down(None, Some(WidgetKey::Delete), &mut ctx) {
                                window.request_redraw();
                            }
                        }
                        VirtualKeyCode::Back => {
                            if app.key_down(None, Some(WidgetKey::Delete), &mut ctx) {
                                window.request_redraw();
                            }
                        }
                        VirtualKeyCode::Up => {
                            if app.key_down(None, Some(WidgetKey::Up), &mut ctx) {
                                window.request_redraw();
                            }
                        }
                        VirtualKeyCode::Right => {
                            if app.key_down(None, Some(WidgetKey::Right), &mut ctx) {
                                window.request_redraw();
                            }
                        }
                        VirtualKeyCode::Down => {
                            if app.key_down(None, Some(WidgetKey::Down), &mut ctx) {
                                window.request_redraw();
                            }
                        }
                        VirtualKeyCode::Left => {
                            if app.key_down(None, Some(WidgetKey::Left), &mut ctx) {
                                window.request_redraw();
                            }
                        }
                        VirtualKeyCode::Space => {
                            if app.key_down(None, Some(WidgetKey::Space), &mut ctx) {
                                window.request_redraw();
                            }
                        }
                        VirtualKeyCode::Tab => {
                            if app.key_down(None, Some(WidgetKey::Tab), &mut ctx) {
                                window.request_redraw();
                            }
                        }
                        VirtualKeyCode::Return => {
                            if app.key_down(None, Some(WidgetKey::Return), &mut ctx) {
                                window.request_redraw();
                            }
                        }
                        VirtualKeyCode::Escape => {
                            if app.key_down(None, Some(WidgetKey::Escape), &mut ctx) {
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
                    DeviceEvent::Text { codepoint: _ } => {}
                },
                _ => (),
            }

            // Handle input events
            if input.update(&event) {
                // Close events
                if
                /*input.key_pressed(VirtualKeyCode::Escape) ||*/
                input.close_requested() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if input.mouse_pressed(0) {
                    let coords = input.mouse().unwrap();
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

                if input.mouse_released(0) {
                    let coords = input.mouse().unwrap();
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

                if input.mouse_held(0) {
                    let diff = input.mouse_diff();
                    if diff.0 != 0.0 || diff.1 != 0.0 {
                        let coords = input.mouse().unwrap();
                        let pixel_pos: (usize, usize) = pixels
                            .window_pos_to_pixel(coords)
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        #[cfg(feature = "ui")]
                        if ui.touch_dragged(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                            window.request_redraw();
                        }

                        if app.touch_dragged(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                            window.request_redraw();
                        }
                    }
                } else {
                    let diff = input.mouse_diff();
                    if diff.0 != 0.0 || diff.1 != 0.0 {
                        let coords = input.mouse().unwrap();
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

                // Resize the window
                if let Some(size) = input.window_resized() {
                    let _rc = pixels.resize_surface(size.width, size.height);
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

                // Test if the app needs an update
                if app.update(&mut ctx) {
                    window.request_redraw();
                }
                /*
                let curr_time = get_time();

                // Game tick ?
                if curr_time > game_tick_timer + GAME_TICK_IN_MS {
                    window.request_redraw();
                    game_tick_timer = curr_time;
                    anim_counter = anim_counter.wrapping_add(1);
                } else {

                    // If not, lets see if we need to redraw for the target fps
                    let fps = creator_lib::rust_target_fps() as f32;//if mouse_wheel_ongoing { 60.0 } else { curr_screen.get_target_fps() as f32 };
                    //println!("{}", fps);
                    let tick_in_ms =  (1000.0 / fps) as u128;

                    if curr_time > timer + tick_in_ms {
                        window.request_redraw();
                        timer = curr_time;
                    } else
                    if mouse_wheel_ongoing == false {
                        let t = (timer + tick_in_ms - curr_time) as u64;
                        if t > 10 {
                            std::thread::sleep(Duration::from_millis(10));
                        }
                    }
                }*/
            }
        });
    }

    // Run on WASM
    #[cfg(target_arch = "wasm32")]
    pub fn run(&mut self, mut app: Box<dyn TheTrait>) {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(async move {
            use log::error;
            use pixels::{Pixels, SurfaceTexture};
            use std::rc::Rc;
            use winit::dpi::LogicalSize;
            use winit::event::KeyboardInput;
            use winit::event::{DeviceEvent, Event, WindowEvent};
            use winit::event::{ElementState, VirtualKeyCode};
            use winit::event_loop::{ControlFlow, EventLoop};
            use winit::window::WindowBuilder;
            use winit_input_helper::WinitInputHelper;

            const WIDTH: u32 = 800;
            const HEIGHT: u32 = 600;

            fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
                error!("{method_name}() failed: {err}");
            }

            let event_loop = EventLoop::new();
            let window = {
                let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
                WindowBuilder::new()
                    .with_title("The Framework")
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                    .build(&event_loop)
                    .expect("WindowBuilder error")
            };

            let window = Rc::new(window);

            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                use winit::platform::web::WindowExtWebSys;

                // Retrieve current width and height dimensions of browser client window
                let get_window_size = || {
                    let client_window = web_sys::window().unwrap();
                    LogicalSize::new(
                        client_window.inner_width().unwrap().as_f64().unwrap(),
                        client_window.inner_height().unwrap().as_f64().unwrap(),
                    )
                };

                let window = Rc::clone(&window);

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
                let closure =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
                        let size = get_window_size();
                        window.set_inner_size(size)
                    })
                        as Box<dyn FnMut(_)>);
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
                Pixels::new_async(WIDTH, HEIGHT, surface_texture)
                    .await
                    .expect("Pixels error")
            };

            let mut ctx = TheContext::new(WIDTH as usize, HEIGHT as usize);
            app.init(&mut ctx);

            event_loop.run(move |event, _, control_flow| {
                // Draw the current frame
                if let Event::RedrawRequested(_) = event {
                    //world.draw(pixels.frame_mut());
                    app.draw(pixels.frame_mut(), &ctx);
                    //app:draw()
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
                                if app.key_down(Some(*char), None) {
                                    window.request_redraw();
                                }
                            }
                        },

                        WindowEvent::ModifiersChanged(state) => match state {
                            _ => {
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
                                if app.key_down(None, Some(WidgetKey::Delete)) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Back => {
                                if app.key_down(None, Some(WidgetKey::Delete)) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Up => {
                                if app.key_down(None, Some(WidgetKey::Up)) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Right => {
                                if app.key_down(None, Some(WidgetKey::Right)) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Down => {
                                if app.key_down(None, Some(WidgetKey::Down)) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Left => {
                                if app.key_down(None, Some(WidgetKey::Left)) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Space => {
                                if app.key_down(None, Some(WidgetKey::Space)) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Tab => {
                                if app.key_down(None, Some(WidgetKey::Tab)) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Return => {
                                if app.key_down(None, Some(WidgetKey::Return)) {
                                    window.request_redraw();
                                }
                            }
                            VirtualKeyCode::Escape => {
                                if app.key_down(None, Some(WidgetKey::Escape)) {
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
                                if app.mouse_wheel(((*x * 100.0) as isize, (*y * 100.0) as isize)) {
                                    window.request_redraw();
                                    //mouse_wheel_ongoing = true;
                                }

                                if *x == 0.0 && *y == 0.0 {
                                    // mouse_wheel_ongoing = false;
                                }
                            }
                            winit::event::MouseScrollDelta::PixelDelta(p) => {
                                //println!("mouse wheel Pixel Delta: ({},{})", p.x, p.y);
                                if app.mouse_wheel((p.x as isize, p.y as isize)) {
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
                    if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    if input.mouse_pressed(0) {
                        let coords = input.mouse().unwrap();
                        let pixel_pos: (usize, usize) = pixels
                            .window_pos_to_pixel(coords)
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        #[cfg(feature = "ui")]
                        if ui.touch_down(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                            window.request_redraw();
                        }

                        if app.touch_down(0, pixel_pos.0 as f32, pixel_pos.1 as f32) {
                            window.request_redraw();
                        }
                    } else if input.mouse_pressed(1) {
                        let coords = input.mouse().unwrap();
                        let pixel_pos: (usize, usize) = pixels
                            .window_pos_to_pixel(coords)
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        #[cfg(feature = "ui")]
                        if ui.touch_down(pixel_pos.0 as f32, pixel_pos.1 as f32, &mut ctx) {
                            window.request_redraw();
                        }

                        if app.touch_down(1, pixel_pos.0 as f32, pixel_pos.1 as f32) {
                            window.request_redraw();
                        }
                    }

                    if input.mouse_released(0) {
                        let coords = input.mouse().unwrap();
                        let pixel_pos: (usize, usize) = pixels
                            .window_pos_to_pixel(coords)
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        if app.touch_up(pixel_pos.0 as f32, pixel_pos.1 as f32) {
                            window.request_redraw();
                        }
                    }

                    if input.mouse_held(0) {
                        let diff = input.mouse_diff();
                        if diff.0 != 0.0 || diff.1 != 0.0 {
                            let coords = input.mouse().unwrap();
                            let pixel_pos: (usize, usize) = pixels
                                .window_pos_to_pixel(coords)
                                .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                            if app.touch_dragged(pixel_pos.0 as f32, pixel_pos.1 as f32) {
                                window.request_redraw();
                            }
                        }
                    } else {
                        let diff = input.mouse_diff();
                        if diff.0 != 0.0 || diff.1 != 0.0 {
                            let coords = input.mouse().unwrap();
                            let pixel_pos: (usize, usize) = pixels
                                .window_pos_to_pixel(coords)
                                .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                            if app.hover(pixel_pos.0 as f32, pixel_pos.1 as f32) {
                                window.request_redraw();
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
                    }

                    // Update internal state and request a redraw
                    app.update();
                    window.request_redraw();
                }
            });
        });
    }
}
