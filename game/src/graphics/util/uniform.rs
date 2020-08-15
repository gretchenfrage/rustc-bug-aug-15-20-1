//! Uniform buffer utilities, especially mucking around with bytes. 

use bytemuck::{Pod, bytes_of};

/// Trait for shader uniform buffers. 
pub trait GenericUniforms {
    /// Binary size of a uniform buffer. 
    const SIZE: usize;

    /// Write `self`'s binary representation to a byte array. 
    ///
    /// `bytes` must be `Self::SIZE` in length. 
    fn encode_to(&self, bytes: &mut [u8]);
}

/// Types which can convert into an ABI-compatible GLSL uniform struct field. 
///
/// There are some implementations which could exist, but do not currently, 
/// like booleans and non-square matrices. 
pub trait UniformField {
    /// The corresponding GLSL type. 
    const GLSL_TYPE: &'static str;

    /// Binary-compatible representation. 
    type Abi: Pod;

    /// Convert to binary-compatible representation. 
    fn to_abi(&self) -> Self::Abi;
}

/// Implement `GenericUniform` on a type with GLSL-like syntax. 
///
/// Example:
/// ```
/// use vek::*;
/// 
/// struct Example {
///     a: i32,
///     b: Mat3<f32>,
///     c: Vec2<u32>,
///     d: Rgba<f64>,
/// }
/// 
/// uniform! {
///     Example {
///         int a: i32,
///         mat3 b: Mat3<f32>,
///         uvec2 c: Vec2<u32>,
///         dvec4 d: Rgba<f64>,
///     }
/// }
/// ```
macro_rules! uniforms {
    (
    $uniform:ident {$(
        $glsl_type:ident $rust_field:ident: $rust_type:ty,
    )*}
    )=>{
        impl $uniform {
            /// Write `self`'s binary representation to a fixed-size byte array. 
            pub fn encode(&self) -> [u8; <Self as $crate::graphics::util::uniform::GenericUniforms>::SIZE] {
                let mut array = [0_u8; <Self as $crate::graphics::util::uniform::GenericUniforms>::SIZE];
                <Self as $crate::graphics::util::uniform::GenericUniforms>::encode_to(self, &mut array);
                array
            }
        }

        impl $crate::graphics::util::uniform::GenericUniforms for $uniform {
            const SIZE: usize = {
                #[repr(C)]
                struct Abi {$(
                    $rust_field: <
                        $rust_type 
                        as 
                        $crate::graphics::util::uniform::UniformField
                    >::Abi,
                )*}

                std::mem::size_of::<Abi>()
            };

            fn encode_to(&self, bytes: &mut [u8]) {
                assert_eq!(bytes.len(), Self::SIZE, "wrong size");

                #[repr(C)]
                struct Abi {$(
                    $rust_field: <
                        $rust_type 
                        as 
                        $crate::graphics::util::uniform::UniformField
                    >::Abi,
                )*}

                impl $crate::graphics::util::pod_fields::PodFields for Abi {
                    fn visit_fields<V>(&self, visitor: &mut V)
                    where
                        V: $crate::graphics::util::pod_fields::PodFieldVisitor,
                    {
                        $(
                        visitor.visit(&self.$rust_field);
                        )*
                    }
                }

                let abi = Abi {$(
                    $rust_field: <
                        $rust_type 
                        as 
                        $crate::graphics::util::uniform::UniformField
                    >::to_abi(&self.$rust_field),
                )*};

                $crate::graphics::util::pod_fields::copy_bytes_to(
                    &abi,
                    bytes,
                );
            }
        }
    };
}

macro_rules! self_uniform_field {
    ($(
        $glsl:literal $type:ty,
    )*)=>{
        $(
        impl UniformField for $type {
            const GLSL_TYPE: &'static str = $glsl;

            type Abi = Self;

            fn to_abi(&self) -> Self {
                *self
            }
        }
        )*
    };
}

self_uniform_field! {
    "int" i32,
    "uint" u32,
    "float" f32,
    "double" f64,
}

macro_rules! vec_uniform_field {
    ($(
        ($glsl:literal $($vek:tt)*) -> $array:ty,
    )*)=>{
        $(
        impl UniformField for vek::vec::repr_c::$($vek)* {
            const GLSL_TYPE: &'static str = $glsl;

            type Abi = $array;

            fn to_abi(&self) -> $array {
                self.into_array()
            }
        }

        #[cfg(feature = "simd-nightly")]
        impl UniformField for vek::vec::repr_simd::$($vek)* {
            const GLSL_TYPE: &'static str = $glsl;

            type Abi = $array;

            fn to_abi(&self) -> $array {
                self.into_array()
            }
        }
        )*
    };
}

