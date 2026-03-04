//! Lighting module - Phong lighting model and materials
//!
//! Provides lighting, materials, and shaders for 3D rendering.

use glam::{Vec3, Vec4};
use std::sync::Arc;

/// Light types supported in the renderer
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    /// Directional light (sun-like, parallel rays)
    Directional,
    /// Point light (omni-directional, with attenuation)
    Point,
    /// Spot light (cone-shaped)
    Spot,
    /// Ambient light (global illumination)
    Ambient,
}

#[derive(Debug, Clone, Copy)]
pub struct Light {
    /// Light type
    pub light_type: LightType,
    /// Light position (for point/spot lights)
    pub position: Vec3,
    /// Light direction (for directional/spot lights)
    pub direction: Vec3,
    /// Light color (RGB)
    pub color: Vec3,
    /// Light intensity
    pub intensity: f32,
    /// Attenuation factors (constant, linear, quadratic)
    pub attenuation: (f32, f32, f32),
    /// Spotlight angle (radians)
    pub spot_angle: f32,
    /// Spotlight penumbra (softness)
    pub spot_penumbra: f32,
    /// Enable/disable light
    pub enabled: bool,
}

impl Light {
    /// Create a new directional light
    pub fn directional(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional,
            position: Vec3::ZERO,
            direction: direction.normalize(),
            color,
            intensity,
            attenuation: (1.0, 0.0, 0.0),
            spot_angle: std::f32::consts::FRAC_PI_4,
            spot_penumbra: 0.0,
            enabled: true,
        }
    }

    /// Create a new point light
    pub fn point(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Point,
            position,
            direction: Vec3::ZERO,
            color,
            intensity,
            attenuation: (1.0, 0.09, 0.032),
            spot_angle: std::f32::consts::PI,
            spot_penumbra: 0.0,
            enabled: true,
        }
    }

    /// Create a new spot light
    pub fn spot(position: Vec3, direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Spot,
            position,
            direction: direction.normalize(),
            color,
            intensity,
            attenuation: (1.0, 0.09, 0.032),
            spot_angle: std::f32::consts::FRAC_PI_6,
            spot_penumbra: 0.1,
            enabled: true,
        }
    }

    /// Create ambient light
    pub fn ambient(color: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Ambient,
            position: Vec3::ZERO,
            direction: Vec3::ZERO,
            color,
            intensity,
            attenuation: (1.0, 0.0, 0.0),
            spot_angle: std::f32::consts::PI,
            spot_penumbra: 0.0,
            enabled: true,
        }
    }

    /// Calculate attenuation at distance
    pub fn calc_attenuation(&self, distance: f32) -> f32 {
        let (constant, linear, quadratic) = self.attenuation;
        1.0 / (constant + linear * distance + quadratic * distance * distance)
    }

    /// Calculate spotlight factor
    pub fn calc_spot_factor(&self, light_dir: Vec3) -> f32 {
        if self.light_type != LightType::Spot {
            return 1.0;
        }

        let theta = light_dir.dot(-self.direction).clamp(-1.0, 1.0).acos();
        let outer_cone = self.spot_angle;
        let inner_cone = outer_cone * (1.0 - self.spot_penumbra);

        if theta < inner_cone {
            1.0
        } else if theta < outer_cone {
            let t = (theta - inner_cone) / (outer_cone - inner_cone);
            1.0 - t * t * (3.0 - 2.0 * t)
        } else {
            0.0
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::directional(Vec3::new(0.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0), 1.0)
    }
}

/// Material type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialType {
    /// Basic flat color
    Basic,
    /// Phong shading
    Phong,
    /// Physical based rendering
    Pbr,
}

