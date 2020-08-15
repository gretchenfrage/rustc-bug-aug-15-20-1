//! `DrawBlocks` subsystem factory. 

use super::*;
use crate::graphics::util::texture_array::TextureArrayBuilder;
use core::num::NonZeroU64;
use vek::*;

const BLOCK_TEXTURE_SIZE: u32 = 16;

/// `DrawBlocks` subsystem factory. 
pub struct DrawBlocksBuilder {
    block_textures: TextureArrayBuilder,
}

/// Expresses a `ShaderModuleSource<'static>`
macro_rules! include_shader {
    ($file:expr)=>{{
        let bytes: &[u8] = include_bytes!($file);
        assert_eq!(bytes.len() % 4, 0);
        let words: &[u32] = unsafe {
            std::slice::from_raw_parts(
                bytes.as_ptr() as *const u8 as *const u32,
                bytes.len() / 4,
            )
        };
        wgpu::ShaderModuleSource::SpirV(
            std::borrow::Cow::Borrowed(
                words
            )
        )
    }};
}

impl DrawBlocksBuilder {
    /// Create a new `DrawBlocksBuilder` in its default state. 
    pub fn new() -> Self {
        let block_textures = TextureArrayBuilder::new([BLOCK_TEXTURE_SIZE; 2]);
        DrawBlocksBuilder {
            block_textures,
        }
    }

    /// Add a block texture to the block texture array. Return its index. 
    ///
    /// The parameter, `bytes`, is the contents of an image file, such as PNG 
    /// or JPEG. This makes an educated guess about which format it is.  
    pub fn add_block_texture(&mut self, bytes: &[u8]) -> Result<u32> {
        trace!("adding block texture");
        self.block_textures.add_layer(bytes)
    }

    /// Attempt to initialize the `DrawBlocks` subsystem.  
    pub fn build(self, gfx: &mut Graphics, command_encoder: &mut CommandEncoder) -> Result<DrawBlocks> {
        // buffers and textures
        let vertex_buffer = BufferVec::new(
                &gfx.device, 
                BufferUsage::VERTEX,
                label("blocks vertex buffer"),
            );
        let (
            block_texture_array, 
            block_sampler_array,
        ) = self.block_textures.build(&gfx.device, command_encoder);
        let uniform_buffer = gfx.device
            .create_buffer(&BufferDescriptor {
                label: label("draw blocks uniform buffer"),
                size: Uniforms::SIZE as u64,
                usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
                mapped_at_creation: false,
            });

        // shaders
        let vert_module = gfx.device
            .create_shader_module(include_shader!("shader.vert.spv"));
        let frag_module = gfx.device
            .create_shader_module(include_shader!("shader.frag.spv"));

        // binding and pileline
        let bind_group_layout = gfx.device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: label("blocks bind group layout"),
                entries: cowslice![
                    // uniform buffer
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                        ty: BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: Some(NonZeroU64::new(Uniforms::SIZE as u64).unwrap()),
                        },
                        count: None,
                    },
                    // block texture array
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            dimension: TextureViewDimension::D2Array,
                            component_type: TextureComponentType::Float,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // block texture sampler
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                        ty: BindingType::Sampler {
                            comparison: false,
                        },
                        count: None,
                    },
                ],
            });
        let bind_group = gfx.device
            .create_bind_group(&BindGroupDescriptor {
                label: label("blocks bind group"),
                layout: &bind_group_layout,
                entries: cowslice![
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(uniform_buffer.slice(..)),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&block_texture_array),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(&block_sampler_array),
                    },
                ],
            });
        let pipeline_layout = gfx.device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: cowslice![&bind_group_layout],
                push_constant_ranges: cowslice![],
            });
        let pipeline = gfx.device
            .create_render_pipeline(&RenderPipelineDescriptor {
                layout: &pipeline_layout,
                vertex_stage: ProgrammableStageDescriptor {
                    module: &vert_module,
                    entry_point: cowstr("main"),
                },
                fragment_stage: Some(ProgrammableStageDescriptor {
                    module: &frag_module,
                    entry_point: cowstr("main"),
                }),
                rasterization_state: Some(RasterizationStateDescriptor {
                    front_face: FrontFace::Ccw,
                    cull_mode: CullMode::Back,
                    clamp_depth: false,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                primitive_topology: PrimitiveTopology::TriangleList,
                color_states: cowslice![
                    ColorStateDescriptor {
                        format: SWAPCHAIN_FMT,
                        color_blend: BlendDescriptor::REPLACE,
                        alpha_blend: BlendDescriptor::REPLACE,
                        write_mask: ColorWrite::ALL,
                    },
                ],
                depth_stencil_state: Some(DepthStencilStateDescriptor {
                    format: DEPTH_FMT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::LessEqual,
                    stencil_front: StencilStateFaceDescriptor::IGNORE,
                    stencil_back: StencilStateFaceDescriptor::IGNORE,
                    stencil_read_mask: 0,
                    stencil_write_mask: 0,
                }),
                vertex_state: wgpu::VertexStateDescriptor {
                    // it's actually not indexed at all
                    index_format: IndexFormat::Uint16, 
                    vertex_buffers: cowslice![
                        VertexBufferDescriptor {
                            stride: Vertex::SIZE as u64,
                            step_mode: InputStepMode::Vertex,
                            attributes: Vertex::attributes(),
                        },
                    ],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        Ok(DrawBlocks {
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            mesh_differ: MeshDiffer::new(),
            block_texture_array,
            block_sampler_array
        })
    }
}