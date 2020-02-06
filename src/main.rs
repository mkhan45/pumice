mod graphics;
use graphics::GraphicsContext;

mod main_state;

extern crate winit;

const RADIUS: f32 = 0.175;

fn main() {
    let mut ctx = GraphicsContext::new();

    let mut x = 0.0;
    let mut y = 0.0;

    ctx.new_circle([x, y], RADIUS);
    ctx.run();
}
