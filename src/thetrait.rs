use crate::prelude::*;

#[allow(unused)]
pub trait TheTrait {

    fn new() -> Self where Self: Sized;

    fn init(&mut self, ctx: &mut TheContext) {
    }

    fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext);

    fn update(&mut self) {
    }

    fn touch_down(&mut self, x: f32, y: f32) -> bool {
        false
    }

    fn touch_dragged(&mut self, x: f32, y: f32) -> bool {
        false
    }

    fn touch_up(&mut self, x: f32, y: f32) -> bool {
        false
    }

    fn hover(&mut self, _x: f32, _y: f32) -> bool {
        false
    }

    fn key_down(&mut self, char: Option<char>, key: Option<WidgetKey>) -> bool {
        false
    }

    fn mouse_wheel(&mut self, delta: (isize, isize)) -> bool {
        false
    }

    fn modifier_changed(&mut self, shift: bool, ctrl: bool, alt: bool, logo: bool) -> bool {
        false
    }

    fn dropped_file(&mut self, _path: String) -> bool {
        false
    }

    /// Open a file requester
    fn open(&mut self) {
    }

    /// Save the file
    fn save(&mut self) {
    }

    /// Save the file as...
    fn save_as(&mut self) {
    }

    // Cut / Copy / Paste

    fn cut(&mut self) -> String {
        "".to_string()
    }

    fn copy(&mut self) -> String {
        "".to_string()
    }

    fn paste(&mut self, text:String) {
    }

    // Undo / Redo

    fn undo(&mut self) {
    }

    fn redo(&mut self) {
    }
}