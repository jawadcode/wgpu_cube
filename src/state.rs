use wgpu::{
    Backends, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, PresentMode, CompositeAlphaMode,
};
use winit::{event::WindowEvent, window::Window};

pub struct State {
    /// A handle to a surface, onto which rendered images can be presented
    surface: Surface,
    /// A handle to a graphics chip
    device: Device,
    /// Executes commands, and provides methods for writing to buffers and textures
    queue: Queue,
    /// Configures a surface for presentation
    config: SurfaceConfiguration,
    /// The size of the window in physical pixels
    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    // Create a connection to the GPU, and setup a surface
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // `instance` is a handle to the GPU
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface() };
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
            height: size.height,#
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);
        todo!()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        todo!()
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        todo!()
    }
}
