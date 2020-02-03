mod graphics;
use graphics::GraphicsContext;

fn main() {
    let mut ctx = GraphicsContext::new();

    let mut x = 0.0;
    let mut y = 0.0;
    let mut dx = 0.0025 * 3.;
    let mut dy = 0.0045 * 3.;

    for frame_num in 0..1200 {
        ctx.new_circle([x, y], 0.1);

        if x + 0.1 >= 1.0 || x - 0.1 <= -1.0{
            dx *= -1.0;
        }
        if y + 0.1 >= 1.0 || y - 0.1 <= -1.0 {
            dy *= -1.0;
        }
        x += dx;
        y += dy;

        ctx.draw(format!("frames/output{:04}.png", frame_num));
    }
}
