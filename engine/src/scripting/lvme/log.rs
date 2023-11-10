use super::LuaVMExtension;
use anyhow::Result;
use log::{debug, error, info, trace, warn};
use mlua::{Lua, Table};

pub struct LogExtension;

impl LuaVMExtension for LogExtension {
    fn namespace(&self) -> &'static str {
        "log"
    }

    fn build_table<'a>(&'a self, lua: &'a Lua) -> Result<Table> {
        let lua_log_trace_fn = lua.create_function(|_, msg: String| {
            trace!(target: "lvme::log", "{}", msg);
            Ok(())
        })?;

        let lua_log_debug_fn = lua.create_function(|_, msg: String| {
            debug!(target: "lvme::log", "{}", msg);
            Ok(())
        })?;

        let lua_log_info_fn = lua.create_function(|_, msg: String| {
            info!(target: "lvme::log", "{}", msg);
            Ok(())
        })?;

        let lua_log_warn_fn = lua.create_function(|_, msg: String| {
            warn!(target: "lvme::log", "{}", msg);
            Ok(())
        })?;

        let lua_log_err_fn = lua.create_function(|_, msg: String| {
            error!(target: "lvme::log", "{}", msg);
            Ok(())
        })?;

        let table = lua.create_table()?;
        table.set("trace", lua_log_trace_fn)?;
        table.set("debug", lua_log_debug_fn)?;
        table.set("info", lua_log_info_fn)?;
        table.set("warn", lua_log_warn_fn)?;
        table.set("error", lua_log_err_fn)?;

        Ok(table)
    }
}
