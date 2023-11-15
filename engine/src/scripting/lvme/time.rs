use super::LuaVMExtension;
use anyhow::Result;
use mlua::{Lua, Table};

pub struct TimeExtension;

impl LuaVMExtension for TimeExtension {
    fn namespace(&self) -> &'static str {
        "time"
    }

    fn build_table<'a>(&'a self, lua: &'a Lua) -> Result<Table> {
        let lua_dt_fn = lua.create_function(|vm, ()| {
            let dt_s = vm.named_registry_value::<f64>("deltatime_s").unwrap();

            Ok(dt_s)
        })?;

        // TODO: Maybe implement in Lua?
        let lua_dt_ms_fn = lua.create_function(|vm, ()| {
            let dt_s = vm.named_registry_value::<f64>("deltatime_s").unwrap();

            Ok(dt_s * 1000.0)
        })?;

        let table = lua.create_table()?;
        table.set("deltatime", lua_dt_fn)?;
        table.set("deltatime_ms", lua_dt_ms_fn)?;

        Ok(table)
    }
}
