pub use winit;

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::instance::{Instance, PhysicalDevice};

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::sync::GpuFuture;

use std::sync::Arc;

use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};

use vulkano::framebuffer::Subpass;
use vulkano::pipeline::GraphicsPipeline;

use vulkano::command_buffer::DynamicState;
use vulkano::pipeline::viewport::Viewport;

use vulkano::buffer::CpuBufferPool;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::swapchain::{PresentMode, SurfaceTransform, Swapchain};

use lyon::math::Point;
use lyon::path::Path;
use lyon::tessellation::basic_shapes;
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::math::Rect;
use lyon::tessellation::math::Size;
use lyon::tessellation::BasicVertexConstructor;
use lyon::tessellation::BuffersBuilder;
use lyon::tessellation::FillTessellator;
use lyon::tessellation::{FillOptions, VertexBuffers};

use vulkano_win::VkSurfaceBuild;

use winit::EventsLoop;
use winit::Window;
use winit::WindowBuilder;

use vulkano::swapchain::AcquireError;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::sync::FlushError;

pub mod error;
pub use error::{PumiceError, PumiceResult};

#[derive(Default, Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub rot: [f32; 3], //degrees, x, y
}

vulkano::impl_vertex!(Vertex, position, color, rot);

struct WithColor([f32; 4]);

impl BasicVertexConstructor<Vertex> for WithColor {
    fn new_vertex(&mut self, position: Point) -> Vertex {
        Vertex {
            position: [position.x, position.y],
            color: self.0,
            rot: [0.0, 0.0, 0.0],
        }
    }
}

struct WithColorRotCenter([f32; 4], [f32; 3]);
impl BasicVertexConstructor<Vertex> for WithColorRotCenter {
    fn new_vertex(&mut self, position: Point) -> Vertex {
        Vertex {
            position: [position.x, position.y],
            color: self.0,
            rot: self.1,
        }
    }
}

pub struct Rotation {
    pub degrees: f32,
    pub point: [f32; 2],
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/vertex.glsl",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/fragment.glsl",
    }
}

pub struct GraphicsContext {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    dynamic_state: DynamicState,
    vertex_shader: vs::Shader,
    fragment_shader: fs::Shader,
    geometry: VertexBuffers<Vertex, u16>,
    pub surface: Arc<vulkano::swapchain::Surface<Window>>,
    events_loop: EventsLoop,
    pub screen_maxes: [f32; 2],
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

        let events_loop = EventsLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();

