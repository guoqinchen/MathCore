//! Camera module - 3D camera controls and transformations
//!
//! Provides camera management with orbit controls, projection, and view transformations.

use glam::{Mat4, Quat, Vec2, Vec3};
use std::f32::consts::FRAC_PI_2;

/// Camera projection type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType {
    /// Perspective projection
    Perspective,
    /// Orthographic projection
    Orthographic,
}

/// Camera projection settings
#[derive(Debug, Clone)]
pub struct Projection {
    /// Projection type
    pub proj_type: ProjectionType,
    /// Field of view (radians) for perspective
    pub fov: f32,
    /// Aspect ratio (width / height)
    pub aspect: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
    /// Viewport width (for orthographic)
    pub width: f32,
    /// Viewport height (for orthographic)
    pub height: f32,
}

impl Projection {
    /// Create a new perspective projection
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            proj_type: ProjectionType::Perspective,
            fov,
            aspect,
            near,
            far,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Create a new orthographic projection
    pub fn orthographic(width: f32, height: f32, near: f32, far: f32) -> Self {
        Self {
            proj_type: ProjectionType::Orthographic,
            fov: 0.0,
            aspect: width / height,
            near,
            far,
            width,
            height,
        }
    }

    /// Update aspect ratio
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Update viewport size (for orthographic)
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.aspect = width / height;
    }

    /// Get the projection matrix
    pub fn get_projection_matrix(&self) -> Mat4 {
        match self.proj_type {
            ProjectionType::Perspective => {
                Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
            }
            ProjectionType::Orthographic => {
                let left = -self.width / 2.0;
                let right = self.width / 2.0;
                let bottom = -self.height / 2.0;
                let top = self.height / 2.0;
                Mat4::orthographic_rh(left, right, bottom, top, self.near, self.far)
            }
        }
    }
}

impl Default for Projection {
    fn default() -> Self {
        Self::perspective(FRAC_PI_2, 16.0 / 9.0, 0.1, 1000.0)
    }
}

/// Orbit controls for camera manipulation
#[derive(Debug, Clone)]
pub struct OrbitControls {
    /// Target point to orbit around
    target: Vec3,
    /// Distance from target
    distance: f32,
    /// Azimuth angle (horizontal rotation)
    azimuth: f32,
    /// Polar angle (vertical rotation)
    polar: f32,
    /// Camera position (derived)
    position: Vec3,
    /// View matrix (derived)
    view: Mat4,
    /// Projection matrix (derived)
    projection: Mat4,

    /// Orbit sensitivity
    pub orbit_sensitivity: f32,
    /// Zoom sensitivity
    pub zoom_sensitivity: f32,
    /// Pan sensitivity
    pub pan_sensitivity: f32,
    /// Enable damping
    pub enable_damping: f32,
    /// Minimum distance
    pub min_distance: f32,
    /// Maximum distance
    pub max_distance: f32,
    /// Minimum polar angle
    pub min_polar: f32,
    /// Maximum polar angle
    pub max_polar: f32,
    /// Current velocity for damping
    velocity_azimuth: f32,
    velocity_polar: f32,
    velocity_distance: f32,
    velocity_pan: Vec3,
    /// Dirty flag
    dirty: bool,
}

impl OrbitControls {
    /// Create new orbit controls
    pub fn new() -> Self {
        let mut controls = Self {
            target: Vec3::ZERO,
            distance: 10.0,
            azimuth: 0.0,
            polar: FRAC_PI_2,
            position: Vec3::new(0.0, 0.0, 10.0),
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
            orbit_sensitivity: 0.005,
            zoom_sensitivity: 0.1,
            pan_sensitivity: 0.01,
            enable_damping: 0.9,
            min_distance: 0.1,
            max_distance: 1000.0,
            min_polar: 0.01,
            max_polar: FRAC_PI_2 - 0.01,
            velocity_azimuth: 0.0,
            velocity_polar: 0.0,
            velocity_distance: 0.0,
            velocity_pan: Vec3::ZERO,
            dirty: true,
        };
        controls.update_position();
        controls
    }

