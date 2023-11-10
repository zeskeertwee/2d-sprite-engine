use crate::render_engine::resources::FrameResources;
use crate::render_engine::RenderEngineResources;
use bevy_ecs::system::{Commands, Res, ResMut};
use log::{error, info};
use wgpu::{SurfaceError, TextureViewDescriptor};

pub fn ecs_render_create_frame_resource(
    mut commands: Commands,
    engine: Res<RenderEngineResources>,
    //mut frame: ResMut<Option<FrameResources>>,
) {
    puffin::profile_function!();
    let output = {
        puffin::profile_scope!("get_surface_texture");
        match engine.surface.get_current_texture() {
            Ok(v) => v,
            Err(SurfaceError::Lost) => {
                info!("Got SurfaceError::Lost, reconfiguring surface");
                engine.reconfigure_surface();
                return;
            }
            Err(SurfaceError::OutOfMemory) => {
                error!("Got SurfaceError::OutOfMemory");
                panic!("Got out of memory error!");
            }
            Err(e) => {
                error!("Got {}, skipping frame", e);
                return;
            }
        }
    };

    let view = {
        puffin::profile_scope!("create_output_view");
        output
            .texture
            .create_view(&TextureViewDescriptor::default())
    };

    //*frame = Some(FrameResources { output, view });
    commands.insert_resource(FrameResources {
        output: Some(output),
        view,
    });
}
