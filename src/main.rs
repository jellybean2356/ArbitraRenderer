mod vertex;
mod renderer;
mod input;
mod camera;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler, 
    event::{WindowEvent, DeviceEvent, DeviceId, MouseButton, ElementState}, 
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, 
    window::{Window, WindowId, CursorGrabMode}
};

use renderer::State;

#[derive(Default)]
struct App {
    state: Option<State>,
    is_focused: bool,
    cursor_in_window: bool,
    cursor_grabbed: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                    .create_window(Window::default_attributes().with_title("Arbitra Rendering Engine"))
                    .unwrap(),
        );

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);
        self.is_focused = true;
        self.cursor_in_window = false;
        self.cursor_grabbed = false;

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Focused(focused) => {
                self.is_focused = focused;
                if !focused {
                    let _ = state.window.set_cursor_grab(CursorGrabMode::None);
                    state.window.set_cursor_visible(true);
                    self.cursor_grabbed = false;
                }
            },
            WindowEvent::CursorEntered { .. } => {
                self.cursor_in_window = true;
            },
            WindowEvent::CursorLeft { .. } => {
                self.cursor_in_window = false;
                if self.cursor_grabbed {
                    let _ = state.window.set_cursor_grab(CursorGrabMode::None);
                    state.window.set_cursor_visible(true);
                    self.cursor_grabbed = false;
                }
            },
            WindowEvent::MouseInput { state: button_state, button, .. } => {
                if button == MouseButton::Left && button_state == ElementState::Pressed && self.is_focused && self.cursor_in_window {
                    let _ = state.window.set_cursor_grab(CursorGrabMode::Confined)
                        .or_else(|_| state.window.set_cursor_grab(CursorGrabMode::Locked));
                    state.window.set_cursor_visible(false);
                    self.cursor_grabbed = true;
                }
            },
            WindowEvent::RedrawRequested => {
                state.render();
            }
            WindowEvent::Resized(size) =>{
                state.resize(size);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !self.cursor_grabbed {
                    return;
                }
                use winit::keyboard::PhysicalKey;
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    let is_pressed = event.state.is_pressed();
                    state.input.handle_key(key_code, is_pressed);
                    if state.input.is_escape_pressed {
                        let _ = state.window.set_cursor_grab(CursorGrabMode::None);
                        state.window.set_cursor_visible(true);
                        self.cursor_grabbed = false;
                    }
                }
            }
            _ => (),
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if !self.cursor_grabbed {
            return;
        }
        
        if let Some(state) = self.state.as_mut() {
            match event {
                DeviceEvent::MouseMotion { delta } => {
                    state.input.handle_mouse_move(delta.0 as f32, delta.1 as f32);
                }
                _ => {}
            }
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}