    /// Create with target and distance
    pub fn with_target(target: Vec3, distance: f32) -> Self {
        let mut controls = Self::new();
        controls.target = target;
        controls.distance = distance;
        controls.update_position();
        controls
    }

    /// Set the target point
    pub fn set_target(&mut self, target: Vec3) {
        self.target = target;
        self.dirty = true;
    }

    /// Get the target point
    pub fn target(&self) -> Vec3 {
        self.target
    }

    /// Set distance
    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance.clamp(self.min_distance, self.max_distance);
        self.dirty = true;
    }

    /// Get distance
    pub fn distance(&self) -> f32 {
        self.distance
    }

    /// Set azimuth angle
    pub fn set_azimuth(&mut self, azimuth: f32) {
        self.azimuth = azimuth;
        self.dirty = true;
    }

    /// Set polar angle
    pub fn set_polar(&mut self, polar: f32) {
        self.polar = polar.clamp(self.min_polar, self.max_polar);
        self.dirty = true;
    }

    /// Handle mouse orbit rotation
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        self.velocity_azimuth -= delta_x * self.orbit_sensitivity;
        self.velocity_polar += delta_y * self.orbit_sensitivity;
    }

    /// Handle mouse wheel zoom
    pub fn zoom(&mut self, delta: f32) {
        self.velocity_distance += delta * self.zoom_sensitivity * self.distance;
    }

    /// Handle middle mouse pan
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        // Calculate right and up vectors in world space
        let right = self.view.x_axis.truncate();
        let up = self.view.y_axis.truncate();

        // Scale by distance for consistent pan speed
        let scale = self.distance * self.pan_sensitivity;

        self.velocity_pan -= right * delta_x * scale;
        self.velocity_pan += up * delta_y * scale;
    }

    /// Handle touch pinch zoom
    pub fn pinch_zoom(&mut self, delta: f32) {
        self.velocity_distance += delta * self.zoom_sensitivity * self.distance;
    }

    /// Handle touch rotate
    pub fn touch_rotate(&mut self, delta_x: f32, delta_y: f32) {
        self.orbit(delta_x, delta_y);
    }

    /// Handle touch pan
    pub fn touch_pan(&mut self, delta_x: f32, delta_y: f32) {
        self.pan(delta_x, delta_y);
    }

    /// Update camera state (call each frame)
    pub fn update(&mut self) {
        // Apply velocities with damping
        if self.enable_damping < 1.0 {
            self.azimuth += self.velocity_azimuth;
            self.polar += self.velocity_polar;
            self.distance += self.velocity_distance;
            self.target += self.velocity_pan;

            // Apply damping
            self.velocity_azimuth *= self.enable_damping;
            self.velocity_polar *= self.enable_damping;
            self.velocity_distance *= self.enable_damping;
            self.velocity_pan *= self.enable_damping;

            // Stop if velocities are very small
            if self.velocity_azimuth.abs() < 0.0001 {
                self.velocity_azimuth = 0.0;
            }
            if self.velocity_polar.abs() < 0.0001 {
                self.velocity_polar = 0.0;
            }
            if self.velocity_distance.abs() < 0.001 {
                self.velocity_distance = 0.0;
            }
            if self.velocity_pan.length() < 0.0001 {
                self.velocity_pan = Vec3::ZERO;
            }
        } else {
            // No damping
            self.azimuth += self.velocity_azimuth;
            self.polar += self.velocity_polar;
            self.distance += self.velocity_distance;
            self.target += self.velocity_pan;
        }

        // Clamp values
        self.polar = self.polar.clamp(self.min_polar, self.max_polar);
        self.distance = self.distance.clamp(self.min_distance, self.max_distance);

        // Update position
        self.update_position();
    }

    /// Update camera position from spherical coordinates
    fn update_position(&mut self) {
        let x = self.distance * self.polar.sin() * self.azimuth.sin();
        let y = self.distance * self.polar.cos();
        let z = self.distance * self.polar.sin() * self.azimuth.cos();

        self.position = self.target + Vec3::new(x, y, z);
        self.view = Mat4::look_at_rh(self.position, self.target, Vec3::Y);
        self.dirty = false;
    }

    /// Get camera position
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Get view matrix
    pub fn view_matrix(&self) -> Mat4 {
        self.view
    }

    /// Get projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        self.projection
    }

    /// Get combined view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection * self.view
    }

    /// Check if camera needs update
    pub fn is_dirty(&self) -> bool {
        self.dirty
            || self.velocity_azimuth != 0.0
            || self.velocity_polar != 0.0
            || self.velocity_distance != 0.0
            || self.velocity_pan != Vec3::ZERO
    }

    /// Force update
    pub fn update_forced(&mut self) {
        self.velocity_azimuth = 0.0;
        self.velocity_polar = 0.0;
        self.velocity_distance = 0.0;
        self.velocity_pan = Vec3::ZERO;
        self.update_position();
    }

    /// Reset to default view
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for OrbitControls {
    fn default() -> Self {
        Self::new()
    }
}

