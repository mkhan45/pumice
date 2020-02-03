use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::Format;
use vulkano::image::{Dimensions, StorageImage};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer};

use vulkano::sync::GpuFuture;

use std::sync::Arc;

use image::{ImageBuffer, Rgba};

use vulkano::framebuffer::Framebuffer;

use vulkano::framebuffer::Subpass;
use vulkano::pipeline::{vertex::TwoBuffersDefinition, GraphicsPipeline};

use vulkano::command_buffer::DynamicState;
use vulkano::pipeline::viewport::Viewport;

use lyon::math::Point;
use lyon::tessellation::basic_shapes;
use lyon::tessellation::FillTessellator;
use lyon::tessellation::geometry_builder::{simple_builder, GeometryBuilder, FillGeometryBuilder, BuffersBuilder};
use lyon::tessellation::math::Rect;
use lyon::tessellation::math::Size;
use lyon::tessellation::{FillOptions, VertexBuffers};
use lyon::path::Path;

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

#[derive(Default, Copy, Clone)]
struct Color {
    rgba: [f32; 4],
}

vulkano::impl_vertex!(Vertex, position);
vulkano::impl_vertex!(Color, rgba);

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/triangle.vert",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/triangle.frag",
    }
}

pub struct GraphicsContext {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    dynamic_state: DynamicState,
    vertex_shader: vs::Shader,
    fragment_shader: fs::Shader,
    geometry: VertexBuffers<Point, u16>,
}

impl GraphicsContext {
    pub fn new() -> Self {
        let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
        let physical = PhysicalDevice::enumerate(&instance)
            .next()
            .expect("no device available");

        let queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics())
            .expect("couldn't find a graphical queue family");
        let (device, mut queues) = {
            Device::new(
                physical,
                &Features::none(),
                &DeviceExtensions {
                    khr_storage_buffer_storage_class: true,
                    ..DeviceExtensions::none()
                },
                [(queue_family, 0.5)].iter().cloned(),
            )
            .expect("failed to create device")
        };

        let queue = queues.next().unwrap();

        let vs = vs::Shader::load(device.clone()).expect("failed to load vertex shader");
        let fs = fs::Shader::load(device.clone()).expect("failed to load fragment shader");

        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [1024.0, 1024.0],
                depth_range: 0.0..1.0,
            }]),
            ..DynamicState::none()
        };

        GraphicsContext {
            instance,
            device,
            queue,
            dynamic_state,
            vertex_shader: vs,
            fragment_shader: fs,
            geometry: VertexBuffers::new(),
        }
    }

    pub fn draw(&mut self) {
        let buf = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            (0..1024 * 1024 * 4).map(|_| 0u8),
        )
        .expect("failed to create buffer");

        let image = StorageImage::new(
            self.device.clone(),
            Dimensions::Dim2d {
                width: 1024,
                height: 1024,
            },
            Format::R8G8B8A8Unorm,
            Some(self.queue.family()),
        )
        .unwrap();

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(self.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::R8G8B8A8Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
            )
            .unwrap(),
        );
        let framebuffer = Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(image.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family())
            .unwrap()
            .begin_render_pass(
                framebuffer.clone(),
                false,
                vec![[0.0, 0.0, 1.0, 1.0].into()],
            )
            .unwrap()
            .end_render_pass()
            .unwrap();

        let graphics_pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                // .vertex_input(TwoBuffersDefinition::<Vertex, Color>::new())
                .vertex_shader(self.vertex_shader.main_entry_point(), ())
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(self.fragment_shader.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(self.device.clone())
                .unwrap(),
        );

        let vertex_buffer = self.geometry.vertices.iter().map(|vertex| Vertex {
            position: [vertex.x, vertex.y],
        });
        let vertex_buffer =
            CpuAccessibleBuffer::from_iter(self.device.clone(), BufferUsage::all(), vertex_buffer)
                .unwrap();
        let index_buffer = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            self.geometry.indices.iter().cloned(),
        )
        .unwrap();

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap()
        .begin_render_pass(
            framebuffer.clone(),
            false,
            vec![[0.0, 0.0, 1.0, 1.0].into()],
        )
        .unwrap()
        .draw_indexed(
            graphics_pipeline.clone(),
            &self.dynamic_state,
            vertex_buffer.clone(),
            index_buffer.clone(),
            (),
            (),
        )
        .unwrap()
        .end_render_pass()
        .unwrap()
        .copy_image_to_buffer(image.clone(), buf.clone())
        .unwrap()
        .build()
        .unwrap();

        let finished = command_buffer.execute(self.queue.clone()).unwrap();
        finished
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let buffer_content = buf.read().unwrap();
        let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
        image.save("triangle.png").unwrap();
        self.geometry.vertices.clear();
        self.geometry.indices.clear();
    }

    pub fn new_circle(&mut self, pos: impl Into<Point>, rad: f32) {
        let options = FillOptions::tolerance(0.0001);
        basic_shapes::fill_circle(
            pos.into(),
            rad,
            &options,
            &mut simple_builder(&mut self.geometry),
        );
    }

    pub fn new_rectangle(&mut self, pos: impl Into<Point>, sides: impl Into<Size>) {
        let options = FillOptions::non_zero();
        let rect = Rect::new(pos.into(), sides.into());
        basic_shapes::fill_rectangle(&rect, &options, &mut simple_builder(&mut self.geometry));
    }

    pub fn new_quad(&mut self, points: [impl Into<Point> + Copy; 4]) {
        let options = FillOptions::non_zero();
        basic_shapes::fill_quad(
            points[0].into(),
            points[1].into(),
            points[2].into(),
            points[3].into(),
            &options,
            &mut simple_builder(&mut self.geometry),
        );
    }

    pub fn new_triangle(&mut self, points: [impl Into<Point> + Copy; 3]) {
        let options = FillOptions::default();

        let mut path_builder = Path::builder();
        path_builder.move_to(points[0].into());
        path_builder.line_to(points[1].into());
        path_builder.line_to(points[2].into());
        path_builder.close();

        let path = path_builder.build();

        let mut tesselator = FillTessellator::new();
        tesselator.tessellate_with_ids(
            path.id_iter(),
            &path,
            Some(&path),
            &options,
            &mut simple_builder(&mut self.geometry),
            );
    }
}
