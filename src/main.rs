mod graphics;
use graphics::GraphicsContext;
use graphics::Vertex;

use lyon::math::Point;
use lyon::tessellation::{VertexBuffers, FillOptions};
use lyon::tessellation::basic_shapes;
use lyon::tessellation::geometry_builder::simple_builder;

fn main() {
    // image.save("triangle.png").unwrap()
    
    let mut ctx = GraphicsContext::new();
    ctx.new_circle(Point::new(0.0, 0.0), 0.5);
    ctx.new_circle(Point::new(0.5, -0.5), 0.2);
    ctx.draw();
}
