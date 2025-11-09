use winit::keyboard::KeyCode;

pub struct Input {
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_space_pressed: bool,
    pub is_shift_pressed: bool,
    pub is_escape_pressed: bool,

    pub mouse_delta: (f32, f32),
}

impl Input {
    pub fn new() -> Self {
        Self {
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_space_pressed: false,
            is_shift_pressed: false,
            is_escape_pressed: false,
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn handle_mouse_move(&mut self, delta_x: f32, delta_y: f32) {
        self.mouse_delta.0 += delta_x;
        self.mouse_delta.1 += delta_y;
    }

    pub fn take_mouse_delta(&mut self) -> (f32, f32) {
        let delta = self.mouse_delta;
        self.mouse_delta = (0.0, 0.0);
        delta
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }

            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_space_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                self.is_shift_pressed = is_pressed;
                true
            }
            KeyCode::Escape => {
                self.is_escape_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }
}