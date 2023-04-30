mod projection;
pub use projection::{CameraProjection, OrthographicProjection};

use glam::{Mat4, Quat, Vec2, Vec3};
use glass::winit::{
    dpi::PhysicalSize,
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
};

const Z_POS: f32 = -10.0;
pub const CAMERA_MOVE_SPEED: f32 = 250.0;

#[rustfmt::skip]
const OPENGL_TO_WGPU: glam::Mat4 = glam::Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);

/// A simple orthographic camera
#[derive(Debug, Copy, Clone)]
pub struct OrthographicCamera {
    pos: Vec2,
    ortho: OrthographicProjection,
}

impl OrthographicCamera {
    pub fn zoom(&mut self, zoom: f32) {
        self.ortho.scale *= zoom;
        self.ortho.scale = self.ortho.scale.clamp(0.15, 5.);
    }

    /// Translates camera position
    pub fn translate(&mut self, translation: Vec2) {
        self.pos += translation;
    }

    /// After window size changes, update our camera
    pub fn update(&mut self, width: f32, height: f32) {
        self.ortho.update(width, height);
    }

    /// Get world to screen matrix to be passed to our rendering
    pub fn world_to_screen(&self) -> Mat4 {
        OPENGL_TO_WGPU
            * self.ortho.get_projection_matrix()
            * Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::IDENTITY,
                self.pos.extend(Z_POS),
            )
    }

    pub fn screen_to_world_pos(
        &self,
        size: PhysicalSize<u32>,
        normalized_window_pos: Vec2,
    ) -> Vec2 {
        let (width, height) = { (size.width as f32, size.height as f32) };
        let updated_pos = Vec2::new(
            normalized_window_pos.x - width / 2.0,
            normalized_window_pos.y - height / 2.0,
        );

        // Flip y axis
        updated_pos * self.ortho.scale - self.pos * Vec2::new(1.0, -1.0)
    }

    pub fn reset_zoom(&mut self) {
        self.zoom(1.0 / self.ortho.scale);
    }

    /// Approximately zoom to fit our canvas size so that it's large enough in the beginning
    pub fn zoom_to_fit_pixels(&mut self, visible_pixels: u32, actual_pixels: u32) {
        self.reset_zoom();
        self.zoom(visible_pixels as f32 / (actual_pixels as f32))
    }
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        OrthographicCamera {
            pos: Vec2::new(0.0, 0.0),
            ortho: OrthographicProjection::default(),
        }
    }
}

pub struct CameraController {
    speed: f32,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut OrthographicCamera, size: PhysicalSize<u32>, dt: f32) {
        let (width, height) = { (size.width as f32, size.height as f32) };
        camera.update(width, height);

        // Move camera with arrows & WASD
        let up = self.is_forward_pressed;
        let down = self.is_backward_pressed;
        let left = self.is_left_pressed;
        let right = self.is_right_pressed;

        let x_axis = -(right as i8) + left as i8;
        let y_axis = -(up as i8) + down as i8;
        let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);
        if move_delta != Vec2::ZERO {
            move_delta /= move_delta.length();
            camera.translate(move_delta * dt * self.speed);
        }
    }
}
