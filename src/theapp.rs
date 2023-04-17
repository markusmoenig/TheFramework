use crate::{prelude::*, TheTrait};

/// TheApp class handles running an application on the current backend.
pub struct TheApp {

}

impl TheApp {
    pub fn new() -> Self {
        Self {

        }
    }

    /// Runs the app
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "pixels_winit")]
    pub fn run(&mut self, mut app: Box<dyn TheTrait>) -> Result<(), pixels::Error> {

        use log::error;

        use pixels::{Pixels, SurfaceTexture};
        use winit::dpi::{LogicalSize, PhysicalSize};
        use winit::event::{Event, DeviceEvent, WindowEvent};
        use winit::event_loop::{ControlFlow, EventLoop};
        use winit::window::WindowBuilder;
        use winit_input_helper::WinitInputHelper;
        use winit::event::KeyboardInput;

        use std::ffi::CString;

        let mut width     : usize = 1300;
        let mut height    : usize = 700;

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
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(width as u32, height as u32, surface_texture)?
        };

        let mut ctx = TheContext::new(width, height);

        // Loop
        event_loop.run(move |event, _, control_flow| {
            use winit::event::{ElementState, VirtualKeyCode};

            if let Event::RedrawRequested(_) = event {
                // let start = get_time();

                let frame = pixels.frame_mut();
                app.draw(frame, &ctx);
                // println!("Time: {}", get_time() - start);
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

                    WindowEvent::DroppedFile(path ) => match path {
                        _ => {
                            let path = CString::new(path.to_str().unwrap()).unwrap();
                            //creator_lib::rust_dropped_file(path.as_ptr() as *const i8);
                            window.request_redraw();
                        }
                    },

                    WindowEvent::ReceivedCharacter(char ) => match char {
                        _ => {
                            let key = CString::new(char.to_string()).unwrap();
                            //if creator_lib::rust_key_down(key.as_ptr() as *const i8) {
                                window.request_redraw();
                            //}
                        }
                    },

                    WindowEvent::ModifiersChanged(state) => match state {
                        _ => {
                            // if creator_lib::rust_key_modifier_changed(state.shift(), state.ctrl(), state.alt(), state.logo()) {
                            //     window.request_redraw();
                            // }
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
                            // if creator_lib::rust_special_key_down(KEY_DELETE) {
                            //     window.request_redraw();
                            // }
                        },
                        VirtualKeyCode::Back => {
                            // if creator_lib::rust_special_key_down(KEY_DELETE) {
                            //     window.request_redraw();
                            // }
                        },
                        VirtualKeyCode::Up => {
                            // if creator_lib::rust_special_key_down(KEY_UP) {
                            //     window.request_redraw();
                            // }
                        },
                        VirtualKeyCode::Right => {
                            // if creator_lib::rust_special_key_down(KEY_RIGHT) {
                            //     window.request_redraw();
                            // }
                        },
                        VirtualKeyCode::Down => {
                            // if creator_lib::rust_special_key_down(KEY_DOWN) {
                            //     window.request_redraw();
                            // }
                        },
                        VirtualKeyCode::Left => {
                            // if creator_lib::rust_special_key_down(KEY_LEFT) {
                            //     window.request_redraw();
                            // }
                        },
                        VirtualKeyCode::Space => {
                            // if creator_lib::rust_special_key_down(KEY_SPACE) {
                            //     window.request_redraw();
                            // }
                        },
                        VirtualKeyCode::Tab => {
                            // if creator_lib::rust_special_key_down(KEY_TAB) {
                            //     window.request_redraw();
                            // }
                        },
                        VirtualKeyCode::Return => {
                            // if creator_lib::rust_special_key_down(KEY_RETURN) {
                            //     window.request_redraw();
                            // }
                        },
                        VirtualKeyCode::Escape => {
                            // if creator_lib::rust_special_key_down(KEY_ESCAPE) {
                            //     window.request_redraw();
                            // }
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
                            // if creator_lib::rust_touch_wheel(*x * 100.0, *y * 100.0) {
                            //     window.request_redraw();
                            //     mouse_wheel_ongoing = true;
                            // }

                            // if *x == 0.0 && *y == 0.0 {
                            //     mouse_wheel_ongoing = false;
                            // }
                        }
                        winit::event::MouseScrollDelta::PixelDelta(p) => {
                            //println!("mouse wheel Pixel Delta: ({},{})", p.x, p.y);
                            // if creator_lib::rust_touch_wheel(p.x as f32, p.y as f32) {
                            //     window.request_redraw();
                            //     mouse_wheel_ongoing = true;
                            // }

                            // if p.x == 0.0 && p.y == 0.0 {
                            //     mouse_wheel_ongoing = false;
                            // }
                        }
                    },
                    _ => (),
                },
                _ => (),
            }

            // Handle input events
            if input.update(&event) {
                // Close events
                if /*input.key_pressed(VirtualKeyCode::Escape) ||*/ input.close_requested() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if input.mouse_pressed(0) {
                    let coords =  input.mouse().unwrap();
                    let pixel_pos: (usize, usize) = pixels.window_pos_to_pixel(coords)
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    // if creator_lib::rust_touch_down(pixel_pos.0 as f32, pixel_pos.1 as f32) {
                    //     window.request_redraw();
                    // }
                }

                if input.mouse_released(0) {
                    let coords =  input.mouse().unwrap();
                    let pixel_pos: (usize, usize) = pixels.window_pos_to_pixel(coords)
                        .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    // if creator_lib::rust_touch_up(pixel_pos.0 as f32, pixel_pos.1 as f32) {
                    //     window.request_redraw();
                    // }
                }

                if input.mouse_held(0) {
                    let diff =  input.mouse_diff();
                    if diff.0 != 0.0 || diff.1 != 0.0 {
                        let coords =  input.mouse().unwrap();
                        let pixel_pos: (usize, usize) = pixels.window_pos_to_pixel(coords)
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        // if creator_lib::rust_touch_dragged(pixel_pos.0 as f32, pixel_pos.1 as f32) {
                        //     window.request_redraw();
                        // }
                    }
                } else {
                    let diff =  input.mouse_diff();
                    if diff.0 != 0.0 || diff.1 != 0.0 {
                        let coords =  input.mouse().unwrap();
                        let pixel_pos: (usize, usize) = pixels.window_pos_to_pixel(coords)
                            .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                        // if creator_lib::rust_hover(pixel_pos.0 as f32, pixel_pos.1 as f32) {
                        //     window.request_redraw();
                        // }
                    }
                }

                // Resize the window
                if let Some(size) = input.window_resized() {
                    let _rc = pixels.resize_surface(size.width, size.height);
                    let scale = window.scale_factor() as u32;
                    let _rc = pixels.resize_buffer(size.width / scale, size.height / scale);
                    // editor.resize(size.width as usize / scale as usize, size.height as usize / scale as usize);
                    width = size.width as usize / scale as usize;
                    height = size.height as usize / scale as usize;
                    ctx.width = width;
                    ctx.height = height;
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
            use winit::event::{Event, VirtualKeyCode};
            use winit::event_loop::{ControlFlow, EventLoop};
            use winit::window::WindowBuilder;
            use winit_input_helper::WinitInputHelper;

            const WIDTH: u32 = 800;
            const HEIGHT: u32 = 600;
            const BOX_SIZE: i16 = 64;

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
                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
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
                Pixels::new_async(WIDTH, HEIGHT, surface_texture)
                    .await
                    .expect("Pixels error")
            };
            let mut world = World::new();

            let mut ctx = TheContext::new(WIDTH as usize, HEIGHT as usize);

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

                // Handle input events
                if input.update(&event) {
                    // Close events
                    if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                        *control_flow = ControlFlow::Exit;
                        return;
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
                    world.update();
                    window.request_redraw();
                }
            });
        });
    }

}
