mod compile_job;

use crate::asset_management::AssetLoader;
use crate::scheduler::JobScheduler;
pub use compile_job::LuaPreCompileJob;
use log::{error, warn};
use mlua::Lua;
use std::ops::Deref;
use std::sync::Arc;

pub struct LuaScript {
    script_id: String,
    binary: Arc<Vec<u8>>,
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
            script_id: script_id.to_string(),
            binary,
        }
    }

    pub fn run(&self) {
        let lua = Lua::new();
        lua.set_compiler(compile_job::get_compiler());
        lua.sandbox(true).unwrap();

        let chunk = lua.load(self.binary.deref());
        chunk.exec().unwrap();
    }
}
