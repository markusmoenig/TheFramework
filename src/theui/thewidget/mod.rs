use crate::prelude::*;

#[allow(unused)]
pub trait TheWidget {
    fn new() -> Self
    where
        Self: Sized;

    fn init(&mut self, ctx: &mut TheContext) {}

    fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext) {}

    fn update(&mut self, ctx: &mut TheContext) {}

    fn needs_update(&mut self, ctx: &mut TheContext) -> bool {
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
        key: Option<WidgetKey>,
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

}
