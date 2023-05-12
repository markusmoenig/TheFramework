TheFramework is an abstraction layer for your application or game. You create your app inside a trait, pass it to TheFramework and it will run on all currently supported application backends.

Basically TheFramework opens a window and provides a pixel buffer for drawing and user events (mouse, keyboard, trackpads etc). to your application trait.

[TheRenderer](https://github.com/markusmoenig/TheRenderer) is an integrated, fast and stateful rendering framework which is integrated into TheFramework.

### Current Backends

* Desktops via [pixels](https://github.com/parasyte/pixels) and [winit](https://github.com/rust-windowing/winit). This is the default backend.

* The Web, also via [pixels](https://github.com/parasyte/pixels) and [winit](https://github.com/rust-windowing/winit).

* Xcode. By compiling your app into a static library you can copy and paste it into the supplied Xcode project. This project opens a Metal surface for drawing and provides native user events from the Metal surface. This allows your app to run natively on macOS, iOS and tvOS and to deliver your app directly to the given AppStores from within Xcode.

### Example

Here is an excerpt from the provided circle [example](./examples/). It draws a circle which will get smoothly change size and color when clicked.

First you define your app trait:

```rust
pub struct Circle {
    circle_id           : u32,
}

impl TheTrait for Circle {
    fn new() -> Self where Self: Sized {
    Self {
            circle_id   : 0,
        }
    }

    /// Init the scene by adding a shape to the world space
    fn init(&mut self, ctx: &mut TheContext) {

        // The world space always has the id of 0
        if let Some(world_space) = ctx.renderer.get_space_mut(0) {
            world_space.set_coord_system(Center);
            self.circle_id = world_space.add_shape(Disc);
            world_space.set_shape_property(self.circle_id, Normal, Color, vec!(1.0, 1.0, 1.0, 1.0));
            world_space.set_shape_property(self.circle_id, Normal, Radius, vec!(100.0));
            world_space.set_shape_property(self.circle_id, Selected, Color, vec!(1.0, 0.0, 0.0, 1.0));
            world_space.set_shape_property(self.circle_id, Selected, Radius, vec!(120.0));
        }
    }

    /// Draw a circle in the middle of the window
    fn draw(&mut self, pixels: &mut [u8], ctx: &mut TheContext) {
        ctx.renderer.draw(pixels, ctx.width, ctx.height);
    }

    /// If the touch event is inside the circle, set the circle state to Selected
    fn touch_down(&mut self, x: f32, y: f32, ctx: &mut TheContext) -> bool {
        if let Some(world_space) = ctx.renderer.get_space_mut(0) {
            if let Some(shape_id) = world_space.get_shape_at(x, y) {
                world_space.set_shape_state(shape_id, Selected);
            } else {
                world_space.set_shape_state(self.circle_id, Normal);
            }
        }
        ctx.renderer.needs_update()
    }

    /// Set the circle state to Selected.
    fn touch_up(&mut self, _x: f32, _y: f32, ctx: &mut TheContext) -> bool {
        if let Some(world_space) = ctx.renderer.get_space_mut(0) {
            world_space.set_shape_state(self.circle_id, Normal);
        }
        ctx.renderer.needs_update()
    }

    /// Query if the renderer needs an update (tramsition animation ongoing etc.)
    fn needs_update(&mut self, ctx: &mut TheContext) -> bool {
        ctx.renderer.needs_update()
    }
}
```

Than in main.rs just pass the Circle struct to TheApp:

```rust
    let circle = Circle::new();
    let mut app = TheApp::new();

    _ = app.run(Box::new(circle));
```

This is all, the run function of the TheApp class will launch your application.

# Running your app

## Running on the Desktop

```bash
cargo run --release
```

Will run your app on the Desktop utilizing pixels and winit.

## Running on the Web

Install the WASM32 target:

```bash
rustup target add wasm32-unknown-unknown
```

Build the project and start a local server to host it:

```bash
cargo run-wasm --release
```

Open http://localhost:8000/ in your browser to run the example.

To build the project without serving it:

```bash
cargo run-wasm --release --build-only
```

## Building for Xcode

To build for Xcode you need to uncomment the last three lines in the Cargo.toml file of the Circle example:

```toml
[lib]
name = "rustapi"
crate-type = ["staticlib"]
```

and than build to a static lib via

```bash
cargo build --release --package circle
```

Copy the resulting librust.a lib to the Xcode/TheFramework folder, open the project in Xcode and run or deploy it.
