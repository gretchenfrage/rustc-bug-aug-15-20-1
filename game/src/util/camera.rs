//! Camera handling. 

use super::angle::{Angle, deg};
use num_traits::real::Real;
use vek::*;

/// Yaw and pitch, which defines the orientation of a camera. 
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct YawPitch<F: Real> {
    /// Yaw angle. 
    pub yaw: Angle<F>,
    /// Pitch angle. 
    pub pitch: Angle<F>,
}

/// Defining information for a camera (location, orientation, etc). 
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera {
    /// Camera position. 
    pub pos: Vec3<f32>,
    /// Camera direction. 
    pub dir: YawPitch<f32>,
    /// Camera horizontal field of view. 
    pub fov: Angle<f32>,
    /// Camer near plane distance. 
    pub near: f32,
    /// Camera far plane distance. 
    pub far: f32,
    /// Camera aspect ratio, width / height. 
    pub aspect_ratio: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            pos: Default::default(),
            dir: Default::default(),
            fov: deg(90.0),
            near: 0.1,
            far: 100.0,
            aspect_ratio: 1.0,
        }
    }
}

impl Camera {
    /// Compute the camera's perspective projection matrix. 
    ///
    /// Left handed, zero-to-one. 
    pub fn proj(&self) -> Mat4<f32> {
        Mat4::perspective_lh_zo(
            (self.fov).clamp(deg(30.0), deg(150.0)).rad(),
            self.aspect_ratio,
            self.near,
            self.far,
        )
    }

    /// Compute the camera's view matrix. 
    pub fn view(&self) -> Mat4<f32> {
        Mat4::rotation_x(-self.dir.pitch.rad())
            * Mat4::rotation_y(-self.dir.yaw.rad())
            * Mat4::translation_3d(-self.pos)
    }
}