*TheFramework is work in progress and features are incomplete and may change*

TheFramework is an abstraction layer for your application or 2D game. You create your app inside a trait, pass it to TheFramework and it will run on all currently supported application backends.

Basically TheFramework opens a window and provides a pixel buffer for drawing and user events (mouse, keyboard, trackpads etc). to your application trait.

It also provides a range of 2D drawing and utility functions for your application or game.

This enables you to focus entirely on your application without worrying about the platform specifics.

### Current Backends

* Desktops via [pixels](https://github.com/parasyte/pixels) and [winit](https://github.com/rust-windowing/winit). This is the default backend.

* The Web, also via [pixels](https://github.com/parasyte/pixels) and [winit](https://github.com/rust-windowing/winit).

* Xcode. By compiling your app into a static library you can copy and paste it into the supplied Xcode project. This project opens a Metal surface for drawing and provides native user events from the Metal surface. This allows your app to run natively on macOS, iOS and tvOS and to deliver your app directly to the given AppStores from within Xcode.

### Example

Here is an excerpt from the provided circle [example](./examples/). It draws a circle which can be modified by user input.

First you define your app trait:

```rust

struct Circle {
    radius          : usize,
}

impl TheTrait for Circle {
    fn new() -> Self where Self: Sized {
        Self {
            radius  : 100,
        }
    }

    /// Draw a circle in the middle of the window
    fn draw(&mut self, pixels: &mut [u8], ctx: &TheContext) {

        ctx.draw.circle(pixels, &(ctx.width / 2 - self.radius, ctx.height / 2 - self.radius, self.radius * 2, self.radius * 2), ctx.width, &[255, 255, 255, 255], self.radius);
    }

    /// Update the app state
    fn update(&mut self) {
    }

    // User event handling omitted, please see the example source code.
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
