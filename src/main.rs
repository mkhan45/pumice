mod graphics;
use graphics::GraphicsContext;

mod main_state;

extern crate winit;

const RADIUS: f32 = 0.175;

struct Data {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    frame_num: usize,
    t0: std::time::Instant,
}

fn update(ctx: &mut GraphicsContext, data: &mut Data) {
    if data.frame_num == 0 {
        data.t0 = std::time::Instant::now();
    }
    if data.frame_num % 60 == 0 {
        dbg!(data.frame_num, data.t0.elapsed().as_millis());
    }
    data.frame_num += 1;
    ctx.new_circle([data.x, data.y], RADIUS);

    data.x += data.dx;
    data.y += data.dy;

    if data.x + RADIUS >= 1.0 || data.x - RADIUS <= -1.0 {
        data.dx *= -1.0;
    }
    if data.y + RADIUS >= 1.0 || data.y - RADIUS <= -1.0 {
        data.dy *= -1.0;
    }
}

fn main() {
    let ctx = GraphicsContext::new();
    let mut data = Data{
        x: 0.0,
        y: 0.0,
        dx: 0.025,
        dy: -0.01,
        frame_num: 0,
        t0: std::time::Instant::now(),
    };

    ctx.run::<Data>(&mut data, &update);
}
