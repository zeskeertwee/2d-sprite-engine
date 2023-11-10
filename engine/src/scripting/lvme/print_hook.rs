use crate::scripting::lvme::{AnyhowResultToLuaResult, LuaVMExtension};
use log::trace;
use mlua::{Function, Lua, Table};

pub struct PrintHookExtension;

impl LuaVMExtension for PrintHookExtension {
    fn namespace(&self) -> &'static str {
        "print"
    }

    fn single_function<'a>(&'a self, lua: &'a Lua) -> anyhow::Result<Function> {
        Ok(lua
            .create_function(|_, msg: String| {
                trace!(target: "lvme::print_hook", "{}", msg);
                Ok(())
            })
            .to_mlua()?)
    }
}
