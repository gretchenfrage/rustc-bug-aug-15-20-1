#![allow(unused_imports)]

#[macro_use]
extern crate pear;
#[macro_use]
extern crate tracing;
extern crate tracing_subscriber;

extern crate wgpu;
extern crate winit;
extern crate bytemuck;
extern crate memoffset;
extern crate image;

extern crate rand;
extern crate mint;
extern crate vek;
extern crate num_traits;

extern crate iter_vals;
extern crate arraymap;
extern crate futures;
extern crate array_iterator;
extern crate smallvec;

pub mod graphics;
pub mod util;
pub mod input;

use std::time::{Instant, Duration};
use util::{
    fps_tracker::FpsTracker,
    angle::*,
    camera::YawPitch,
};
use graphics::{
    *,
    builder::GraphicsBuilder,
};
use input::{
    Key,
    InputManagerBuilder,
    InputManager,
    InputEvent,
    MouseButton,
    WindowState,
};
use pear::*;
use arraymap::ArrayMap;

/// Target FPS. 
const FPS: u32 = 60;

use vek::*;

fn main() {
    try_main()
        .unwrap_or_else(|e| {
            error!("{}", e);
            panic!("main function failed");
        });
}

fn try_main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        //.event_format(tracing_subscriber::fmt::format::json())
        //.with_env_filter("trace")
        .with_env_filter("warn,game=trace,pear=trace,floatilla=trace")
        //.with_env_filter("warn,game=trace,pear=trace,floatilla=trace,gfx_backend_vulkan=error")
        .init();
    
    // initialize
    let mut graphics = GraphicsBuilder::new();
    graphics.add_block_texture(include_bytes!("textures/stone.png"))?;
    graphics.add_block_texture(include_bytes!("textures/dirt.png"))?;
    graphics.add_block_texture(include_bytes!("textures/grass.png"))?;
    graphics.add_block_texture(include_bytes!("textures/grass_side.png"))?;
    graphics.add_block_texture(include_bytes!("textures/sand.png"))?;
    graphics.add_block_texture(include_bytes!("textures/snow.png"))?;
    graphics.add_block_texture(include_bytes!("textures/ice.png"))?;
    graphics.add_block_texture(include_bytes!("textures/hellstone.png"))?;
    graphics.add_block_texture(include_bytes!("textures/gravel.png"))?;
    graphics.add_block_texture(include_bytes!("textures/coal_ore.png"))?;
    graphics.add_block_texture(include_bytes!("textures/iron_ore.png"))?;
    graphics.add_block_texture(include_bytes!("textures/gold_ore.png"))?;
    graphics.add_block_texture(include_bytes!("textures/diamond_ore.png"))?;
    graphics.add_block_texture(include_bytes!("textures/red_ore.png"))?;
    let mut fps_tracker = FpsTracker::default();

    let (mut graphics, hijacker) = graphics.build()?;

    let mut input = InputManagerBuilder::new();
    let move_forward = input.bind(Key::W);
    let move_backward = input.bind(Key::S);
    let move_left = input.bind(Key::A);
    let move_right = input.bind(Key::D);
    let move_up = input.bind(Key::Space);
    let move_down = input.bind(Key::LShift); 
    let mut input = input.build(graphics.window().clone());

    // put some bloxs
    for x in -5i32..=5 {
        for y in -5i32..=5 {
            for z in -5i32..=5 {
                let tex_index = x + y + z;
                let tex_index = (tex_index + 14) % 14;
                let pos = Vec3::new(x, y, z) * 4;
                let geom = BLOCK_MESH_TEMPLATE
                    .iter()
                    .flat_map(|face| face.iter())
                    .map(|prim| {
                        prim.map(|vert| DrawBlocksVertex {
                            pos: (vert.pos + pos).map(|n| n as f32),
                            tex_coord: vert.tex.map(|n| n as f32),
                            tex_index: tex_index as u32,
                        })
                    });
                graphics.set_block_mesh(pos, geom);
            }
        }
    }

    // main loop
    hijacker.hijack(move |events: &[WinitEvent], delta: f32| {
        let start_time = Instant::now();
        fps_tracker.log_frame();

        input.update(events);

        if input.events()
            .iter()
            .any(|&(event, _)| event == InputEvent::Click(MouseButton::Left))
        {
            input.capture_mouse();
        }

        if input.state() == WindowState::Captured {
        let cam_dir: &mut YawPitch<f32> = graphics.cam_dir_mut();
        let look_speed: Angle<f32> = deg(0.1);
        let mouse_movement: Vec2<f32> = input.mouse_captured_movement().map(|n| n as f32);
        cam_dir.yaw += look_speed * mouse_movement.x;
        cam_dir.yaw %= deg(360.0);
        cam_dir.pitch += look_speed * mouse_movement.y;
        cam_dir.pitch = cam_dir.pitch.clamp(deg(-90.0), deg(90.0));

        let mut move_dir: Vec3<f32> = [0.0; 3].into();
            let forward_dir = Vec3::new(
                cam_dir.yaw.sin(), 
                0.0, 
                cam_dir.yaw.cos()
            );
            let right_dir = Vec3::new(
                (cam_dir.yaw + deg(90.0)).sin(),
                0.0,
                (cam_dir.yaw + deg(90.0)).cos(), 
            );
            if input.is_pressed(move_forward) {
                move_dir += forward_dir;
            }
            if input.is_pressed(move_backward) {
                move_dir -= forward_dir;
            }
            if input.is_pressed(move_right) {
                move_dir += right_dir;
            }
            if input.is_pressed(move_left) {
                move_dir -= right_dir;
            }
            if move_dir != [0.0; 3].into() {
                move_dir.normalize();
            }
            if input.is_pressed(move_up) {
                move_dir.y += 1.0;
            }
            if input.is_pressed(move_down) {
                move_dir.y -= 1.0;
            }
            let move_speed: f32 = 17.5;
            *graphics.cam_pos_mut() += move_dir * move_speed * delta;
        }

        if input.is_closing() {
            info!("exit by request");
            return Ok(ControlFlow::Exit);
        }

        graphics.draw()?;

        let wait_until = start_time + (Duration::from_secs(1) / FPS);
        Ok(ControlFlow::WaitUntil(wait_until))  
    })
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PosTex { pos: Vec3<i32>, tex: Vec2<i32> }

