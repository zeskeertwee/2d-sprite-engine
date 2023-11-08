use crate::asset_management::ToUuid;
use crate::scheduler::{Job, JobFrequency};
use crate::AssetLoader;
use mlua;
use std::ops::Deref;
use wgpu::{Device, Queue};

pub struct LuaPreCompileJob {
    // the asset id of the script to precompile
    script_id: String,
}

impl ToUuid for LuaPreCompileJob {}

impl Job for LuaPreCompileJob {
    fn get_freq(&self) -> JobFrequency {
        JobFrequency::Once
    }

    fn run(&mut self, _: &Device, _: &Queue) -> anyhow::Result<()> {
        puffin::profile_scope!("LuaPreCompileJob", &self.script_id);
        let script = AssetLoader::get_asset(&self.script_id)?;
        let compiled = get_compiler().compile(script.deref());
        AssetLoader::add_compiled_lua_script(&self.script_id, compiled);
        Ok(())
    }
}

impl LuaPreCompileJob {
    pub fn new(script_id: &str) -> Self {
        LuaPreCompileJob {
            script_id: script_id.to_string(),
        }
    }
}

pub(super) fn get_compiler() -> mlua::Compiler {
    mlua::Compiler::new()
        .set_optimization_level(1)
        .set_coverage_level(2)
        .set_debug_level(2)
}
