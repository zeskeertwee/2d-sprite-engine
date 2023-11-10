use crate::asset_management::{AssetLoader, KEEP_ASSET_NAMES, UUID_NAMESPACE_ASSETS};
use std::any::type_name;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::atomic::Ordering;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Uuid {
    inner: ::uuid::Uuid,
}

impl Deref for Uuid {
    type Target = ::uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if KEEP_ASSET_NAMES.load(Ordering::Relaxed) {
            f.write_str(
                // TODO: prevent deadlocks
                AssetLoader::with_loader(|loader| match &loader.name_cache {
                    Some(x) => match x.get(&self.inner) {
                        Some(x) => x.to_string(),
                        None => self.inner.to_string(),
                    },
                    None => panic!("Expedted an initialized name cache"),
                })
                .as_str(),
            )
        } else {
            f.write_str(&self.inner.to_string())
        }
    }
}

impl Debug for Uuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner.to_string())
    }
}

impl Uuid {
    pub fn new_v5(namespace: &Uuid, name: &[u8]) -> Self {
        Self {
            inner: uuid::Uuid::new_v5(namespace, name),
        }
    }

    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {
            inner: uuid::Uuid::from_bytes(bytes),
        }
    }
}

pub trait ToUuid {
    fn uuid(&self) -> Uuid {
        Uuid::new_v5(&UUID_NAMESPACE_ASSETS, self.type_name().as_bytes())
    }

    fn type_name(&self) -> String {
        type_name::<Self>().to_string()
    }
}
