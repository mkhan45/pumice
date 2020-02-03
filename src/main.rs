mod graphics;
use graphics::GraphicsContext;

fn main() {
    let mut ctx = GraphicsContext::new();

    ctx.new_circle([0.0, 0.0], 0.3);
    ctx.new_circle([0.5, 0.5], 0.2);
    ctx.new_rectangle([0.4, -0.5], [0.5, 0.25]);
    ctx.new_quad([
        [-0.9, -0.9], [-0.8, -0.4], [-0.4, -0.3], [0.0, -0.6]
    ]);
    ctx.new_rectangle([-0.4, 0.5], [0.45, 0.56]);
    ctx.new_triangle([[-0.6, 0.4], [-0.6, 0.25], [-0.35, 0.4]]);
    ctx.draw("output.png".to_string());
}