/// Camera combining orbit controls and projection
#[derive(Debug, Clone)]
pub struct Camera {
    pub controls: OrbitControls,
    pub projection: Projection,
}

impl Camera {
    /// Create a new camera
    pub fn new() -> Self {
        Self {
            controls: OrbitControls::new(),
            projection: Projection::default(),
        }
    }

    /// Create with custom projection
    pub fn with_projection(projection: Projection) -> Self {
        Self {
            controls: OrbitControls::new(),
            projection,
        }
    }

    /// Update (call each frame)
    pub fn update(&mut self) {
        self.controls.update();
        self.projection.proj_type = ProjectionType::Perspective;
    }

    /// Get view matrix
    pub fn view_matrix(&self) -> Mat4 {
        self.controls.view_matrix()
    }

    /// Get projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        self.projection.get_projection_matrix()
    }

    /// Get view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection.get_projection_matrix() * self.controls.view_matrix()
    }

    /// Get camera position
    pub fn position(&self) -> Vec3 {
        self.controls.position()
    }

    /// Get camera target
    pub fn target(&self) -> Vec3 {
        self.controls.target()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// Mouse interaction state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    None,
}

/// Mouse state for camera control
#[derive(Debug, Clone)]
pub struct MouseState {
    /// Current mouse position
    pub position: Vec2,
    /// Previous mouse position
    pub prev_position: Vec2,
    /// Delta movement
    pub delta: Vec2,
    /// Current button state
    pub button: MouseButton,
    /// Scroll delta
    pub scroll_delta: f32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            prev_position: Vec2::ZERO,
            delta: Vec2::ZERO,
            button: MouseButton::None,
            scroll_delta: 0.0,
        }
    }

    /// Update state for next frame
    pub fn update(&mut self) {
        self.delta = self.position - self.prev_position;
        self.prev_position = self.position;
        self.scroll_delta = 0.0;
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

/// Touch state for camera control
#[derive(Debug, Clone)]
pub struct TouchState {
    /// Primary touch position
    pub position: Vec2,
    /// Previous touch position
    pub prev_position: Vec2,
    /// Secondary touch position (for pinch)
    pub position2: Vec2,
    /// Previous secondary touch
    pub prev_position2: Vec2,
    /// Delta movement
    pub delta: Vec2,
    /// Previous distance between touches
    pub prev_distance: f32,
    /// Current distance between touches
    pub distance: f32,
    /// Pinch delta
    pub pinch_delta: f32,
    /// Number of active touches
    pub touch_count: usize,
}

impl TouchState {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            prev_position: Vec2::ZERO,
            position2: Vec2::ZERO,
            prev_position2: Vec2::ZERO,
            delta: Vec2::ZERO,
            prev_distance: 0.0,
            distance: 0.0,
            pinch_delta: 0.0,
            touch_count: 0,
        }
    }

    /// Update touch positions and calculate deltas
    pub fn update(&mut self) {
        self.delta = self.position - self.prev_position;
        self.prev_position = self.position;
        self.prev_position2 = self.position2;
        self.prev_distance = self.distance;
        self.pinch_delta = 0.0;
    }

    /// Calculate distance between two touches
    pub fn calc_distance(&self) -> f32 {
        (self.position2 - self.position).length()
    }
}

