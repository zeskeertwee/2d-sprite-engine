use crate::ecs::resources::DeltaTime;
use crate::scripting::LuaScript;
use bevy_ecs::system::{NonSend, Query, Res};
use log::trace;
use mlua::Lua;

pub fn update_scripts(vm: NonSend<Lua>, dt: Res<DeltaTime>, mut scripts: Query<(&mut LuaScript)>) {
    puffin::profile_function!();
    // first update the VM with the new updated status
    trace!("set dt {}", dt.as_secs_f64());
    // vm.globals()
    //     .set("__deltatime_seconds", dt.as_secs_f64())
    //     .unwrap();
    vm.load(format!("__deltatime_seconds = {}", dt.as_secs_f64()))
        .exec()
        .unwrap();

    for mut script in scripts.iter_mut() {
        script.run_in_vm(&vm);
    }
}
