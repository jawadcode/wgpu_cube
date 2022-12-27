use state::State;
use wgpu::SurfaceError;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod state;
pub mod vertex;

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WGPU Cube")
        .build(&event_loop)
        .unwrap();
    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(phys_size) => state.resize(*phys_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size)
            }
            _ => (),
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                // All is well
                Ok(_) => (),
                // Reconfigure the surface if lost
                Err(SurfaceError::Lost) => state.resize(state.size),
                // The system is OOM, should probably quit lol
                Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // Any other errors should be resolved by the next frame
                Err(e) => log::error!("{e:#?}"),
            }
        }
        Event::MainEventsCleared => {
            // `RedrawRequested` will only trigger once, unless we manually request it.
            window.request_redraw();
        }
        _ => (),
    })
}