impl Default for TouchState {
    fn default() -> Self {
        Self::new()
    }
}

/// Input handler combining mouse and touch
#[derive(Debug, Clone)]
pub struct InputHandler {
    pub mouse: MouseState,
    pub touch: TouchState,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            mouse: MouseState::new(),
            touch: TouchState::new(),
        }
    }

    /// Process mouse move event
    pub fn on_mouse_move(&mut self, x: f32, y: f32) {
        self.mouse.position = Vec2::new(x, y);
    }

    /// Process mouse button event
    pub fn on_mouse_button(&mut self, button: MouseButton) {
        self.mouse.button = button;
    }

    /// Process mouse scroll event
    pub fn on_mouse_scroll(&mut self, delta: f32) {
        self.mouse.scroll_delta = delta;
    }

    /// Process touch start event
    pub fn on_touch_start(&mut self, x: f32, y: f32, touches: usize) {
        self.touch.position = Vec2::new(x, y);
        self.touch.prev_position = self.touch.position;
        self.touch.touch_count = touches;
        if touches >= 2 {
            self.touch.position2 = self.touch.position;
            self.touch.prev_position2 = self.touch.position2;
        }
    }

    /// Process touch move event
    pub fn on_touch_move(&mut self, x: f32, y: f32, touches: usize) {
        self.touch.prev_position = self.touch.position;
        self.touch.position = Vec2::new(x, y);
        self.touch.touch_count = touches;
    }

    /// Process touch end event
    pub fn on_touch_end(&mut self, touches: usize) {
        self.touch.touch_count = touches;
    }

    /// Update camera controls from input
    pub fn update_controls(&mut self, controls: &mut OrbitControls) {
        // Mouse controls
        match self.mouse.button {
            MouseButton::Left => {
                controls.orbit(self.mouse.delta.x, self.mouse.delta.y);
            }
            MouseButton::Middle => {
                controls.pan(self.mouse.delta.x, self.mouse.delta.y);
            }
            MouseButton::Right => {
                controls.pan(self.mouse.delta.x, self.mouse.delta.y);
            }
            MouseButton::None => {}
        }

        // Mouse scroll zoom
        if self.mouse.scroll_delta != 0.0 {
            controls.zoom(-self.mouse.scroll_delta);
        }

        // Touch controls
        if self.touch.touch_count == 1 {
            controls.touch_rotate(self.touch.delta.x, self.touch.delta.y);
        } else if self.touch.touch_count == 2 {
            controls.touch_pan(self.touch.delta.x, self.touch.delta.y);
            controls.pinch_zoom(self.touch.pinch_delta);
        }

        // Reset deltas
        self.mouse.update();
        self.touch.update();
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_perspective() {
        let proj = Projection::perspective(FRAC_PI_2, 16.0 / 9.0, 0.1, 1000.0);
        let mat = proj.get_projection_matrix();
        assert!(mat.is_finite());
    }

    #[test]
    fn test_projection_orthographic() {
        let proj = Projection::orthographic(10.0, 10.0, 0.1, 100.0);
        let mat = proj.get_projection_matrix();
        assert!(mat.is_finite());
    }

    #[test]
    fn test_orbit_controls_orbit() {
        let mut controls = OrbitControls::new();
        controls.orbit(10.0, 10.0);
        assert!(controls.is_dirty());
    }

    #[test]
    fn test_orbit_controls_zoom() {
        let mut controls = OrbitControls::new();
        let initial_dist = controls.distance();
        controls.zoom(1.0);
        controls.update();
        assert!(controls.distance() > initial_dist);
    }

    #[test]
    fn test_camera_view_projection() {
        let mut camera = Camera::new();
        camera.update();
        let vp = camera.view_projection_matrix();
        assert!(vp.is_finite());
    }
}
