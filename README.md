# pumice ![](https://travis-ci.com/mkhan45/pumice.svg?branch=master)
A vulkano-made light and brittle game engine that rocks

## Note: Doesn't rock yet

I wanted to make a game engine and learn vulkan with `vulkano`. It might become useable to make simple games quickly, but I'm not planning on adding features beyond what makes it useable. 

## Goals:
- [X] Proper color input
- [X] Some way to handle different window sizes
- [ ] Support for custom shaders?
- [ ] Sprites

Try out the examples:
```
cargo run --example flappy
```
or
```
cargo run --example bouncy
```

## Simple Example:
```rust
use pumice::winit;
use pumice::GraphicsContext;

const RADIUS: f32 = 0.175;

// the struct that holds all the main data for the game
struct Data {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    paused: bool,
}

// the main update function that accepts an &mut GraphicsContext and Data
// drawing and updating data are both done here.
fn update(ctx: &mut GraphicsContext, data: &mut Data) {
    ctx.new_circle([data.x, data.y], RADIUS, [1.0, 0.0, 0.0, 1.0]);

    if !data.paused {
        data.x += data.dx;
        data.y += data.dy;

        if data.x + RADIUS >= 1.0 || data.x - RADIUS <= -1.0 {
            data.dx *= -1.0;
        }
        if data.y + RADIUS >= 1.0 || data.y - RADIUS <= -1.0 {
            data.dy *= -1.0;
        }
    }
}

// Right now the winit events aren't preparsed in any way but I might change that
fn handle_event(winit_event: &winit::Event, data: &mut Data) {
    match winit_event {
        winit::Event::DeviceEvent { event, .. } => match event {
            winit::DeviceEvent::Key(input) => {
                let keycode = input.virtual_keycode;
                match keycode {
                    Some(winit::VirtualKeyCode::Space) => {
                        if input.state == winit::ElementState::Pressed {
                            data.paused = !data.paused;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn main() {
    let ctx = GraphicsContext::new();
    let mut data = Data {
        x: 0.0,
        y: 0.0,
        dx: 0.025,
        dy: -0.01,
        paused: false,
    };

    // tell ctx the Data struct to use, the update function, event handling function,
    // and clear color
    ctx.run::<Data>(&mut data, &update, &handle_event, [0.0, 0.0, 0.0, 1.0]);
}

```
