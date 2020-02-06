use pumice::GraphicsContext;
use pumice::winit;

const RADIUS: f32 = 0.175;

struct Data {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
}

fn update(ctx: &mut GraphicsContext, data: &mut Data) {
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

fn handle_event(event: &winit::Event, data: &mut Data) {
}

fn main() {
    let ctx = GraphicsContext::new();
    let mut data = Data {
        x: 0.0,
        y: 0.0,
        dx: 0.025,
        dy: -0.01,
    };

    ctx.run::<Data>(&mut data, &update, &handle_event);
}
