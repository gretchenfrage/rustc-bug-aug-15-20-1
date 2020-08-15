//! Graphics utilities.

use std::borrow::Cow;
use vek::*;

pub mod pod_fields;
#[macro_use]
pub mod vertex;
#[macro_use]
pub mod uniform;
pub mod mesh_diff;
pub mod buffer_vec;
pub mod texture_array;

/// Helper function. 
pub fn label(label: &str) -> Option<Cow<str>> {
    Some(Cow::Borrowed(label))
}

/// Helper function. 
pub fn cowstr(s: &str) -> Cow<str> {
    Cow::Borrowed(s)
}

/// `vec!`-like syntax for a borrowed `Cow<&[_]>`.
macro_rules! cowslice {
    ($($item:expr),* $(,)?)=>{
        Cow::Borrowed(&[
            $($item,)*
        ])
    };
}

/// Const-friendly replacement for `Mat4::new`. 
///
/// Assumes column-major. 
macro_rules! mat4 {
    (
        $m00:expr, $m01:expr, $m02:expr, $m03:expr,
        $m10:expr, $m11:expr, $m12:expr, $m13:expr, 
        $m20:expr, $m21:expr, $m22:expr, $m23:expr, 
        $m30:expr, $m31:expr, $m32:expr, $m33:expr $(,)?
    )=>{
        Mat4 {
            cols: Vec4 {
                x: Vec4 { x: $m00, y: $m01, z: $m02, w: $m03, },
                y: Vec4 { x: $m10, y: $m11, z: $m12, w: $m13, },
                z: Vec4 { x: $m20, y: $m21, z: $m22, w: $m23, },
                w: Vec4 { x: $m30, y: $m31, z: $m32, w: $m33, },
            }
        }
    };
}

/// Correction matrix. 
///
/// I don't understand why this needs to exist. 
pub const CORR: Mat4<f32> = mat4! {
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
};