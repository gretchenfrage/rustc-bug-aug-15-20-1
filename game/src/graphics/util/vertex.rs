//! Vertex utilities, especially mucking around with bytes. 

use std::{
    borrow::{Cow, BorrowMut},
    mem::size_of,
    any::{TypeId, type_name},
};
use pear::*;
use bytemuck::{Pod, bytes_of};
use wgpu::*;

/// Trait for shader vertex types. 
pub trait GenericVertex {
    /// Binary size of a vertex. 
    const SIZE: usize;

    /// Metadata about `Self`'s binary representation for GLSL.  
    fn attributes() -> Cow<'static, [VertexAttributeDescriptor]>;

    /// Write `self`'s binary representation to a byte array. 
    ///
    /// `bytes` must be `Self::SIZE` in length. 
    fn encode_to(&self, bytes: &mut [u8]);
}

/// Types that are ABI-compatible with a GLSL vertex attribute type. 
pub trait VertexAttrib: Pod {
    const COMPATIBLE_FORMATS: &'static [VertexFormat]; 
}

/// Types which can convert into an ABI-compatible GLSL vertex attribute type. 
pub trait IntoVertexAttrib {
    type Into: VertexAttrib;

    fn into_vertex_attrib(self) -> Self::Into;
}

/// Implement `GenericVertex` on a type with GLSL-like syntax. 
///
/// ```
/// use vek::*;
/// 
/// struct Example {
///     pos: Vec3<f32>,
///     tex_coord: Vec2<f32>,
///     tex_index: u32,
/// }
/// 
/// vertex! {
///     Example {
///         layout(location = 0) in vec3 pos: Vec3<f32>,
///         layout(location = 1) in vec2 tex_coord: Vec2<f32>,
///         layout(location = 2) in uint tex_index: u32,
///     }
/// }
/// ```
macro_rules! vertex {
    (
    $vertex:ident {$(
        layout(location = $location:expr) in $glsl_type:ident $rust_field:ident : $rust_type:ty,
    )*}
    )=>{
        impl $vertex {
            /// Write `self`'s binary representation to a fixed-size byte array. 
            pub fn encode(&self) -> [u8; <Self as $crate::graphics::util::vertex::GenericVertex>::SIZE] {
                let mut array = [0_u8; <Self as $crate::graphics::util::vertex::GenericVertex>::SIZE];
                <Self as $crate::graphics::util::vertex::GenericVertex>::encode_to(self, &mut array);
                array
            }
        }

        impl $crate::graphics::util::vertex::GenericVertex for $vertex {
            const SIZE: usize = {
                0 $( + 
                    std::mem::size_of::<
                        <
                            $rust_type 
                            as 
                            $crate::graphics::util::vertex::IntoVertexAttrib
                        >::Into
                    >()
                )*
            };

            fn attributes() -> std::borrow::Cow<'static, [$crate::wgpu::VertexAttributeDescriptor]> {
                use $crate::wgpu::{
                    VertexAttributeDescriptor,
                    VertexFormat,
                };

                let mut vec: Vec<VertexAttributeDescriptor> = Vec::new();
                let mut curr_offset: u64 = 0;

                $({
                    let attr_size = std::mem::size_of::<
                        <
                            $rust_type 
                            as 
                            $crate::graphics::util::vertex::IntoVertexAttrib
                        >::Into
                    >() as u64;
                    let attr_format = $crate::graphics::util::vertex::deduce_vertex_format::<
                            <
                                $rust_type 
                                as 
                                $crate::graphics::util::vertex::IntoVertexAttrib
                            >::Into
                        >(stringify!($glsl_type))
                        .unwrap_or_else(|e| {
                            error!("{}", e);
                            panic!();
                        });

                    vec.push(VertexAttributeDescriptor {
                        offset: curr_offset,
                        format: attr_format,
                        shader_location: $location,
                    });
                    curr_offset += attr_size;
                })*

                debug_assert_eq!(curr_offset, Self::SIZE as u64);
                std::borrow::Cow::Owned(vec)
            }

            fn encode_to(&self, mut bytes: &mut [u8]) {
                assert_eq!(bytes.len(), Self::SIZE, "wrong size");

                $({
                    let attr = <
                            $rust_type 
                            as 
                            $crate::graphics::util::vertex::IntoVertexAttrib
                        >::into_vertex_attrib(self.$rust_field.clone());
                    let attr_bytes = $crate::bytemuck::bytes_of(&attr);
                    for (i, b) in attr_bytes.iter().copied().enumerate() {
                        bytes[i] = b;
                    }
                    bytes = &mut bytes[attr_bytes.len()..];
                })*

                debug_assert!(bytes.is_empty());
            }
        }
    };
}

/// Determine a vertex attribute format from a rust type and a GLSL type.
pub fn deduce_vertex_format<A: VertexAttrib>(glsl_type: &str) -> Result<VertexFormat>
{
    let glsl_compat = glsl_compatible_vertex_formats(glsl_type)?;
    let rust_compat = A::COMPATIBLE_FORMATS;

    let mut iter = glsl_compat.iter().copied()
        .filter(|format| rust_compat.contains(format));
    let format = iter.next()
        .ok_or_else(|| pear!(
            {glsl_type=glsl_type, rust_type=type_name::<A>()},
            "no reasonable VertexFormat found",
        ))?;
    if iter.next().is_some() {
        return Err(pear!(
            {glsl_type=glsl_type, rust_type=type_name::<A>()},
            "more than one reasonable VertexFormat found",
        ));
    }

    Ok(format)
}

