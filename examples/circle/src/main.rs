use theframework::*;

pub mod circle;
use crate::circle::Circle;

fn main() {
    let circle = Circle::new();
    let mut app = TheApp::new();

    () = app.run(Box::new(circle));
}
