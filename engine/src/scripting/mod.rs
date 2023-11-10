mod compile_job;
mod lvme;
pub(crate) mod systems;

use crate::asset_management::AssetLoader;
use crate::scheduler::JobScheduler;
use bevy_ecs::prelude::Component;
pub use compile_job::LuaPreCompileJob;
use log::warn;
use mlua::Lua;
use std::ops::Deref;
use std::sync::Arc;

// todo: use all files in lvme-impl folder
const LVME_IMPLEMENTATION: &'static str = include_str!("../../lvme-impl/deltatime.lua");

#[derive(Component)]
pub struct LuaScript {
    setup: bool,
    pub(self) script_id: String,
    pub(self) binary: Arc<Vec<u8>>,
}

impl LuaScript {
    pub fn new(script_id: &str) -> Self {
        let binary = match AssetLoader::get_precompiled_lua_script(script_id) {
            Some(v) => v,
            None => {
                warn!("Lua script {} not precompiled!", script_id);
                let job = JobScheduler::submit(Box::new(LuaPreCompileJob::new(script_id)));
                match job.flush() {
                    Ok(_) => (),
                    Err(e) => panic!("Failed to compile script: {:?}", e),
                }

                AssetLoader::get_precompiled_lua_script(script_id).unwrap()
            }
        };

        LuaScript {
            setup: false,
            script_id: script_id.to_string(),
            binary,
        }
    }

    pub fn run_setup(&self, vm: &Lua) {
        vm.load(LVME_IMPLEMENTATION).exec().unwrap();

        vm.load(self.binary.deref()).exec().unwrap();
        vm.load(r#"setup()"#).exec().unwrap();
    }

    pub fn run_in_vm(&mut self, vm: &Lua) {
        if !self.setup {
            self.run_setup(vm);
            self.setup = true;
        }

        vm.load(r#"update()"#).exec().unwrap();
    }
}

pub fn create_lua_vm() -> Lua {
    let vm = Lua::new();
    lvme::insert_all_extensions(&vm).unwrap();
    vm.set_compiler(compile_job::get_compiler());
    vm.sandbox(true).unwrap();

    vm
}
