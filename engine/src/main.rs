#![feature(box_syntax, type_alias_impl_trait)]

mod asset_management;
mod buffer;
mod camera;
mod draw_batch;
mod pipelines;
mod render_engine;
mod scheduler;
mod sprite;
mod texture;
mod ui;
mod vertex;

use dialog::DialogBox;
use render_engine::RenderEngine;
use std::panic::catch_unwind;
use std::time::Instant;

use crate::asset_management::AssetLoader;
use crate::sprite::Sprite;
use log::{debug, error, info, trace, warn};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const TARGET_FPS: f64 = 144.0;
const TARGET_FRAMETIME: f64 = 1000.0 / TARGET_FPS;

fn main() {
    asset_management::KEEP_ASSET_NAMES.store(true, std::sync::atomic::Ordering::Relaxed);

    match catch_unwind(|| engine_main()) {
        Ok(_) => (),
        Err(e) => {
            let err = {
                match e.downcast_ref::<&'static str>() {
                    Some(x) => (*x).to_string(),
                    None => match e.downcast_ref::<String>() {
                        Some(x) => x.to_owned(),
                        None => "[Error could not be downcast to a string]".to_string(),
                    },
                }
            };
            let dialog_box = dialog::Message::new(format!(
                "The engine encountered a fatal error and had to exit: {}",
                err
            ));
            dialog_box.show();
        }
    }
}

fn engine_main() {
    pretty_env_logger::init();
    AssetLoader::add_archive("./res/redist/shaders.pak").unwrap();
    AssetLoader::add_archive("./res/redist/assets.pak").unwrap();

    let event_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut render_engine = RenderEngine::new(&window, &event_loop);
    let mut last_cache_clean = Instant::now();

    let tex = AssetLoader::load_texture("tux-32.png").unwrap();
    let sprite_id = render_engine.insert_sprite(Sprite::new(tex.clone(), [5.0, 5.0, 1.0]));
    render_engine.insert_sprite(Sprite::new(tex, [-150.0, -100.0, 0.0]));

    event_loop.run(move |event, _, control_flow| {
        render_engine.process_event(&event);

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => render_engine.resize(new_size),
                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    scale_factor,
                } => {
                    render_engine.resize(*new_inner_size);
                    render_engine.update_scale_factor(scale_factor);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let pos =
                        render_engine
                            .camera()
                            .mouse_pos_to_world_space(cgmath::Vector2::new(
                                position.x as f32,
                                position.y as f32,
                            ));
                    render_engine.get_sprite_mut(sprite_id).unwrap().position =
                        cgmath::Vector3::new(pos.x, pos.y, 0.0);
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                render_engine.update();
                match render_engine.render(&window) {
                    Ok(_) => (),
                    Err(wgpu::SurfaceError::Lost) => {
                        render_engine.reconfigure_surface();
                        warn!("wgpu::SurfaceError::Lost");
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = ControlFlow::Exit;
                        error!("wgpu::SurfaceError::OutOfMemory");
                    }
                    Err(e) => warn!("{}", e),
                }

                if last_cache_clean.elapsed().as_secs_f64() >= 10.0 {
                    AssetLoader::clean_cache();
                    last_cache_clean = Instant::now();
                }
            }
            Event::MainEventsCleared
            | Event::UserEvent(ui::integration::EguiRequestRedrawEvent::RequestRedraw) => {
                window.request_redraw()
            }
            _ => (),
        }
    })
}
