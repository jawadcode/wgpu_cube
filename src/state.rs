use cgmath::Vector3;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType,
    BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor,
    CompositeAlphaMode, Device, DeviceDescriptor, Face, Features, FragmentState, FrontFace,
    IndexFormat, Instance, Limits, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
    PolygonMode, PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, SamplerBindingType, ShaderStages, Surface, SurfaceConfiguration,
    SurfaceError, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{
    camera::{Camera, CameraController, CameraUniform},
    texture::OurTexture,
    vertex::{Vertex, INDICES, VERTICES},
};

pub struct State {
    /// A handle to a surface, onto which rendered images can be presented
    pub surface: Surface,
    /// A handle to a graphics chip
    pub device: Device,
    /// Executes commands, and provides methods for writing to buffers and textures
    pub queue: Queue,
    /// Configures a surface for presentation
    pub config: SurfaceConfiguration,
    /// The size of the window in physical pixels
    pub size: PhysicalSize<u32>,
    /// A handle to a graphics rendering pipeline
    render_pipeline: RenderPipeline,

    /// A handle to a buffer of vertices
    vertex_buffer: Buffer,
    /// Indices into `vertex_buffer` which allow for deduplication of vertices
    index_buffer: Buffer,
    /// The number of indices in `index_buffer`
    num_indices: u32,
    /// All of the associated information for a `wgpu::Texture`
    _diffuse_texture: OurTexture,
    /// A group of bound resources
    diffuse_bind_group: BindGroup,

    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

impl State {
    // Create a connection to the GPU, and setup a surface
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // `instance` is a handle to the GPU
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                // Find an adapter that can present to `surface`
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        log::info!("Adapter: {:#?}", &adapter);
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    // any extra features
                    features: Features::empty(),
                    // the minimum limits for certain types of resources that our adapter should meet
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            // The method used to sync the surface with the display,
            // `PresentMode::Fifo` will cap the display rate at the display's framerate
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("plank_texture.png");
        let diffuse_texture =
            OurTexture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        // We have a bind group layout as it allows us to swap out bind groups on the fly, as long as the layout is the same
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera = Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 4.0, 6.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let camera_controller = CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update_view_proj(&camera);

        // A uniform buffer for the camera
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                // the "main function" for the vertex shader
                entry_point: "vs_main",
                // what type of vertices we want to pass to the vertex shader
                buffers: &[Vertex::desc()],
            },
            // technically optional
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                // what colour outputs wgpu should set up,
                // currently only need one for the `surface`
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                // every 3 vertices will correspond to 1 triangle
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                // how to determine if a triangle is facing forwards or not
                // in this case the triangle is facing forwards if the vertices are arranged counter-clockwise
                front_face: FrontFace::Ccw,
                // cull a triangle (don't render it) if it is facing backwards
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                // how many samples the pipeline will use
                count: 1,
                // which samples should be active, in this case, we want to use all of them
                mask: !0,
                // to do with anti-aliasing
                alpha_to_coverage_enabled: false,
            },
            // how many array layers the render attachments can have
            // we won't be rendering to array textures, hence the `None`
            multiview: None,
        });
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,
            diffuse_bind_group,
            _diffuse_texture: diffuse_texture,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // `encoder.begin_render_pass()` takes a mutable reference to `encoder`
        // which we want to drop once we're done with, hence the block expression
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // Submit the finished command buffer for execution
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
