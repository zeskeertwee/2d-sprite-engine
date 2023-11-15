mod log;
mod print_hook;
mod time;

use ::log::{trace, warn};
use anyhow::{bail, Result};
use lazy_static::lazy_static;
use mlua::{Function, Lua, Table};
use std::error::Error;
use std::fmt::Debug;

lazy_static! {
    static ref EXTENSIONS: Vec<Box<dyn LuaVMExtension>> = vec![
        Box::new(log::LogExtension),
        Box::new(print_hook::PrintHookExtension),
        Box::new(time::TimeExtension)
    ];
}

pub trait LuaVMExtension: Sync {
    fn namespace(&self) -> &'static str;
    fn build_table<'a>(&'a self, lua: &'a Lua) -> Result<Table> {
        bail!("Not implemented!");
    }

    fn single_function<'a>(&'a self, lua: &'a Lua) -> Result<Function> {
        bail!("Not implemented!");
    }
}

pub fn insert_all_extensions(lua: &Lua) -> Result<()> {
    for ext in EXTENSIONS.iter() {
        match ext.build_table(lua) {
            Ok(t) => lua.globals().set(ext.namespace(), t)?,
            Err(_) => match ext.single_function(lua) {
                Ok(f) => lua.globals().set(ext.namespace(), f)?,
                Err(_) => warn!(
                    "LVME extension {} hasn't provided a table or single function!",
                    ext.namespace()
                ),
            },
        }

        trace!("Added LVME extension: {}", ext.namespace());
    }

    Ok(())
}

pub trait AnyhowResultToLuaResult<V> {
    fn to_mlua(self) -> Result<V, mlua::Error>
    where
        Self: Sized;
}

impl<V, E> AnyhowResultToLuaResult<V> for core::result::Result<V, E>
where
    E: Debug,
{
    fn to_mlua(self) -> Result<V, mlua::Error>
    where
        Self: Sized,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(mlua::Error::RuntimeError(format!("{:?}", e))),
        }
    }
}