vec_uniform_field! {
    // i32
    ("ivec2" Vec2<i32>) -> [i32; 2],
    ("ivec3" Vec3<i32>) -> [i32; 3],
    ("ivec4" Vec4<i32>) -> [i32; 4],

    ("ivec3" Rgb<i32>) -> [i32; 3],
    ("ivec4" Rgba<i32>) -> [i32; 4],

    ("ivec2" Extent2<i32>) -> [i32; 2],
    ("ivec3" Extent3<i32>) -> [i32; 3],

    // u32
    ("uvec2" Vec2<u32>) -> [u32; 2],
    ("uvec3" Vec3<u32>) -> [u32; 3],
    ("uvec4" Vec4<u32>) -> [u32; 4],

    ("uvec3" Rgb<u32>) -> [u32; 3],
    ("uvec4" Rgba<u32>) -> [u32; 4],

    ("uvec2" Extent2<u32>) -> [u32; 2],
    ("uvec3" Extent3<u32>) -> [u32; 3],

    // f32
    ("vec2" Vec2<f32>) -> [f32; 2],
    ("vec3" Vec3<f32>) -> [f32; 3],
    ("vec4" Vec4<f32>) -> [f32; 4],

    ("vec3" Rgb<f32>) -> [f32; 3],
    ("vec4" Rgba<f32>) -> [f32; 4],

    ("vec2" Extent2<f32>) -> [f32; 2],
    ("vec3" Extent3<f32>) -> [f32; 3],

    // f64
    ("dvec2" Vec2<f64>) -> [f64; 2],
    ("dvec3" Vec3<f64>) -> [f64; 3],
    ("dvec4" Vec4<f64>) -> [f64; 4],

    ("dvec3" Rgb<f64>) -> [f64; 3],
    ("dvec4" Rgba<f64>) -> [f64; 4],

    ("dvec2" Extent2<f64>) -> [f64; 2],
    ("dvec3" Extent3<f64>) -> [f64; 3],
}

macro_rules! mat_uniform_field {
    ($(
        ($glsl:literal $($vek:tt)*) -> $array:ty,
    )*)=>{
        $(
        impl UniformField for vek::mat::repr_c::column_major::$($vek)* {
            const GLSL_TYPE: &'static str = $glsl;

            type Abi = $array;

            fn to_abi(&self) -> $array {
                self.into_col_array()
            }
        }

        impl UniformField for vek::mat::repr_c::row_major::$($vek)* {
            const GLSL_TYPE: &'static str = $glsl;

            type Abi = $array;

            fn to_abi(&self) -> $array {
                self.into_col_array()
            }
        }

        #[cfg(feature = "simd-nightly")]
        impl UniformField for vek::mat::repr_simd::column_major::$($vek)* {
            const GLSL_TYPE: &'static str = $glsl;

            type Abi = $array;

            fn to_abi(&self) -> $array {
                self.into_col_array()
            }
        }

        #[cfg(feature = "simd-nightly")]
        impl UniformField for vek::mat::repr_simd::row_major::$($vek)* {
            const GLSL_TYPE: &'static str = $glsl;

            type Abi = $array;

            fn to_abi(&self) -> $array {
                self.into_col_array()
            }
        }
        )*
    };
}

mat_uniform_field! {
    ("mat2" Mat2<f32>) -> [f32; 4],
    ("mat3" Mat3<f32>) -> [f32; 9],
    ("mat4" Mat4<f32>) -> [f32; 16],

    ("dmat2" Mat2<f64>) -> [f64; 4],
    ("dmat3" Mat3<f64>) -> [f64; 9],
    ("dmat4" Mat4<f64>) -> [f64; 16],
}

#[test]
fn macro_test() {
    use vek::*;

    struct Example {
        a: i32,
        b: Mat3<f32>,
        c: Vec2<u32>,
        d: Rgba<f64>,
    }

    uniforms! {
        Example {
            int a: i32,
            mat3 b: Mat3<f32>,
            uvec2 c: Vec2<u32>,
            dvec4 d: Rgba<f64>,
        }
    }
}