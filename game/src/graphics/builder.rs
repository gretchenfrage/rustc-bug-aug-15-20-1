//! Game renderer factory. 

use super::*;
use crate::graphics::draw_blocks::builder::DrawBlocksBuilder;
use vek::*;
use std::sync::{
    mpsc,
    Arc,
};

/// Game renderer factory. 
pub struct GraphicsBuilder {
    draw_blocks: DrawBlocksBuilder,
}

impl GraphicsBuilder {
    /// Create a new `GraphicsBuilder` in its default state. 
    pub fn new() -> Self {
        GraphicsBuilder {
            draw_blocks: DrawBlocksBuilder::new(),
        }
    }

    /// Add a block texture to the block texture array. Return its index. 
    ///
    /// The parameter, `bytes`, is the contents of an image file, such as PNG 
    /// or JPEG. This makes an educated guess about which format it is.  
    pub fn add_block_texture(&mut self, bytes: &[u8]) -> Result<u32> {
        self.draw_blocks.add_block_texture(bytes)
    }

    /// Attempt to construct a renderer. 
    /// 
    /// Once everything is initialized, the actual rendering should be done in 
    /// a render loop through the `MainHijacker`. 
    pub fn build(self) -> Result<(Graphics, MainHijacker)> {
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop)?;
        let instance = Instance::new(BackendBit::PRIMARY);
        let surface = unsafe {
            instance.create_surface(&window)
        };
        let adapter = block_on({
                instance.request_adapter(
                    &RequestAdapterOptions {
                        power_preference: PowerPreference::Default,
                        compatible_surface: Some(&surface),
                    })
            })
            .ok_or_else(|| pear!({}, "no graphics adapter found"))?;
        let (device, queue) = block_on(
            adapter.request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    shader_validation: false,
                    limits: Limits::default(),
                },
                None)
            )?;
        let window_size = window.inner_size();
        let swapchain_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: SWAPCHAIN_FMT,
            width: window_size.width,
            height: window_size.height,
            present_mode: PresentMode::Mailbox,
        };
        let swapchain = device.create_swap_chain(&surface, &swapchain_desc);
        let depth_texture_desc = TextureDescriptor {
            size: Extent3d {
                width: swapchain_desc.width,
                height: swapchain_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_FMT,
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            label: label("depth buffer"),
        };
        let depth_texture = device.create_texture(&depth_texture_desc);
        let depth_texture_view = depth_texture.create_default_view();
        
        let (events_send, events_recv) = mpsc::channel();
        let mut gfx = Graphics {
            window: Arc::new(window),
            surface,
            adapter,
            device,
            queue,
            swapchain_desc,
            swapchain,
            depth_texture_desc,
            depth_texture_view,
            window_size,
            window_size_changed: false,
            events_recv,
            cam: Camera::default(),
            subsystems: None,
        };

        // command encoder for subsystems to load
        let mut command_encoder = gfx.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: label("initialize command encoder"),
            });
        let draw_blocks = self.draw_blocks.build(&mut gfx, &mut command_encoder)?;

        gfx.queue.submit(once(command_encoder.finish()));

        let subsystems = Subsystems {
            draw_blocks,
        };
        gfx.subsystems = Some(subsystems);

        let hijacker = MainHijacker {
            event_loop,
            events_send,
        };
        Ok((gfx, hijacker))
    }
}