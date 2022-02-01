use lazy_static::lazy_static;
use wasmtime::{Config, Engine, Instance, Linker, Module, OptLevel, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;
use wasmtime_wasi::WasiCtx;

mod compile_job;
use crate::AssetLoader;
pub use compile_job::WASMPreCompileJob;

lazy_static! {
    static ref WASM_ENGINE: WASMEngine = WASMEngine::new();
}

pub struct WASMEngine {
    engine: Engine,
}

impl WASMEngine {
    pub fn new() -> Self {
        let engine = Engine::new(&get_engine_config()).expect("Failed to create engine");

        WASMEngine { engine }
    }

    pub fn precompile_script(wasm: &[u8]) -> anyhow::Result<Vec<u8>> {
        WASM_ENGINE.engine.precompile_module(wasm)
    }

    pub fn run_script(script_id: &str) {
        let binary = AssetLoader::get_precompiled_wasm_script(&script_id)
            .expect("WASM script to be precompiled");
        log::info!("Instantiating script...");

        let mut linker = Linker::new(&WASM_ENGINE.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s).expect("Failed to add WASI to linker");

        let module = unsafe { Module::deserialize(&WASM_ENGINE.engine, &*binary) }
            .expect("Failed to create module");
        let wasi = WasiCtxBuilder::new().inherit_stdout().build();
        let mut store = Store::new(&WASM_ENGINE.engine, wasi);
        linker
            .module(&mut store, "", &module)
            .expect("Failed to link module");

        log::info!("Script instantiated!");

        linker
            .get_default(&mut store, "")
            .unwrap()
            .typed::<(), (), _>(&store)
            .unwrap()
            .call(&mut store, ())
            .unwrap();
    }
}

fn get_engine_config() -> Config {
    let mut config = Config::new();
    config.consume_fuel(false);
    config.cranelift_opt_level(OptLevel::Speed);
    config.parallel_compilation(false);
    config
}