/// Material structure for surface rendering
#[derive(Debug, Clone)]
pub struct Material {
    /// Material type
    pub material_type: MaterialType,
    /// Base color (albedo)
    pub base_color: Vec4,
    /// Emissive color (self-illumination)
    pub emissive: Vec3,
    /// Ambient occlusion
    pub ambient_occlusion: f32,
    /// Roughness (0 = mirror, 1 = matte)
    pub roughness: f32,
    /// Metallic (0 = dielectric, 1 = metal)
    pub metallic: f32,
    /// Specular color
    pub specular: Vec3,
    /// Shininess (Phong exponent)
    pub shininess: f32,
    /// Reflectivity
    pub reflectivity: f32,
    /// Transparency/opacity
    pub opacity: f32,
    /// Refractive index
    pub ior: f32,
    /// Double-sided rendering
    pub double_sided: bool,
    /// Wireframe mode
    pub wireframe: bool,
}

impl Material {
    /// Create a new Phong material
    pub fn phong() -> Self {
        Self {
            material_type: MaterialType::Phong,
            base_color: Vec4::new(0.8, 0.8, 0.8, 1.0),
            emissive: Vec3::ZERO,
            ambient_occlusion: 1.0,
            roughness: 0.5,
            metallic: 0.0,
            specular: Vec3::new(1.0, 1.0, 1.0),
            shininess: 32.0,
            reflectivity: 0.5,
            opacity: 1.0,
            ior: 1.5,
            double_sided: false,
            wireframe: false,
        }
    }

    /// Create a basic flat material
    pub fn basic() -> Self {
        Self {
            material_type: MaterialType::Basic,
            base_color: Vec4::new(0.8, 0.8, 0.8, 1.0),
            emissive: Vec3::ZERO,
            ambient_occlusion: 1.0,
            roughness: 1.0,
            metallic: 0.0,
            specular: Vec3::ZERO,
            shininess: 0.0,
            reflectivity: 0.0,
            opacity: 1.0,
            ior: 1.5,
            double_sided: false,
            wireframe: false,
        }
    }

    /// Create a PBR material
    pub fn pbr() -> Self {
        Self {
            material_type: MaterialType::Pbr,
            base_color: Vec4::new(0.8, 0.8, 0.8, 1.0),
            emissive: Vec3::ZERO,
            ambient_occlusion: 1.0,
            roughness: 0.5,
            metallic: 0.0,
            specular: Vec3::new(0.04, 0.04, 0.04),
            shininess: 0.0,
            reflectivity: 0.5,
            opacity: 1.0,
            ior: 1.5,
            double_sided: false,
            wireframe: false,
        }
    }

    /// Set base color
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.base_color = color;
        self
    }

    /// Set metallic
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic.clamp(0.0, 1.0);
        self
    }

    /// Set roughness
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// Set emissive
    pub fn with_emissive(mut self, emissive: Vec3) -> Self {
        self.emissive = emissive;
        self
    }

    /// Set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set double-sided
    pub fn with_double_sided(mut self, double_sided: bool) -> Self {
        self.double_sided = double_sided;
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::phong()
    }
}

/// Light manager for scene lighting
#[derive(Debug, Clone)]
pub struct LightManager {
    /// Ambient light
    pub ambient: Light,
    /// Directional lights (up to 4)
    pub directional: [Option<Light>; 4],
    /// Point lights (up to 8)
    pub point: [Option<Light>; 8],
    /// Spot lights (up to 4)
    pub spot: [Option<Light>; 4],
    /// Number of active directional lights
    num_directional: usize,
    /// Number of active point lights
    num_point: usize,
    /// Number of active spot lights
    num_spot: usize,
}

impl LightManager {
    /// Create a new light manager
    pub fn new() -> Self {
        Self {
            ambient: Light::ambient(Vec3::new(0.2, 0.2, 0.2), 1.0),
            directional: [None, None, None, None],
            point: [None; 8],
            spot: [None; 4],
            num_directional: 0,
            num_point: 0,
            num_spot: 0,
        }
    }

    /// Create with default scene lighting
    pub fn default_scene() -> Self {
        let mut manager = Self::new();
        // Add a default directional light (sun)
        manager.add_directional(Light::directional(
            Vec3::new(-0.5, -1.0, -0.5).normalize(),
            Vec3::new(1.0, 0.98, 0.95),
            1.0,
        ));
        // Add fill light
        manager.add_directional(Light::directional(
            Vec3::new(0.5, 0.5, 0.5).normalize(),
            Vec3::new(0.4, 0.4, 0.5),
            0.5,
        ));
        manager
    }

