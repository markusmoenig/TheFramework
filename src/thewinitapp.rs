use std::{num::NonZeroU32, sync::Arc};

use softbuffer::Surface;
use web_time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{
        DeviceEvent, DeviceId, ElementState, MouseButton, StartCause, Touch, TouchPhase,
        WindowEvent,
    },
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, ModifiersState, NamedKey},
    window::{Icon, Window, WindowAttributes, WindowId},
};
use winit_input_helper::WinitInputHelper;

use crate::prelude::*;

// Platform-aware accelerator modifiers (AltGr-safe)
#[inline]
fn is_accel_mods(m: &ModifiersState) -> bool {
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
fn accel_physical_to_ascii(code: KeyCode, shift: bool) -> Option<char> {
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

fn blit_rgba_into_softbuffer(
    ui_frame: &[u8],
    scale_factor: usize,
    width: usize,
    height: usize,
    dest: &mut [u32],
) {
    let dest_width = width * scale_factor;
    let dest_height = height * scale_factor;
    debug_assert_eq!(dest.len(), dest_width * dest_height);

    if scale_factor == 1 {
        // Direct copy without extra allocation.
        for (dst, rgba) in dest.iter_mut().zip(ui_frame.chunks_exact(4)) {
            *dst = (rgba[2] as u32) | ((rgba[1] as u32) << 8) | ((rgba[0] as u32) << 16);
        }
    } else {
        for y in 0..height {
            let src_row = &ui_frame[y * width * 4..(y + 1) * width * 4];
            for x in 0..width {
                let offset = x * 4;
                let r = src_row[offset] as u32;
                let g = src_row[offset + 1] as u32;
                let b = src_row[offset + 2] as u32;
                let color = b | (g << 8) | (r << 16);

                let dest_x = x * scale_factor;
                let dest_y = y * scale_factor;

                for sy in 0..scale_factor {
                    let row = dest_y + sy;
                    let row_start = row * dest_width + dest_x;
                    dest[row_start..row_start + scale_factor].fill(color);
                }
            }
        }
    }
}

fn translate_coord_to_local(x: f32, y: f32, scale_factor: f32) -> (f32, f32) {
    (x / scale_factor, y / scale_factor)
}

struct TheWinitContext {
    window: Arc<Window>,
    ctx: TheContext,
    ui_frame: Vec<u8>,
    surface: Surface<Arc<Window>, Arc<Window>>,
}

impl TheWinitContext {
    fn from_window(window: Arc<Window>) -> Self {
        #[cfg(not(target_os = "macos"))]
        let scale_factor = 1.0;
        // Make sure to set the initial scale factor on macOS
        #[cfg(target_os = "macos")]
        let scale_factor = window.scale_factor() as f32;

        let size = window.inner_size();
        let ctx = TheContext::new(size.width as usize, size.height as usize, scale_factor);

        let ui_frame = vec![0; (size.width * size.height * 4) as usize];

        let context = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        if let (Some(width), Some(height)) = (
            NonZeroU32::new(size.width * scale_factor as u32),
            NonZeroU32::new(size.height * scale_factor as u32),
        ) {
            surface.resize(width, height).unwrap();
        }

        TheWinitContext {
            window,
            ctx,
            ui_frame,
            surface,
        }
    }
}

struct TheWinitApp {
    args: Option<Vec<String>>,
    ctx: Option<TheWinitContext>,
    app: Box<dyn TheTrait>,

    input: WinitInputHelper,
    mods: ModifiersState,
    target_frame_time: Duration,
    next_frame_time: Instant,

    #[cfg(feature = "ui")]
    ui: TheUI,
}

impl TheWinitApp {
    fn new(args: Option<Vec<String>>, app: Box<dyn TheTrait>) -> Self {
        let fps = app.target_fps();

        TheWinitApp {
            args,
            ctx: None,
            app,
            input: WinitInputHelper::new(),
            mods: ModifiersState::empty(),
            target_frame_time: Duration::from_secs_f64(1.0 / fps),
            next_frame_time: Instant::now(),
            #[cfg(feature = "ui")]
            ui: TheUI::new(),
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Option<Arc<Window>> {
        let window_title = self.app.window_title();
        let mut icon: Option<Icon> = None;
        if let Some(window_icon) = self.app.window_icon() {
            icon = Icon::from_rgba(window_icon.0, window_icon.1, window_icon.2).ok();
        }

        let (width, height) = self.app.default_window_size();
        let size = LogicalSize::new(width as f64, height as f64);

        let window_attributes = WindowAttributes::default()
            .with_title(window_title)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_window_icon(icon); //TODO on Windows

        #[cfg(target_arch = "wasm32")]
        let window_attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;

            window_attributes.with_append(true)
        };

        let window = event_loop.create_window(window_attributes).unwrap();

        Some(Arc::new(window))
    }

    fn init_context(&mut self, window: Arc<Window>) -> TheWinitContext {
        let mut ctx = TheWinitContext::from_window(window);

        #[cfg(feature = "ui")]
        {
            self.ui.init(&mut ctx.ctx);

            self.ui.canvas.root = true;
            self.ui.canvas.set_dim(
                TheDim::new(0, 0, ctx.ctx.width as i32, ctx.ctx.height as i32),
                &mut ctx.ctx,
            );

            self.app.init_ui(&mut self.ui, &mut ctx.ctx);
            self.ui
                .canvas
                .layout(ctx.ctx.width as i32, ctx.ctx.height as i32, &mut ctx.ctx);
        }

        self.app.init(&mut ctx.ctx);

        // If available set the command line arguments to the trait.
        if let Some(args) = self.args.take() {
            self.app.set_cmd_line_args(args, &mut ctx.ctx);
        }

        ctx
    }

    fn render(&mut self) {
        let Some(ctx) = &mut self.ctx else {
            return;
        };

        if ctx.ctx.width == 0 || ctx.ctx.height == 0 {
            return;
        }

        #[cfg(feature = "ui")]
        self.app.pre_ui(&mut ctx.ctx);

        #[cfg(feature = "ui")]
        self.ui.draw(&mut ctx.ui_frame, &mut ctx.ctx);

        // We always call this for apps who use the "ui" feature
        // but do not use the UI API
        self.app.draw(&mut ctx.ui_frame, &mut ctx.ctx);

        let mut buffer = ctx.surface.buffer_mut().unwrap();
        blit_rgba_into_softbuffer(
            &ctx.ui_frame,
            ctx.ctx.scale_factor as usize,
            ctx.ctx.width,
            ctx.ctx.height,
            &mut *buffer,
        );
        buffer.present().unwrap();

        #[cfg(feature = "ui")]
        self.app.post_ui(&mut ctx.ctx);
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        let Some(ctx) = &mut self.ctx else {
            return;
        };

        if size.width != 0 && size.height != 0 {
            ctx.surface
                .resize(
                    NonZeroU32::new(size.width).unwrap(),
                    NonZeroU32::new(size.height).unwrap(),
                )
                .unwrap();

            let width = (size.width as f32 / ctx.ctx.scale_factor) as usize;
            let height = (size.height as f32 / ctx.ctx.scale_factor) as usize;
            ctx.ctx.width = width;
            ctx.ctx.height = height;

            ctx.ui_frame.resize(width * height * 4, 0);

            #[cfg(feature = "ui")]
            self.ui
                .canvas
                .set_dim(TheDim::new(0, 0, width as i32, height as i32), &mut ctx.ctx);
            #[cfg(feature = "ui")]
            ctx.ctx.ui.send(TheEvent::Resize);

            ctx.window.request_redraw();
        }
    }
}

impl ApplicationHandler for TheWinitApp {
    fn new_events(&mut self, _: &ActiveEventLoop, _: StartCause) {
        self.input.step();
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ctx.is_none() {
            if let Some(window) = self.create_window(event_loop) {
                self.ctx = Some(self.init_context(window));
            }
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if self.input.process_window_event(&event) {
            self.render();
        }

        let Some(ctx) = &mut self.ctx else {
            return;
        };

        match event {
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
                            if is_accel_mods(&self.mods) {
                                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key
                                {
                                    if let Some(ch) =
                                        accel_physical_to_ascii(code, self.mods.shift_key())
                                    {
                                        #[cfg(feature = "ui")]
                                        if self.ui.key_down(Some(ch), None, &mut ctx.ctx) {
                                            ctx.window.request_redraw();
                                        }
                                        if self.app.key_down(Some(ch), None, &mut ctx.ctx) {
                                            ctx.window.request_redraw();
                                        }
                                        return; // handled as accelerator
                                    }
                                }
                            }
                            if str.is_ascii() {
                                for ch in str.chars() {
                                    #[cfg(feature = "ui")]
                                    if self.ui.key_down(Some(ch), None, &mut ctx.ctx) {
                                        ctx.window.request_redraw();
                                    }
                                    if self.app.key_down(Some(ch), None, &mut ctx.ctx) {
                                        ctx.window.request_redraw();
                                    }
                                }
                            }
                            None
                        }
                        _ => None,
                    };
                    if key.is_some() {
                        #[cfg(feature = "ui")]
                        if self.ui.key_down(None, key.clone(), &mut ctx.ctx) {
                            ctx.window.request_redraw();
                        }
                        if self.app.key_down(None, key, &mut ctx.ctx) {
                            ctx.window.request_redraw();
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
                            if is_accel_mods(&self.mods) {
                                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key
                                {
                                    if let Some(ch) =
                                        accel_physical_to_ascii(code, self.mods.shift_key())
                                    {
                                        #[cfg(feature = "ui")]
                                        if self.ui.key_up(Some(ch), None, &mut ctx.ctx) {
                                            ctx.window.request_redraw();
                                        }
                                        if self.app.key_up(Some(ch), None, &mut ctx.ctx) {
                                            ctx.window.request_redraw();
                                        }
                                        return;
                                    }
                                }
                            }
                            if str.is_ascii() {
                                for ch in str.chars() {
                                    #[cfg(feature = "ui")]
                                    if self.ui.key_up(Some(ch), None, &mut ctx.ctx) {
                                        ctx.window.request_redraw();
                                    }
                                    if self.app.key_up(Some(ch), None, &mut ctx.ctx) {
                                        ctx.window.request_redraw();
                                    }
                                }
                            }
                            None
                        }
                        _ => None,
                    };
                    if key.is_some() {
                        #[cfg(feature = "ui")]
                        if self.ui.key_up(None, key.clone(), &mut ctx.ctx) {
                            ctx.window.request_redraw();
                        }
                        if self.app.key_up(None, key, &mut ctx.ctx) {
                            ctx.window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                // keep a copy of current modifiers for accelerator checks
                self.mods = state;

                #[cfg(feature = "ui")]
                if self.ui.modifier_changed(
                    state.shift_key(),
                    state.control_key(),
                    state.alt_key(),
                    state.super_key(),
                    &mut ctx.ctx,
                ) {
                    ctx.window.request_redraw();
                }
                if self.app.modifier_changed(
                    state.shift_key(),
                    state.control_key(),
                    state.alt_key(),
                    state.super_key(),
                ) {
                    ctx.window.request_redraw();
                }
            }
            WindowEvent::Touch(Touch {
                phase, location, ..
            }) => {
                let (x, y) = translate_coord_to_local(
                    location.x as f32,
                    location.y as f32,
                    ctx.ctx.scale_factor,
                );

                match phase {
                    TouchPhase::Started => {
                        let mut redraw = false;
                        #[cfg(feature = "ui")]
                        {
                            if self.ui.touch_down(x as f32, y as f32, &mut ctx.ctx) {
                                redraw = true;
                            }
                        }
                        if self.app.touch_down(x as f32, y as f32, &mut ctx.ctx) {
                            redraw = true;
                        }

                        if redraw {
                            ctx.window.request_redraw();
                        }
                    }
                    TouchPhase::Moved => {
                        let mut redraw = false;
                        #[cfg(feature = "ui")]
                        {
                            if self.ui.touch_dragged(x as f32, y as f32, &mut ctx.ctx) {
                                redraw = true;
                            }
                        }
                        if self.app.touch_dragged(x as f32, y as f32, &mut ctx.ctx) {
                            redraw = true;
                        }
                        if redraw {
                            ctx.window.request_redraw();
                        }
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        let mut redraw = false;
                        #[cfg(feature = "ui")]
                        {
                            if self.ui.touch_up(x as f32, y as f32, &mut ctx.ctx) {
                                redraw = true;
                            }
                        }
                        if self.app.touch_up(x as f32, y as f32, &mut ctx.ctx) {
                            redraw = true;
                        }

                        if redraw {
                            ctx.window.request_redraw();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        self.input.process_device_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.input.end_step();

        let now = Instant::now();
        if now >= self.next_frame_time {
            if let Some(ctx) = &self.ctx {
                ctx.window.request_redraw();
            }
            self.next_frame_time = now + self.target_frame_time;
        }

        #[cfg(not(target_arch = "wasm32"))]
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));

        if self.input.close_requested() {
            event_loop.exit();
            return;
        }

        if let Some(size) = self.input.window_resized() {
            self.resize(size);
        }

        let Some(ctx) = &mut self.ctx else {
            return;
        };

        if self.input.mouse_pressed(MouseButton::Left) {
            if let Some(coord) = self.input.cursor() {
                let (x, y) = translate_coord_to_local(coord.0, coord.1, ctx.ctx.scale_factor);

                #[cfg(feature = "ui")]
                if self.ui.touch_down(x as f32, y as f32, &mut ctx.ctx) {
                    ctx.window.request_redraw();
                }

                if self.app.touch_down(x as f32, y as f32, &mut ctx.ctx) {
                    ctx.window.request_redraw();
                }
            }
        }
        if self.input.mouse_pressed(MouseButton::Right) {
            if let Some(coord) = self.input.cursor() {
                let (x, y) = translate_coord_to_local(coord.0, coord.1, ctx.ctx.scale_factor);

                #[cfg(feature = "ui")]
                if self.ui.context(x as f32, y as f32, &mut ctx.ctx) {
                    ctx.window.request_redraw();
                }

                if self.app.touch_down(x as f32, y as f32, &mut ctx.ctx) {
                    ctx.window.request_redraw();
                }
            }
        }
        if self.input.mouse_released(MouseButton::Left) {
            if let Some(coord) = self.input.cursor() {
                let (x, y) = translate_coord_to_local(coord.0, coord.1, ctx.ctx.scale_factor);

                #[cfg(feature = "ui")]
                if self.ui.touch_up(x as f32, y as f32, &mut ctx.ctx) {
                    ctx.window.request_redraw();
                }

                if self.app.touch_up(x as f32, y as f32, &mut ctx.ctx) {
                    ctx.window.request_redraw();
                }
            }
        }
        if self.input.mouse_held(MouseButton::Left) {
            let diff = self.input.mouse_diff();
            if diff.0 != 0.0 || diff.1 != 0.0 {
                if let Some(coord) = self.input.cursor() {
                    let (x, y) = translate_coord_to_local(coord.0, coord.1, ctx.ctx.scale_factor);

                    #[cfg(feature = "ui")]
                    if self.ui.touch_dragged(x as f32, y as f32, &mut ctx.ctx) {
                        ctx.window.request_redraw();
                    }

                    if self.app.touch_dragged(x as f32, y as f32, &mut ctx.ctx) {
                        ctx.window.request_redraw();
                    }
                }
            }
        } else {
            let diff = self.input.mouse_diff();
            if diff.0 != 0.0 || diff.1 != 0.0 {
                if let Some(coord) = self.input.cursor() {
                    let (x, y) = translate_coord_to_local(coord.0, coord.1, ctx.ctx.scale_factor);

                    #[cfg(feature = "ui")]
                    if self.ui.hover(x as f32, y as f32, &mut ctx.ctx) {
                        ctx.window.request_redraw();
                    }

                    if self.app.hover(x as f32, y as f32, &mut ctx.ctx) {
                        ctx.window.request_redraw();
                    }
                }
            }
        }
        let (x, y) = self.input.scroll_diff();
        if x != 0.0 || y != 0.0 {
            #[cfg(feature = "ui")]
            if self.ui.mouse_wheel((x as i32, y as i32), &mut ctx.ctx) {
                ctx.window.request_redraw();
            }

            if self.app.mouse_wheel((x as isize, y as isize), &mut ctx.ctx) {
                ctx.window.request_redraw();
            }
        }

        if let Some(path) = self.input.dropped_file() {
            self.app.dropped_file(path.to_str().unwrap().to_string());
            ctx.window.request_redraw();
        }

        #[cfg(feature = "ui")]
        if self.ui.update(&mut ctx.ctx) {
            ctx.window.request_redraw();
        }

        #[cfg(feature = "ui")]
        // Test if the app needs an update
        if self.app.update_ui(&mut self.ui, &mut ctx.ctx) {
            ctx.window.request_redraw();
        }

        // Test if the app needs an update
        if self.app.update(&mut ctx.ctx) {
            ctx.window.request_redraw();
        }
    }
}

pub fn run_winit_app(args: Option<Vec<String>>, app: Box<dyn TheTrait>) {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut winit_app = TheWinitApp::new(args, app);

    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut winit_app).unwrap();
}
