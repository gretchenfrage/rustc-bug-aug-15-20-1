//! Texture array utilities. 

use std::iter::repeat;
use crate::graphics::label;
use wgpu::{
    *, 
    util::{
        DeviceExt,
        BufferInitDescriptor,
    },
};
use vek::*;
use image::{
    self,
    imageops::{
        self,
        FilterType,
    },
};
use pear::*;

/// Utility for arranging texture array data, then uploading it to WGPU. 
///
/// Currently, this is hard-coded to use the `Rgba8UnormSrgb` format. 
#[derive(Clone)]
pub struct TextureArrayBuilder {
    dim: Extent2<u32>,
    layers: u32,
    data: Vec<u8>,
    bytes_per_row: u32,
}

impl TextureArrayBuilder {
    /// Begin building a texture array with the specified dimensions. 
    pub fn new<E: Into<Extent2<u32>>>(dim: E) -> Self {
        let dim = dim.into();

        let bytes_per_row = match dim.w % COPY_BYTES_PER_ROW_ALIGNMENT {
            0 => dim.w,
            rem => (dim.w - rem + 1) * COPY_BYTES_PER_ROW_ALIGNMENT,
        };

        TextureArrayBuilder {
            dim: dim.into(),
            layers: 0,
            data: Vec::new(),
            bytes_per_row,
        }
    }

    /// Add a layer to the texture array. Return its index. 
    ///
    /// The parameter, `bytes`, is the contents of an image file, such as PNG 
    /// or JPEG. This makes an educated guess about which format it is.  
    pub fn add_layer(&mut self, bytes: &[u8]) -> Result<u32> {
        let mut image = image::load_from_memory(bytes)?
            .into_rgba();

        if image.dimensions() != self.dim.into_tuple() {
            warn!(
                image_dimensions = ?image.dimensions(),
                texture_array_dimensions = ?self.dim,
                "incorrect layer size, resizing"
            );
            image = imageops::resize(
                &image,
                self.dim.w,
                self.dim.h,
                FilterType::Nearest,
            );
        }

        let raw = image.into_raw();
        for row in 0..self.dim.h {
            let i1 = (row * self.dim.w * 4) as usize;
            let i2 = ((row + 1) * self.dim.w * 4) as usize;
            let pad = (self.bytes_per_row - (self.dim.w * 4)) as usize;

            self.data.extend(raw[i1..i2].iter().copied());
            self.data.extend(repeat(0).take(pad));
        }

        let layer = self.layers;
        self.layers += 1;
        Ok(layer)
    }

    /// Get the number of layers loaded into this texture array. 
    pub fn num_layers(&self) -> u32 {
        self.layers
    }

    /// Get the dimensions of this texture array. 
    pub fn dimensions(&self) -> Extent2<u32> {
        self.dim
    }

    /// Upload data to the GPU, creating a `TextureView` and `Sampler` for this texture array. 
    pub fn build(
        &self, 
        device: &Device, 
        command_encoder: &mut CommandEncoder,
    ) -> (TextureView, Sampler) {
        trace!("uploading texture array to WGPU");

        let texture = device
            .create_texture(&TextureDescriptor {
                label: label("texture array"),
                size: Extent3d {
                    width: self.dim.w,
                    height: self.dim.h,
                    depth: self.layers,
                },
                // TODO: mipmapping
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            });

        let copy_src = device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("texture array upload buffer"),
                contents: &self.data,
                usage: BufferUsage::COPY_SRC,
            });
        for layer in 0..self.layers {
            command_encoder
                .copy_buffer_to_texture(
                    BufferCopyView {
                        buffer: &copy_src,
                        layout: TextureDataLayout {
                            offset: (layer * self.bytes_per_row * self.dim.h) as u64,
                            bytes_per_row: self.bytes_per_row,
                            rows_per_image: self.dim.h,
                        },
                    },
                    TextureCopyView {
                        texture: &texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                    },
                    Extent3d {
                        width: self.dim.w,
                        height: self.dim.h,
                        depth: 1,
                    }
                );
        }

        let texture_view = texture.create_default_view();
        let sampler = device
            .create_sampler(&SamplerDescriptor {
                label: label("texture array sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Nearest,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                compare: None,
                anisotropy_clamp: None,
            });

        (texture_view, sampler)
    }
}
