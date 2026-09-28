mod renderer;
pub mod simulation;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::Window,
};

use crate::{renderer::Renderer, simulation::Simulation};

pub struct App<const N: usize> {
    renderer: Option<Renderer<N>>,
    simulation: Simulation<N>,
    current_time: std::time::Instant,
}

impl<const N: usize> App<N> {
    pub fn new(simulation: Simulation<N>) -> Self {
        Self {
            renderer: None,
            simulation,
            current_time: std::time::Instant::now(),
        }
    }

    pub fn run(simulation: Simulation<N>) -> anyhow::Result<()> {
        env_logger::init();

        let event_loop = EventLoop::with_user_event().build()?;
        let mut app = App::<N>::new(simulation);
        event_loop.run_app(&mut app)?;

        Ok(())
    }
}

impl<const N: usize> ApplicationHandler<Renderer<N>> for App<N> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.renderer = Some(pollster::block_on(Renderer::new(window)).unwrap())
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Renderer<N>) {
        self.renderer = Some(event)
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        let dt = self.current_time.elapsed();
        self.current_time = std::time::Instant::now();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => renderer.resize(size.height, size.width),
            WindowEvent::RedrawRequested => {
                // for i in (N / 2)..N {
                //     self.simulation.height_map[i] += 0.0001; // rain
                // }
                //
                // for i in 0..N {
                //     self.simulation.height_map[i] -= 0.0001 / 2.0; // evaporation
                // }

                self.simulation.update(dt.as_secs_f32());
                renderer.update(self.simulation.height());
                match renderer.render() {
                    Ok(_) => {}
                    Err(e) => {
                        // Log the error and exit gracefully
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => renderer.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}