        GraphicsContext {
            instance,
            device,
            queue,
            dynamic_state,
            vertex_shader: vs,
            fragment_shader: fs,
            geometry: VertexBuffers::new(),
            surface,
            events_loop,
            screen_maxes: [1.0, 1.0],
        }
    }

    pub fn new_circle(
        &mut self,
        pos: impl Into<Point>,
        rad: f32,
        color: [f32; 4],
    ) -> PumiceResult<()> {
        let options = FillOptions::tolerance(0.0001);
        let mut buffer_builder = BuffersBuilder::new(&mut self.geometry, WithColor(color));
        match basic_shapes::fill_circle(pos.into(), rad, &options, &mut buffer_builder) {
            Ok(_) => Ok(()),
            Err(tesselate_error) => Err(PumiceError::from(tesselate_error)),
        }
    }

    pub fn new_rectangle(
        &mut self,
        pos: impl Into<Point>,
        sides: impl Into<Size>,
        color: [f32; 4],
    ) -> PumiceResult<()> {
        self.new_rectangle_full(pos, sides, color, None)
    }

    pub fn new_rectangle_rotcenter(
        &mut self,
        pos: impl Into<Point>,
        sides: impl Into<Size>,
        rot: f32,
        color: [f32; 4],
    ) -> PumiceResult<()> {
        let point = pos.into();
        let sides = sides.into();
        self.new_rectangle_full(
            point,
            sides,
            color,
            Some(Rotation {
                degrees: rot,
                point: [point.x + sides.width / 2.0, point.y + sides.height / 2.0],
            }),
        )
    }

    pub fn new_rectangle_full(
        &mut self,
        pos: impl Into<Point>,
        sides: impl Into<Size>,
        color: [f32; 4],
        rot: Option<Rotation>, //degrees
    ) -> PumiceResult<()> {
        let options = FillOptions::non_zero();
        let rect = Rect::new(pos.into(), sides.into());
        if let Some(rot) = rot {
            let mut buffer_builder = BuffersBuilder::new(
                &mut self.geometry,
                WithColorRotCenter(color, [rot.degrees, rot.point[0], rot.point[1]]),
            );
            match basic_shapes::fill_rectangle(&rect, &options, &mut buffer_builder) {
                Ok(_) => Ok(()),
                Err(tesselate_error) => Err(PumiceError::from(tesselate_error)),
            }
        } else {
            let mut buffer_builder = BuffersBuilder::new(&mut self.geometry, WithColor(color));
            match basic_shapes::fill_rectangle(&rect, &options, &mut buffer_builder) {
                Ok(_) => Ok(()),
                Err(tesselate_error) => Err(PumiceError::from(tesselate_error)),
            }
        }
    }

    pub fn new_quad(
        &mut self,
        points: [impl Into<Point> + Copy; 4],
        color: [f32; 4],
    ) -> PumiceResult<()> {
        let options = FillOptions::non_zero();
        let mut buffer_builder = BuffersBuilder::new(&mut self.geometry, WithColor(color));
        let result = basic_shapes::fill_quad(
            points[0].into(),
            points[1].into(),
            points[2].into(),
            points[3].into(),
            &options,
            &mut buffer_builder,
        );
        match result {
            Ok(_) => Ok(()),
            Err(tesselate_error) => Err(PumiceError::from(tesselate_error)),
        }
    }

    // pub fn new_triangle(&mut self, points: [impl Into<Point> + Copy; 3], color: [f32; 4]) {
    //     let options = FillOptions::default();
    //     let mut buffer_builder = BuffersBuilder::new(&mut self.geometry, WithColor(color));

    //     let mut path_builder = Path::builder();
    //     path_builder.move_to(points[0].into());
    //     path_builder.line_to(points[1].into());
    //     path_builder.line_to(points[2].into());
    //     path_builder.close();

    //     let path = path_builder.build();

    //     let mut tesselator = FillTessellator::new();
    //     tesselator.tessellate_with_ids(
    //         path.id_iter(),
    //         &path,
    //         Some(&path),
    //         &options,
    //         &mut buffer_builder,
    //     );
    // }

    pub fn run<D>(
        mut self,
        data: &mut D,
        update: &dyn Fn(&mut GraphicsContext, &mut D) -> PumiceResult<()>,
        handle_event: &dyn Fn(&winit::Event, &mut D) -> PumiceResult<()>,
        clear_color: [f32; 4],
    ) {
        let physical = PhysicalDevice::enumerate(&self.instance)
            .next()
            .expect("no device available");

        let caps = self
            .surface
            .capabilities(physical)
            .expect("failed to get surface capabilities");
        let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        let (mut swapchain, images) = Swapchain::new(
            self.device.clone(),
            self.surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            caps.supported_usage_flags,
            &self.queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None,
        )
        .expect("failed to create swapchain");

        let mut previous_frame_end =
            Box::new(vulkano::sync::now(self.device.clone())) as Box<dyn GpuFuture>;

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(self.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
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

        let mut framebuffers =
            window_size_dependent_setup(&images, render_pass.clone(), &mut self.dynamic_state);

        let graphics_pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(self.vertex_shader.main_entry_point(), ())
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(self.fragment_shader.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(self.device.clone())
                .unwrap(),
        );

        let mut recreate_swapchain = false;
        let uniform_buffer =
            CpuBufferPool::<vs::ty::Data>::new(self.device.clone(), BufferUsage::all());

        let (window_size, hidpi_factor) = {
            let surface = self.surface.window();
            (surface.get_inner_size(), surface.get_hidpi_factor())
        };

        loop {
            if recreate_swapchain {
                if let Some(dimensions) = window_size {
                    let dimensions: (u32, u32) = dimensions.to_physical(hidpi_factor).into();
                    let dimensions = [dimensions.0, dimensions.1];

                    let (new_swapchain, new_images) =
                        match swapchain.recreate_with_dimension(dimensions) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                            Err(err) => panic!("Error recreating swapchain: {:?}", err),
                        };

                    swapchain = new_swapchain;
                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut self.dynamic_state,
                    );

                    recreate_swapchain = false;
                }
            }

            let uniform_buffer_subbuffer = {
                let uniform_data = vs::ty::Data {
                    scale: [1.0 / self.screen_maxes[0], 1.0 / self.screen_maxes[1]],
                };

                uniform_buffer.next(uniform_data).unwrap()
            };

            let set = Arc::new(
                PersistentDescriptorSet::start(graphics_pipeline.clone(), 0)
                    .add_buffer(uniform_buffer_subbuffer)
                    .unwrap()
                    .build()
                    .unwrap(),
            );

            let (image_num, acquire_future) =
                match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(result) => result,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        continue;
                    }
                    Err(err) => panic!("error acquiring next image {:?}", err),
                };

            update(&mut self, data);

            let command_buffer = {
                let vertex_buffer = CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::all(),
                    self.geometry.vertices.iter().cloned(),
                )
                .unwrap();
                let index_buffer = CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::all(),
                    self.geometry.indices.iter().cloned(),
                )
                .unwrap();

                let clear_values = vec![clear_color.into()];

                AutoCommandBufferBuilder::primary_one_time_submit(
                    self.device.clone(),
                    self.queue.family(),
                )
                .unwrap()
                .begin_render_pass(framebuffers[image_num].clone(), false, clear_values)
                .unwrap()
                .draw_indexed(
                    graphics_pipeline.clone(),
                    &self.dynamic_state,
                    vertex_buffer.clone(),
                    index_buffer.clone(),
                    set.clone(),
                    (),
                )
                .unwrap()
                .end_render_pass()
                .unwrap()
                .build()
                .unwrap()
            };

            let future = previous_frame_end
                .join(acquire_future)
                .then_execute(self.queue.clone(), command_buffer)
                .unwrap()
                .then_swapchain_present(self.queue.clone(), swapchain.clone(), image_num)
                .then_signal_fence_and_flush();

            match future {
                Ok(future) => {
                    previous_frame_end = Box::new(future) as Box<_>;
                }
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                    previous_frame_end =
                        Box::new(vulkano::sync::now(self.device.clone())) as Box<_>;
                }
                Err(e) => {
                    println!("{:?}", e);
                    previous_frame_end =
                        Box::new(vulkano::sync::now(self.device.clone())) as Box<_>;
                }
            }
            self.geometry.vertices.clear();
            self.geometry.indices.clear();

            let mut close = false;
            self.events_loop.poll_events(|event| {
                handle_event(&event, data);
                match event {
                    winit::Event::WindowEvent {
                        event: winit::WindowEvent::CloseRequested,
                        ..
                    } => {
                        close = true;
                    }
                    winit::Event::WindowEvent {
                        event: winit::WindowEvent::Resized(_),
                        ..
                    } => recreate_swapchain = true,
                    _ => {}
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
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
