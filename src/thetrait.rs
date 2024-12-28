use crate::prelude::*;

#[allow(unused)]
pub trait TheTrait {
    fn new() -> Self
    where
        Self: Sized;

    fn init(&mut self, ctx: &mut TheContext) {}

    fn default_window_size(&self) -> (usize, usize) {
        (1200, 700)
    }

    fn window_title(&self) -> String {
        "TheFramework based App".to_string()
    }

    fn window_icon(&self) -> Option<(Vec<u8>, u32, u32)> {
        None
    }

    fn set_cmd_line_args(&mut self, args: Vec<String>, ctx: &mut TheContext) {}

    #[cfg(feature = "ui")]
    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {}

    fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext) {}

    fn update(&mut self, ctx: &mut TheContext) -> bool {
        false
    }

    #[cfg(feature = "ui")]
    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        false
    }

    fn touch_down(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        false
    }

    fn touch_dragged(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        false
    }

    fn touch_up(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        false
    }

    fn hover(&mut self, _x: f32, _y: f32, ctx: &mut TheContext) -> bool {
        false
    }

    fn key_down(
        &mut self,
        char: Option<char>,
        key: Option<TheKeyCode>,
        ctx: &mut TheContext,
    ) -> bool {
        false
    }

    fn key_up(
        &mut self,
        char: Option<char>,
        key: Option<TheKeyCode>,
        ctx: &mut TheContext,
    ) -> bool {
        false
    }

    fn mouse_wheel(&mut self, delta: (isize, isize), ctx: &mut TheContext) -> bool {
        false
    }

    fn modifier_changed(&mut self, shift: bool, ctrl: bool, alt: bool, logo: bool) -> bool {
        false
    }

    fn dropped_file(&mut self, _path: String) -> bool {
        false
    }

    fn closing(&self) -> bool {
        false
    }

    // Life Circles

    #[cfg(feature = "ui")]
    fn post_ui(&mut self, ctx: &mut TheContext) {}

    #[cfg(feature = "ui")]
    fn pre_ui(&mut self, ctx: &mut TheContext) {}

    /// Open a file requester
    fn open(&mut self) {}

    /// Save the file
    fn save(&mut self) {}

    /// Save the file as...
    fn save_as(&mut self) {}

    // Cut / Copy / Paste

    fn cut(&mut self) -> String {
        "".to_string()
    }

    fn copy(&mut self) -> String {
        "".to_string()
    }

    fn paste(&mut self, text: String) {}

    // Undo / Redo

    fn undo(&mut self) {}

    fn redo(&mut self) {}
}