macro_rules! define_glsl_to_vertex {
    ($(
        $type:literal => [$($format:ident),* $(,)?],
    )*)=>{
        /// Lookup from GLSL types (eg. `vec2`) to compatible vertex attribute formats. 
        pub fn glsl_compatible_vertex_formats(glsl: &str) -> Result<&'static [VertexFormat]> {
            match glsl {
                $(
                $type => Ok(&[$(
                    VertexFormat::$format,
                )*]),
                )*
                _ => Err(pear!({glsl_type=glsl}, "invalid GLSL type")),
            }
        }
    };
}

define_glsl_to_vertex! {
    "uint" => [Uint],
    "uvec2" => [Uchar2, Ushort2, Uint2],
    "uvec3" => [Uint3],
    "uvec4" => [Uchar4, Ushort4, Uint4],

    "int" => [Int],
    "ivec2" => [Char2, Short2, Int2],
    "ivec3" => [Int3],
    "ivec4" => [Char4, Short4, Int4],

    "float" => [Float],
    "vec2" => [Uchar2Norm, Char2Norm, Ushort2Norm, Short2Norm, Half2, Float2],
    "vec3" => [Float3],
    "vec4" => [Uchar4Norm, Char4Norm, Ushort4Norm, Short4Norm, Half4, Float4],
}

macro_rules! vertex_attrib {
    ($(
        $type:ty = [$($format:ident),* $(,)?];
    )*)=>{
        $(
        impl VertexAttrib for $type {
            const COMPATIBLE_FORMATS: &'static [VertexFormat] = &[
                $(
                    VertexFormat::$format,
                )*
            ];
        }

        impl IntoVertexAttrib for $type {
            type Into = Self;

            fn into_vertex_attrib(self) -> Self { self }
        }
        )*
    };
}

vertex_attrib! {
    [u8; 2] = [Uchar2, Uchar2Norm];
    [u8; 4] = [Uchar4, Uchar4Norm];
    [i8; 2] = [Char2, Char2Norm];
    [i8; 4] = [Char4, Char4Norm];
    [u16; 2] = [Ushort2, Ushort2Norm];
    [u16; 4] = [Ushort4, Ushort4Norm];
    [i16; 2] = [Short2, Short2Norm];
    [i16; 4] = [Short4, Short4Norm];
    // Half2 has no core rust equivalent
    // Half4 has no core rust equivalent
    f32 = [Float];
    [f32; 2] = [Float2];
    [f32; 3] = [Float3];
    [f32; 4] = [Float4];
    u32 = [Uint];
    [u32; 2] = [Uint2];
    [u32; 3] = [Uint3];
    [u32; 4] = [Uint4];
    i32 = [Int];
    [i32; 2] = [Int2];
    [i32; 3] = [Int3];
    [i32; 4] = [Int4];
}

macro_rules! vek_into_vertex_attrib {
    ($(
        ($($vek:tt)*) -> $abi:ty;
    )*)=>{
        $(
        impl IntoVertexAttrib for vek::vec::repr_c::$($vek)* {
            type Into = $abi;

            fn into_vertex_attrib(self) -> $abi {
                self.into_array()
            }
        }

        #[cfg(feature = "simd-nightly")]
        impl IntoVertexAttrib for vek::vec::repr_simd::$($vek)* {
            type Into = $abi;

            fn into_vertex_attrib(self) -> $abi {
                self.into_array()
            }
        }
        )*
    };
}

vek_into_vertex_attrib! {
    (Vec2<u8>) -> [u8; 2];
    (Vec4<u8>) -> [u8; 4];
    (Extent2<u8>) -> [u8; 2];
    (Rgba<u8>) -> [u8; 4];

    (Vec2<i8>) -> [i8; 2];
    (Vec4<i8>) -> [i8; 4];
    (Extent2<i8>) -> [i8; 2];
    (Rgba<i8>) -> [i8; 4];

    (Vec2<u16>) -> [u16; 2];
    (Vec4<u16>) -> [u16; 4];
    (Extent2<u16>) -> [u16; 2];
    (Rgba<u16>) -> [u16; 4];

    (Vec2<i16>) -> [i16; 2];
    (Vec4<i16>) -> [i16; 4];
    (Extent2<i16>) -> [i16; 2];
    (Rgba<i16>) -> [i16; 4];

    (Vec2<f32>) -> [f32; 2];
    (Vec3<f32>) -> [f32; 3];
    (Vec4<f32>) -> [f32; 4];
    (Extent2<f32>) -> [f32; 2];
    (Extent3<f32>) -> [f32; 3];
    (Rgb<f32>) -> [f32; 3];
    (Rgba<f32>) -> [f32; 4];

    (Vec2<u32>) -> [u32; 2];
    (Vec3<u32>) -> [u32; 3];
    (Vec4<u32>) -> [u32; 4];
    (Extent2<u32>) -> [u32; 2];
    (Extent3<u32>) -> [u32; 3];
    (Rgb<u32>) -> [u32; 3];
    (Rgba<u32>) -> [u32; 4];

    (Vec2<i32>) -> [i32; 2];
    (Vec3<i32>) -> [i32; 3];
    (Vec4<i32>) -> [i32; 4];
    (Extent2<i32>) -> [i32; 2];
    (Extent3<i32>) -> [i32; 3];
    (Rgb<i32>) -> [i32; 3];
    (Rgba<i32>) -> [i32; 4];
}

#[test]
fn macro_test() {
    use vek::*;
    
    struct Example {
        pos: Vec3<f32>,
        tex_coord: Vec2<f32>,
        tex_index: u32,
    }
    
    vertex! {
        Example {
            layout(location = 0) in vec3 pos: Vec3<f32>,
            layout(location = 1) in vec2 tex_coord: Vec2<f32>,
            layout(location = 2) in uint tex_index: u32,
        }
    }
}