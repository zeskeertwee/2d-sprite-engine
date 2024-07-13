mod compile_job;
mod lvme;
pub(crate) mod systems;

use crate::asset_management::AssetLoader;
use crate::scheduler::JobScheduler;
use bevy_ecs::prelude::Component;
pub use compile_job::LuaPreCompileJob;
use log::{trace, warn};
use mlua::Lua;
use std::ops::Deref;
use std::sync::Arc;

// todo: use all files in lvme-impl folder

#[derive(Component)]
pub struct LuaScript {
    setup: bool,
    pub(self) script_id: String,
    pub(self) binary: Arc<Vec<u8>>,
}

impl LuaScript {
    pub fn new(script_id: &str) -> Self {
        let binary = get_lua_script_or_compile(script_id);

        LuaScript {
            setup: false,
            script_id: script_id.to_string(),
            binary,
        }
    }

    pub fn run_setup(&self, vm: &Lua) {
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
    trace!("Creating new lua VM");

    // sets up the rust side of the LVME
    lvme::insert_all_extensions(&vm).unwrap();

    for i in AssetLoader::list_archive_entries("lvme.pak").unwrap() {
        let binary = get_lua_script_or_compile(&i);
        vm.load(binary.deref()).exec().unwrap();
    }

    vm.set_compiler(compile_job::get_compiler());
    vm.sandbox(true).unwrap();

    vm
}

pub fn get_lua_script_or_compile(script_id: &str) -> Arc<Vec<u8>> {
    match AssetLoader::get_precompiled_lua_script(script_id) {
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
    }
}
