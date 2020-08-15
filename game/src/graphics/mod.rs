//! Game renderer. 

use crate::util::{
    angle::*,
    camera::{YawPitch, Camera},
};
use self::{
    draw_blocks::DrawBlocks,
    util::{
        label,
        cowstr,
    },
};
use std::{
    borrow::Cow,
    time::Instant,
    iter::once,
    sync::{
        mpsc, 
        Arc,
    },
};
use pear::*;
use wgpu::{
    *, 
    util::{
        DeviceExt,
        BufferInitDescriptor,
    },
};
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::Window,
    event::{
        KeyboardInput,
        VirtualKeyCode,
        ElementState,
    },
};
use futures::executor::block_on;
use vek::*;

pub mod builder;
#[macro_use]
mod util;
mod draw_blocks;

pub use draw_blocks::Vertex as DrawBlocksVertex;

/// Texture format we use for the swapchain color.
const SWAPCHAIN_FMT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
/// Texture format we use for the depth buffer. 
const DEPTH_FMT: TextureFormat = TextureFormat::Depth32Float;
/// Swapchain clear color. 
const CLEAR_COLOR: Color = Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };
/// Depth buffer clear value. 
const CLEAR_DEPTH: f32 = 1.0;

/// Game renderer. 
pub struct Graphics {
    // core things
    window: Arc<Window>,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    swapchain_desc: SwapChainDescriptor,
    swapchain: SwapChain,
    depth_texture_desc: TextureDescriptor<'static>,
    depth_texture_view: TextureView,
    window_size: PhysicalSize<u32>,
    window_size_changed: bool,
    events_recv: mpsc::Receiver<WinitEvent>,

    // not so core things
    cam: Camera,

    // subsystems
    subsystems: Option<Subsystems>,
}

/// Subsystems get detached from `Graphics` so that we can call the subsystems 
/// while also mutably passing them `Graphics`. 
struct Subsystems {
    draw_blocks: DrawBlocks,
}

macro_rules! subsys {
    ($self:expr,$subsys:ident)=>{ $self.subsystems.as_mut().unwrap().$subsys };
}

impl Graphics {
    /// Draw a frame. 
    pub fn draw(&mut self) -> Result<()> {
        // process events
        while let Ok(event) = self.events_recv.try_recv() {
            // detect resize
            if let &WinitEvent::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } = &event {
                self.window_size = size;
                self.window_size_changed = true;
            }
        }

        // create command encoder
        let mut command_encoder = self.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: label("frame command encoder"),
            });

        // handle resize
        if self.window_size_changed {
            self.swapchain_desc.width = self.window_size.width;
            self.swapchain_desc.height = self.window_size.height;
            self.swapchain = self.device.create_swap_chain(&self.surface, &self.swapchain_desc);
            
            self.depth_texture_desc.size.width = self.window_size.width;
            self.depth_texture_desc.size.height = self.window_size.height;
            let depth_texture = self.device.create_texture(&self.depth_texture_desc);
            self.depth_texture_view = depth_texture.create_default_view();

            self.cam.aspect_ratio = self.window_size.width as f32 / self.window_size.height as f32;
        }
        self.window_size_changed = false;

        // create frame
        // must be done AFTER rebuilding swapchain
        let mut frame = self.swapchain.get_current_frame()?;

        // draw subsystems
        let mut subsystems = self.subsystems.take().unwrap();
        let mut subsys_errs = Vec::new();
        subsystems.draw_blocks.draw(self, &mut frame, &mut command_encoder)
            .push_err(&mut subsys_errs);
        self.subsystems = Some(subsystems);
        if !subsys_errs.is_empty() {
            return Err(subsys_errs.wrap(pear!({}, "drawing subsystem failure")));
        }

        // submit command encoder
        self.queue.submit(once(command_encoder.finish()));

        // request redraw
        self.window.request_redraw();

        Ok(())
    }

    /// Get the camera position. 
    pub fn cam_pos(&self) -> Vec3<f32> {
        self.cam.pos
    }

    /// Get the camera position by mutable reference. 
    pub fn cam_pos_mut(&mut self) -> &mut Vec3<f32> {
        &mut self.cam.pos
    }

    /// Get the camera direction.
    pub fn cam_dir(&self) -> YawPitch<f32> {
        self.cam.dir
    }

    /// Get the camera direction by mutable reference. 
    pub fn cam_dir_mut(&mut self) -> &mut YawPitch<f32> {
        &mut self.cam.dir
    }

    /// Get the camera field of view. 
    pub fn cam_fov(&self) -> Angle<f32> {
        self.cam.fov
    }

    /// Get the camera field of view by mutable reference. 
    pub fn cam_fov_mut(&mut self) -> &mut Angle<f32> {
        &mut self.cam.fov
    }

    /// Get the winit window. 
    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    /// Set the mesh geometry for a single block. 
    ///
    /// Override any existing mesh geometry for that block. 
    pub fn set_block_mesh<I>(&mut self, pos: Vec3<i32>, mesh: I)
    where
        I: IntoIterator<Item=[DrawBlocksVertex; 3]>
    {
        subsys!(self,draw_blocks).set_block_mesh(pos, mesh)
    }
}