    /// Add a directional light
    pub fn add_directional(&mut self, light: Light) -> Option<usize> {
        if light.light_type != LightType::Directional {
            return None;
        }
        if self.num_directional >= 4 {
            return None;
        }

        for (i, slot) in self.directional.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(light);
                self.num_directional += 1;
                return Some(i);
            }
        }
        None
    }

    /// Add a point light
    pub fn add_point(&mut self, light: Light) -> Option<usize> {
        if light.light_type != LightType::Point {
            return None;
        }
        if self.num_point >= 8 {
            return None;
        }

        for (i, slot) in self.point.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(light);
                self.num_point += 1;
                return Some(i);
            }
        }
        None
    }

    /// Add a spot light
    pub fn add_spot(&mut self, light: Light) -> Option<usize> {
        if light.light_type != LightType::Spot {
            return None;
        }
        if self.num_spot >= 4 {
            return None;
        }

        for (i, slot) in self.spot.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(light);
                self.num_spot += 1;
                return Some(i);
            }
        }
        None
    }

    /// Remove a directional light
    pub fn remove_directional(&mut self, index: usize) {
        if index < 4 && self.directional[index].is_some() {
            self.directional[index] = None;
            self.num_directional -= 1;
        }
    }

    /// Remove a point light
    pub fn remove_point(&mut self, index: usize) {
        if index < 8 && self.point[index].is_some() {
            self.point[index] = None;
            self.num_point -= 1;
        }
    }

    /// Remove a spot light
    pub fn remove_spot(&mut self, index: usize) {
        if index < 4 && self.spot[index].is_some() {
            self.spot[index] = None;
            self.num_spot -= 1;
        }
    }

    /// Get ambient light
    pub fn get_ambient(&self) -> &Light {
        &self.ambient
    }

    /// Set ambient light
    pub fn set_ambient(&mut self, light: Light) {
        self.ambient = light;
    }

    /// Iterate over all active lights
    pub fn iter_lights(&self) -> impl Iterator<Item = &Light> {
        self.directional
            .iter()
            .flatten()
            .chain(self.point.iter().flatten())
            .chain(self.spot.iter().flatten())
    }

    /// Get number of active lights
    pub fn num_lights(&self) -> usize {
        1 + self.num_directional + self.num_point + self.num_spot
    }

    /// Clear all lights
    pub fn clear(&mut self) {
        self.directional = [None, None, None, None];
        self.point = [None; 8];
        self.spot = [None; 4];
        self.num_directional = 0;
        self.num_point = 0;
        self.num_spot = 0;
    }
}

impl Default for LightManager {
    fn default() -> Self {
        Self::default_scene()
    }
}

/// Light uniform buffer structure for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    /// Light position (w = light type: 0=directional, 1=point, 2=spot)
    pub position: [f32; 4],
    /// Light direction (for directional/spot)
    pub direction: [f32; 4],
    /// Light color and intensity
    pub color: [f32; 4],
    /// Attenuation (constant, linear, quadratic, spot_angle)
    pub attenuation: [f32; 4],
    /// Spot penumbra, enabled flag, padding
    pub spot_penumbra: [f32; 3],
    pub enabled: u32,
}

impl LightUniform {
    /// Create from Light struct
    pub fn from_light(light: &Light) -> Self {
        let light_type = match light.light_type {
            LightType::Directional => 0.0,
            LightType::Point => 1.0,
            LightType::Spot => 2.0,
            LightType::Ambient => 3.0,
        };

        Self {
            position: [
                light.position.x,
                light.position.y,
                light.position.z,
                light_type,
            ],
            direction: [light.direction.x, light.direction.y, light.direction.z, 0.0],
            color: [
                light.color.x * light.intensity,
                light.color.y * light.intensity,
                light.color.z * light.intensity,
                light.intensity,
            ],
            attenuation: [
                light.attenuation.0,
                light.attenuation.1,
                light.attenuation.2,
                light.spot_angle,
            ],
            spot_penumbra: [light.spot_penumbra, 0.0, 0.0],
            enabled: if light.enabled { 1 } else { 0 },
        }
    }
}

