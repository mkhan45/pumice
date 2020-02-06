use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::Format;
use vulkano::image::{Dimensions, StorageImage};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer, AutoCommandBuffer};

use vulkano::sync::GpuFuture;

use std::sync::Arc;

use image::{ImageBuffer, Rgba};

use vulkano::framebuffer::{Framebuffer, RenderPassAbstract, FramebufferAbstract};

use vulkano::framebuffer::Subpass;
use vulkano::pipeline::{vertex::TwoBuffersDefinition, GraphicsPipeline};

use vulkano::command_buffer::DynamicState;
use vulkano::pipeline::viewport::Viewport;

use vulkano::swapchain::{Swapchain, Surface, PresentMode, SurfaceTransform};
use vulkano::image::swapchain::SwapchainImage;

use lyon::math::Point;
use lyon::path::Path;
use lyon::tessellation::basic_shapes;
use lyon::tessellation::geometry_builder::{
    simple_builder, BuffersBuilder, FillGeometryBuilder, GeometryBuilder,
};
use lyon::tessellation::math::Rect;
use lyon::tessellation::math::Size;
use lyon::tessellation::FillTessellator;
use lyon::tessellation::{FillOptions, VertexBuffers};

use vulkano_win::VkSurfaceBuild;

use winit::EventsLoop;
use winit::WindowBuilder;
use winit::Window;

use std::rc::Rc;

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
        let instance = {
            let extensions = vulkano_win::required_extensions();
            match Instance::new(None, &extensions, None) {
                Ok(i) => i,
                Err(e) => panic!("Error creating instance {}", e),
            }
        };
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
                    khr_swapchain: true,
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

    pub fn draw(&mut self, image_num: usize, images: &Vec<Arc<SwapchainImage<Window>>>) -> AutoCommandBuffer {
        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(self.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::B8G8R8A8Srgb,
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

        let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut self.dynamic_state);
        // let framebuffer = Arc::new(
        //     Framebuffer::start(render_pass.clone())
        //     .add(images)
        //     .unwrap()
        //     .build()
        //     .unwrap(),
        // );

        // AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family())
        //     .unwrap()
        //     .begin_render_pass(
        //         framebuffers[image_num].clone(),
        //         false,
        //         vec![[0.0, 0.0, 1.0, 1.0].into()],
        //     )
        //     .unwrap()
        //     .end_render_pass()
        //     .unwrap();


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

        let clear_values = vec!([0.0, 0.0, 1.0, 1.0].into());

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
            .unwrap()
            .begin_render_pass(
                framebuffers[image_num].clone(),
                false,
                clear_values,
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
            // .copy_image_to_buffer(ima.clone(), buf.clone())
            // .unwrap()
            .build()
            .unwrap();

        // let future = previous_frame_end.join(acquire_future)
        //     .then_execute(self.queue.clone(), command_buffer).unwrap()
        //     .then_swapchain_present(self.queue.clone(), swapchain.clone(), image_num)
        //     .then_signal_fence_and_flush();

        // let finished = command_buffer.execute(self.queue.clone()).unwrap();
        // finished
        //     .then_signal_fence_and_flush()
        //     .unwrap()
        //     .wait(None)
        //     .unwrap();

        // let buffer_content = buf.read().unwrap();
        // let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
        
        command_buffer
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

    pub fn run<D>(mut self, data: &mut D, update: &dyn Fn(&mut GraphicsContext, &mut D)) {
        let mut events_loop = EventsLoop::new();
        let surface = WindowBuilder::new().build_vk_surface(&events_loop, self.instance.clone()).unwrap();

        let physical = PhysicalDevice::enumerate(&self.instance)
            .next()
            .expect("no device available");

        let caps = surface.capabilities(physical).expect("failed to get surface capabilities");
        let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        let (mut swapchain, images) = Swapchain::new(self.device.clone(), surface.clone(),
        caps.min_image_count, format, dimensions, 1, caps.supported_usage_flags, &self.queue,
        SurfaceTransform::Identity, alpha, PresentMode::Fifo, true, None)
            .expect("failed to create swapchain");

        let (mut x, mut y) = (0.0, 0.0);
        let (mut dx, mut dy) = (0.02, 0.03);

        let mut previous_frame_end = Box::new(vulkano::sync::now(self.device.clone())) as Box<dyn GpuFuture>;
        loop {
            let (image_num, acquire_future) = vulkano::swapchain::acquire_next_image(swapchain.clone(), None).unwrap();
            // self.new_circle([x, y], 0.2);

            // main loop stuff goes here
            update(&mut self, data);
            // x += dx;
            // y += dy;

            std::thread::sleep_ms(16);

            let command_buffer = self.draw(image_num, &images);

            let future = previous_frame_end.join(acquire_future)
                .then_execute(self.queue.clone(), command_buffer).unwrap()
                .then_swapchain_present(self.queue.clone(), swapchain.clone(), image_num)
                .then_signal_fence_and_flush();

            match future {
                Ok(future) => {
                    previous_frame_end = Box::new(future) as Box<_>;
                }
                Err(e) => {
                    println!("{:?}", e);
                    previous_frame_end = Box::new(vulkano::sync::now(self.device.clone())) as Box<_>;
                }
            }
            self.geometry.vertices.clear();
            self.geometry.indices.clear();

            let mut close = false;
            events_loop.poll_events(|event|{
                match event {
                    winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. } => {
                        close = true;
                    },
                    _ => {},
                };
            });

            if close {
                return;
            }
        }
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    };
    dynamic_state.viewports = Some(vec!(viewport));

    images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
            .add(image.clone()).unwrap()
            .build().unwrap()
        ) as Arc<dyn FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>()
}