/// Wrapper around `winit::event_loop::EventLoop`. 
///
/// Hijacks the main thread and never returns. 
pub struct MainHijacker {
    event_loop: EventLoop<()>,
    events_send: mpsc::Sender<WinitEvent>,
}

/// Winit event type. 
pub type WinitEvent = winit::event::Event<'static, ()>;

/// Main loop control flow. 
pub type ControlFlow = winit::event_loop::ControlFlow;

/// Window event type. 
pub type WindowEvent = winit::event::WindowEvent<'static>;

/// Handle for a `MainHijacker` frame. 
pub trait FrameHandler: 'static {
    /// Handle a single frame, with all the events that have accumulated since 
    /// last frame. 
    ///
    /// - `events` is all the events that have accumulated since last frame.
    /// - `delta` is measured in seconds. the first frame, it is 0. 
    fn frame(&mut self, events: &[WinitEvent], delta: f32) -> Result<ControlFlow>;
}

impl<F> FrameHandler for F 
where
    F: FnMut(&[WinitEvent], f32) -> Result<ControlFlow> + 'static
{
    fn frame(&mut self, events: &[WinitEvent], delta: f32) -> Result<ControlFlow>
    {
        self(events, delta)
    }
}

impl MainHijacker {
    /// Take control of the main thread and enter the draw loop. 
    ///
    /// This draws until `handler` returns `Err` or `ControlFlow::Exit`.
    pub fn hijack<H: FrameHandler>(self, mut handler: H) -> !
    {
        let mut events = Vec::new();
        let mut last_instant: Option<Instant> = None;
        let MainHijacker {
            event_loop,
            events_send,
        } = self;
        event_loop.run(move |event, _, curr_flow| {
            match event.to_static() {
                Some(WinitEvent::MainEventsCleared) => {
                    let instant = Instant::now();
                    let delta: f32 = last_instant
                        .map(|last| { 
                            (
                                (instant - last).as_nanos() as f64 
                                / 1000000000.0
                            ) as f32
                        })
                        .unwrap_or(0.0);
                    last_instant = Some(instant);

                    for event in events.iter().cloned() {
                        let _ = events_send.send(event);
                    }
                    match handler.frame(&events, delta) {
                        Ok(flow) => {
                            *curr_flow = flow;
                        }
                        Err(error) => {
                            error!("frame handler returned error:");
                            error!("{}", error);
                            *curr_flow = ControlFlow::Exit;
                        }
                    };
                    events.clear();
                }
                Some(event) => {
                    events.push(event);
                }
                None => {
                    warn!("non-static winit event");
                }
            };
        })
    }
}