/// Material uniform buffer structure for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    /// Base color (RGBA)
    pub base_color: [f32; 4],
    /// Emissive color (RGB)
    pub emissive: [f32; 3],
    pub _pad0: f32,
    /// PBR params: metallic, roughness, ambient_occlusion, opacity
    pub pbr_params: [f32; 4],
    /// Phong params: specular intensity, shininess, reflectivity, ior
    pub phong_params: [f32; 4],
    /// Flags: material_type, double_sided, wireframe, padding
    pub flags: [u32; 4],
}

impl MaterialUniform {
    /// Create from Material struct
    pub fn from_material(material: &Material) -> Self {
        let material_type = match material.material_type {
            MaterialType::Basic => 0u32,
            MaterialType::Phong => 1u32,
            MaterialType::Pbr => 2u32,
        };

        Self {
            base_color: material.base_color.to_array(),
            emissive: material.emissive.to_array(),
            _pad0: 0.0,
            pbr_params: [
                material.metallic,
                material.roughness,
                material.ambient_occlusion,
                material.opacity,
            ],
            phong_params: [
                material.specular.x * material.reflectivity,
                material.shininess,
                material.reflectivity,
                material.ior,
            ],
            flags: [
                material_type,
                if material.double_sided { 1 } else { 0 },
                if material.wireframe { 1 } else { 0 },
                0,
            ],
        }
    }
}

/// Scene ambient data
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AmbientUniform {
    /// Ambient color and intensity
    pub ambient_color: [f32; 4],
    /// Number of lights
    pub num_lights: u32,
    /// Camera position for specular
    pub camera_position: [f32; 3],
    pub _pad: f32,
}

impl AmbientUniform {
    /// Create with light manager and camera position
    pub fn new(light_manager: &LightManager, camera_pos: Vec3) -> Self {
        Self {
            ambient_color: [
                light_manager.ambient.color.x * light_manager.ambient.intensity,
                light_manager.ambient.color.y * light_manager.ambient.intensity,
                light_manager.ambient.color.z * light_manager.ambient.intensity,
                light_manager.ambient.intensity,
            ],
            num_lights: light_manager.num_lights() as u32,
            camera_position: [camera_pos.x, camera_pos.y, camera_pos.z],
            _pad: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_directional() {
        let light = Light::directional(Vec3::new(0.0, -1.0, 0.0), Vec3::new(1.0, 1.0, 1.0), 1.0);
        assert_eq!(light.light_type, LightType::Directional);
        assert!(light.direction.y < 0.0);
    }

    #[test]
    fn test_light_point() {
        let light = Light::point(Vec3::new(0.0, 5.0, 0.0), Vec3::new(1.0, 0.0, 0.0), 2.0);
        assert_eq!(light.light_type, LightType::Point);
    }

    #[test]
    fn test_light_attenuation() {
        let light = Light::point(Vec3::ZERO, Vec3::ONE, 1.0);
        assert!(light.calc_attenuation(1.0) < 1.0);
    }

    #[test]
    fn test_material() {
        let mat = Material::phong().with_color(Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(mat.material_type, MaterialType::Phong);
        assert_eq!(mat.base_color.x, 1.0);
    }

    #[test]
    fn test_light_manager() {
        let mut manager = LightManager::new();
        assert_eq!(manager.num_lights(), 1); // Ambient

        manager.add_directional(Light::directional(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::ONE,
            1.0,
        ));
        assert_eq!(manager.num_lights(), 2);
    }

    #[test]
    fn test_material_uniform() {
        let mat = Material::phong();
        let uniform = MaterialUniform::from_material(&mat);
        assert!(uniform.phong_params[1] > 0.0);
    }
}
