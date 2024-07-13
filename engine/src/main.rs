mod asset_management;
mod demo_sys;
mod ecs;
mod pipelines;
mod render_engine;
mod scheduler;
mod scripting;
mod sprite;
mod ui;

use crate::asset_management::AssetLoader;
use crate::ecs::EcsWorld;
use crate::render_engine::components::{position::Position, texture::Texture};
use crate::scripting::LuaScript;
use cgmath::{Vector2, Vector3};
use dialog::DialogBox;
use log::{error, trace};
use std::panic::catch_unwind;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

//const TARGET_FPS: f64 = 144.0;
//const TARGET_FRAMETIME: f64 = 1000.0 / TARGET_FPS;

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
            match dialog_box.show() {
                Ok(_) => (),
                Err(e) => error!("Error showing dialog box: {}", e),
            }
        }
    }
}

fn engine_main() {
    pretty_env_logger::init();
    puffin::set_scopes_on(true);

    let _srv = puffin_http::Server::new("0.0.0.0:1999").unwrap();
    trace!("Vach version: {}", vach::VERSION);

    AssetLoader::add_archive("./res/redist/shaders.pak").unwrap();
    AssetLoader::add_archive("./res/redist/assets.pak").unwrap();
    AssetLoader::add_archive("./res/redist/scripts.pak").unwrap();
    AssetLoader::add_archive("./res/redist/lvme.pak").unwrap();

    let event_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut world = EcsWorld::new();
    render_engine::ecs::init_renderer_resources_in_world(&mut world, window, &event_loop);
    render_engine::ecs::insert_renderer_systems_in_schedule(&mut world);
    demo_sys::initialize_in_world(&mut world);

    // this needs to run AFTER the job system is initialized in init_renderer_resources_in_world
    scripting::systems::initialize_systems_in_world(&mut world);

    let tex = AssetLoader::load_texture("tux-32.png").unwrap();

    for i in 0..8 {
        let angle = (i as f32 / 8.0) * 360.0;
        let offset = Vector2::new(angle.cos(), angle.sin()) * 150.0;
        world.insert_entity(|mut e| {
            e.insert(Position([5.0, 5.0, 0.0].into()));
            e.insert(Texture(tex.clone()));
            e.insert(demo_sys::MoveWithCursor { offset });
            e.id()
        });
    }

    let script = LuaScript::new("test-script.lua");

    world.insert_entity(|mut e| {
        e.insert(Position([5.0, 5.0, 1.0].into()));
        e.insert(Texture(tex.clone()));
        e.insert(demo_sys::MoveWithCursor {
            offset: Vector2::new(0.0, 0.0),
        });
        e.id()
    });
    world.insert_entity(|mut e| {
        e.insert(Position([-150.0, -100.0, 1.0].into()));
        e.insert(Texture(tex.clone()));
        e.insert(script);
        e.id()
    });

    let wid = world.get_render_engine(|e| e.window_id());

    log::info!("Entering main loop");
    event_loop.run(move |event, _, control_flow| {
        //render_engine.process_event(&event);
        world.get_render_engine(|mut e| e.process_event(&event));

        match event {
            Event::WindowEvent { event, window_id } if window_id == wid => match event {
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
                WindowEvent::KeyboardInput { input, .. } => {
                    world.update_keyboard_input(input);
                }
                WindowEvent::Resized(new_size) => {
                    world.get_render_engine(|mut e| e.resize(new_size))
                }
                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    scale_factor,
                } => {
                    world.get_render_engine(|mut e| {
                        e.resize(*new_inner_size);
                        e.update_scale_factor(scale_factor);
                    });
                }
                WindowEvent::CursorMoved { position, .. } => {
                    world
                        .update_cursor_position(Vector2::new(position.x as f32, position.y as f32));
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                puffin::GlobalProfiler::lock().new_frame();
                world.run_schedule();
            }
            Event::MainEventsCleared
            | Event::UserEvent(ui::integration::EguiRequestRedrawEvent::RequestRedraw) => {
                world.get_render_engine(|e| e.request_redraw());
            }
            _ => (),
        }
    })
}
