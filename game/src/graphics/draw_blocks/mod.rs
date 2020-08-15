//! Graphics subsystem for drawing the block grid. 

use super::*;
use crate::graphics::util::{
    vertex::GenericVertex,
    uniform::GenericUniforms,
    mesh_diff::MeshDiffer,
    buffer_vec::BufferVec,
    CORR,
};
use vek::*;
use crate::arraymap::ArrayMap;

pub mod builder;

/// Graphics subsystem for drawing the block grid. 
pub struct DrawBlocks {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    uniform_buffer: Buffer,
    mesh_differ: MeshDiffer<[i32; 3], [[u8; Vertex::SIZE]; 3]>,
    vertex_buffer: BufferVec<[[u8; Vertex::SIZE]; 3]>,
    block_texture_array: TextureView,
    block_sampler_array: Sampler,
}

impl DrawBlocks {
    /// Draw a frame. 
    pub fn draw(
        &mut self,
        gfx: &mut Graphics, 
        frame: &mut SwapChainFrame, 
        command_encoder: &mut CommandEncoder,
    ) -> Result<()> {
        // update mesh
        let patch = self.mesh_differ.commit();
        self.vertex_buffer.apply_patch(&patch, &gfx.device, command_encoder);

        // set uniforms
        let uniforms = Uniforms {
            corr_proj_view: CORR * gfx.cam.proj() * gfx.cam.view(),
        };
        let uniforms_copy_src = gfx.device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("uniforms copy src"),
                contents: &uniforms.encode(),
                usage: BufferUsage::COPY_SRC,
            });
        command_encoder
            .copy_buffer_to_buffer(
                &uniforms_copy_src,
                0,
                &self.uniform_buffer,
                0,
                Uniforms::SIZE as u64,
            );

        // render pass
        let mut pass = command_encoder
            .begin_render_pass(&RenderPassDescriptor {
                color_attachments: cowslice![
                    RenderPassColorAttachmentDescriptor {
                        attachment: &frame.output.view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(CLEAR_COLOR),
                            store: true,
                        }
                    },
                ],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &gfx.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(CLEAR_DEPTH),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.as_buffer_slice());

        let num_prims = self.vertex_buffer.len_elems() as u32 * 3;
        pass.draw(0..num_prims, 0..1);

        Ok(())
    }

    /// Set the mesh geometry for a single block. 
    ///
    /// Override any existing mesh geometry for that block. 
    pub fn set_block_mesh<I>(&mut self, pos: Vec3<i32>, mesh: I)
    where
        I: IntoIterator<Item=[Vertex; 3]>
    {
        let verts = mesh
            .into_iter()
            .map(|array| array.map(|vert| vert.encode()));
        self.mesh_differ.stage(pos.into_array(), verts);
    }
}

/// Block vertex type.
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vec3<f32>,
    pub tex_coord: Vec2<f32>,
    pub tex_index: u32,
}

vertex! {
    Vertex {
        layout(location = 0) in vec3 pos: Vec3<f32>,
        layout(location = 1) in vec2 tex_coord: Vec2<f32>,
        layout(location = 2) in uint tex_index: u32,
    }
}

/// Draw terrain uniform type. 
#[derive(Copy, Clone, Debug)]
pub struct Uniforms {
    /// Correction * Projection * View
    pub corr_proj_view: Mat4<f32>,
}

uniforms! {
    Uniforms {
        mat4 corr_proj_view: Mat4<f32>,
    }
}