macro_rules! pos_tex_dsl {
    ($(
    $pos_x_1:expr, $pos_y_1:expr, $pos_z_1:expr, $tex_u_1:expr, $tex_v_1:expr,
    $pos_x_2:expr, $pos_y_2:expr, $pos_z_2:expr, $tex_u_2:expr, $tex_v_2:expr,
    $pos_x_3:expr, $pos_y_3:expr, $pos_z_3:expr, $tex_u_3:expr, $tex_v_3:expr,
    )*)=>{
        [$(
            [
                PosTex {
                    pos: Vec3 { x: $pos_x_1 as _, y: $pos_y_1 as _, z: $pos_z_1 as _ },
                    tex: Vec2 { x: $tex_u_1 as _, y: $tex_v_1 as _ },
                },
                PosTex {
                    pos: Vec3 { x: $pos_x_2 as _, y: $pos_y_2 as _, z: $pos_z_2 as _ },
                    tex: Vec2 { x: $tex_u_2 as _, y: $tex_v_2 as _ },
                },
                PosTex {
                    pos: Vec3 { x: $pos_x_3 as _, y: $pos_y_3 as _, z: $pos_z_3 as _ },
                    tex: Vec2 { x: $tex_u_3 as _, y: $tex_v_3 as _ },
                }
            ],
        )*]
    };
}

/// Boilerplate position and texture data for cube-shape blocks. 
///
/// 6 faces, with indices corresponding to `AxisUnit3::(to|from)_index`. 
/// Each face is two triangles. Front faces are counter-clockwise. 
pub const BLOCK_MESH_TEMPLATE: [[[PosTex; 3]; 2]; 6] = [
    pos_tex_dsl![
        // +X
        1, 0, 0, 0, 1,
        1, 0, 1, 1, 1,
        1, 1, 0, 0, 0,
        1, 1, 0, 0, 0,
        1, 0, 1, 1, 1,
        1, 1, 1, 1, 0,
    ],
    pos_tex_dsl![
        // +Y
        0, 1, 0, 0, 1,
        1, 1, 0, 1, 1,
        0, 1, 1, 0, 0,
        0, 1, 1, 0, 0,
        1, 1, 0, 1, 1,
        1, 1, 1, 1, 0,
    ],
    pos_tex_dsl![
        // +Z
        1, 0, 1, 0, 1,
        0, 0, 1, 1, 1,
        1, 1, 1, 0, 0,
        1, 1, 1, 0, 0,
        0, 0, 1, 1, 1,
        0, 1, 1, 1, 0,
    ],
    pos_tex_dsl![
        // -X
        0, 0, 1, 0, 1,
        0, 0, 0, 1, 1,
        0, 1, 1, 0, 0,
        0, 1, 1, 0, 0,
        0, 0, 0, 1, 1,
        0, 1, 0, 1, 0,
    ],
    pos_tex_dsl![
        // -Y
        0, 0, 1, 0, 1,
        1, 0, 1, 1, 1,
        0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
        1, 0, 1, 1, 1,
        1, 0, 0, 1, 0,
    ],
    pos_tex_dsl![
        // -Z
        0, 0, 0, 0, 1,
        1, 0, 0, 1, 1,
        0, 1, 0, 0, 0,
        0, 1, 0, 0, 0,
        1, 0, 0, 1, 1,
        1, 1, 0, 1, 0,
    ],
];
