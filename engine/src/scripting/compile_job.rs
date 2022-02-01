use crate::asset_management::ToUuid;
use crate::scheduler::Job;
use crate::AssetLoader;
use wgpu::{Device, Queue};

pub struct WASMPreCompileJob {
    // the asset id of the script to precompile
    script_id: String,
}

impl ToUuid for WASMPreCompileJob {}

impl Job for WASMPreCompileJob {
    fn run(&mut self, _: &Device, _: &Queue) -> anyhow::Result<()> {
        let wasm = AssetLoader::get_asset(&self.script_id)?;
        let wasm_compiled = super::WASMEngine::precompile_script(&wasm)?;
        AssetLoader::add_compiled_wasm_script(&self.script_id, wasm_compiled);
        Ok(())
    }
}

impl WASMPreCompileJob {
    pub fn new(script_id: &str) -> Self {
        WASMPreCompileJob {
            script_id: script_id.to_string(),
        }
    }
}
