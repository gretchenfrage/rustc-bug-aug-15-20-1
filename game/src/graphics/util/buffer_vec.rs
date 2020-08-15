//! Resizeable GPU buffer. 

use super::mesh_diff::MeshPatch;
use crate::graphics::label;
use std::{
    borrow::Cow,
    mem::size_of,
    marker::PhantomData,
};
use bytemuck::{self, Pod};
use wgpu::{
    *, 
    util::{
        DeviceExt,
        BufferInitDescriptor,
    },
};

/// Resizeable GPU buffer. 
///
/// Designed to work in tandem with `MeshDiffer`. 
pub struct BufferVec<T: Pod> {
    label: Option<Cow<'static, str>>,
    usage: BufferUsage,

    buffer: Buffer,
    // length in elements
    len: usize,
    // capacity in elements
    capacity: usize,

    p: PhantomData<T>,
}

const BUFFER_VEC_DEFAULT_CAPACITY: usize = 512;

impl<T: Pod> BufferVec<T> {
    /// Create a new, empty `BufferVec`. 
    pub fn new(device: &Device, usage: BufferUsage, label: Option<Cow<'static, str>>) -> Self {
        let usage = usage | BufferUsage::COPY_DST | BufferUsage::COPY_SRC;

        let buffer = device.create_buffer(&BufferDescriptor {
            label: label.clone(),
            size: (BUFFER_VEC_DEFAULT_CAPACITY * size_of::<T>()) as u64,
            usage,
            mapped_at_creation: false,
        });
        BufferVec {
            label,
            usage,
            buffer,
            capacity: BUFFER_VEC_DEFAULT_CAPACITY,
            len: 0,
            p: PhantomData,
        }
    }

    /// Access the current underlying buffer. 
    pub fn as_inner(&self) -> &Buffer {
        &self.buffer
    }

    /// View the initialized elements as a slice. 
    pub fn as_buffer_slice(&self) -> BufferSlice {
        self.buffer.slice(0..self.len_bytes() as u64)
    }

    /// Current length in elements. 
    pub fn len_elems(&self) -> usize {
        self.len
    }

    /// Current length in bytes.
    pub fn len_bytes(&self) -> usize {
        self.len * size_of::<T>()
    }

    /// Current capacity in elems. 
    pub fn capacity_elems(&self) -> usize {
        self.capacity
    }

    /// Current capacity in bytes. 
    pub fn capacity_bytes(&self) -> usize {
        self.capacity * size_of::<T>()
    }

    /// Unconditionally reallocate. 
    fn realloc(
        &mut self,
        new_capacity: usize,
        device: &Device,
        command_encoder: &mut CommandEncoder,
    ) {
        assert!(self.len <= new_capacity);

        let new_buffer = device.create_buffer(&BufferDescriptor {
            label: self.label.clone(),
            size: (new_capacity * size_of::<T>()) as u64,
            usage: self.usage,
            mapped_at_creation: false,
        });

        command_encoder.copy_buffer_to_buffer(
            &self.buffer,
            0,
            &new_buffer,
            0,
            self.len_bytes() as u64,
        );

        self.buffer = new_buffer;
        self.capacity = new_capacity;
    }

    /// Set the current length (in elems), which may trigger a re-allocation. 
    ///
    /// This requires a `CommandEncoder` to record commands to complete the 
    /// potential re-allocation, as well as a `Device` to create the potential 
    /// new underlying allocation. 
    pub fn set_len(
        &mut self,
        new_len: usize,
        device: &Device,
        command_encoder: &mut CommandEncoder,
    ) {
        let mut new_capacity = self.capacity_elems();
        if new_capacity < new_len {
            while new_capacity < new_len {
                new_capacity *= 2;
            }
        } else if new_capacity > new_len * 4 {
            new_capacity /= 4;
            if new_capacity < BUFFER_VEC_DEFAULT_CAPACITY {
                new_capacity = BUFFER_VEC_DEFAULT_CAPACITY;
            }
        }

        if new_capacity != self.capacity_elems() {
            self.realloc(new_capacity, device, command_encoder);
        }

        self.len = new_len;
    }

    /// Apply a `mesh_diff::MeshPatch` to `self`.
    pub fn apply_patch(
        &mut self, 
        patch: &MeshPatch<T>,
        device: &Device,
        command_encoder: &mut CommandEncoder,
    ) {
        // there may be a length change even if there's no writes
        self.set_len(patch.new_len, device, command_encoder);

        // cheap case
        if patch.writes_data.is_empty() {
            return;
        }

        trace!("writing patches to buffer vec");

        // copy data, with a single src buffer
        let copy_src_bytes: &[u8] = bytemuck::cast_slice(&patch.writes_data);
        let copy_src_buffer = device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("buffer vec patch source"),
                contents: copy_src_bytes,
                usage: BufferUsage::COPY_SRC,
            });

        for part in patch.iter_contiguous() {
            command_encoder
                .copy_buffer_to_buffer(
                    &copy_src_buffer,
                    (part.src_start * size_of::<T>()) as u64,
                    self.as_inner(),
                    (part.dst_start * size_of::<T>()) as u64,
                    (part.len * size_of::<T>()) as u64,
                );
        }
    }
}
