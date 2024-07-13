mod cache_clean_job;
mod gpu_texture_ref;
mod texture_load_job;
mod uuid;

pub use cache_clean_job::CacheCleanJob;
pub use gpu_texture_ref::GpuTextureRef;
use texture_load_job::TextureLoadJob;
pub use uuid::{ToUuid, Uuid};

use crate::render_engine::texture::GpuTexture;
use crate::scheduler::JobScheduler;
use anyhow::{bail, Result};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use image::ImageFormat;
use lazy_static::lazy_static;
use log::{info, warn};
use parking_lot::Mutex;
use std::fmt::Display;
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use vach::archive::{Archive, HeaderConfig};
use vach::crypto::PublicKey;
use wgpu::{Device, Queue};

const NAMESPACE_ASSETS: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x15, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
];
pub const UUID_NAMESPACE_ASSETS: Uuid = Uuid::from_bytes(NAMESPACE_ASSETS);
const PUB_KEY: &[u8] = include_bytes!("../../../res/keys/key.pub");
pub static KEEP_ASSET_NAMES: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref ASSET_LOADER: AssetLoader = AssetLoader::init();
}

pub struct AssetLoader {
    pub(crate) name_cache: Option<DashMap<::uuid::Uuid, String>>,
    pub(crate) active_cache_debug_ui: AtomicU8,
    pub(crate) header_config: Arc<HeaderConfig>,
    pub(crate) archives: DashMap<Uuid, Mutex<Archive<File>>>,
    pub(crate) raw_cache: DashMap<Uuid, Arc<Vec<u8>>>,
    pub(crate) tex_cache: DashMap<Uuid, Arc<GpuTexture>>,
    pub(crate) tex_placeholder: ArcSwap<Option<Arc<GpuTexture>>>,
    pub(crate) tex_placeholder_uuid: ArcSwap<Option<Uuid>>,
    pub(crate) lua_script_cache: DashMap<Uuid, Arc<Vec<u8>>>,
}

impl AssetLoader {
    fn init() -> Self {
        Self {
            active_cache_debug_ui: AtomicU8::new(0),
            name_cache: if KEEP_ASSET_NAMES.load(Ordering::Relaxed) {
                Some(DashMap::new())
            } else {
                None
            },
            header_config: Arc::new({
                let mut header_config = HeaderConfig::default();
                header_config.public_key =
                    Some(PublicKey::from_bytes(PUB_KEY).expect("a valid public key"));
                header_config
            }),
            archives: DashMap::new(),
            raw_cache: DashMap::new(),
            tex_cache: DashMap::new(),
            tex_placeholder: ArcSwap::new(Arc::new(None)),
            tex_placeholder_uuid: ArcSwap::new(Arc::new(None)),
            lua_script_cache: DashMap::new(),
        }
    }

    pub(crate) fn add_to_active_cache_debug_ui() {
        Self::with_loader(|loader| {
            loader.active_cache_debug_ui.fetch_add(1, Ordering::Relaxed);
        })
    }

    pub(crate) fn remove_from_active_cache_debug_ui() {
        Self::with_loader(|loader| {
            loader.active_cache_debug_ui.fetch_sub(1, Ordering::Relaxed);
        })
    }

    pub fn set_tex_placeholder(
        device: &Device,
        queue: &Queue,
        id: &str,
        format: ImageFormat,
    ) -> Result<()> {
        let data = Self::get_asset_uncached(id)?;
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes());
        let texture = GpuTexture::new_from_data_with_format(
            device,
            queue,
            data.deref(),
            format,
            Some(id),
            uuid,
        )?;
        let atex = Arc::new(texture);
        Self::insert_into_texture_cache(id, Arc::clone(&atex));
        Self::with_loader(|loader| {
            loader.tex_placeholder.store(Arc::new(Some(atex)));
            loader.tex_placeholder_uuid.store(Arc::new(Some(uuid)));
        });
        Ok(())
    }

    pub fn add_archive<T: AsRef<Path> + Into<PathBuf>>(path: T) -> Result<()> {
        let archive_path = path.into();
        let archive_file = File::open(&archive_path)?;
        let archive_name = archive_path.file_name().unwrap().to_string_lossy();
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, archive_name.as_bytes());

        let header_config = Self::with_loader(|loader| Arc::clone(&loader.header_config));
        let archive = Archive::with_config(archive_file, &header_config)?;

        for (name, _) in archive.entries() {
            log::trace!("{} contains {}", archive_name, name);
        }

        Self::with_loader(|loader| loader.archives.insert(uuid, Mutex::new(archive)));
        Self::insert_asset_name(archive_name.to_string().as_str());

        info!("Loaded archive {} with UUID {}", archive_name, uuid);

        Ok(())
    }
    
    pub fn list_archive_entries(id: &str) -> Option<Vec<String>> {
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes());
        Self::with_loader(|loader| {
            match loader.archives.get(&uuid) {
                Some(x) => {
                    let mut entries = Vec::new();
                    for (name, _) in x.lock().entries() {
                        entries.push(name.to_string());
                    }
                    Some(entries)
                },
                None => None,
            }
        })
    }

    pub(crate) fn with_loader<R, F: FnOnce(&AssetLoader) -> R>(fun: F) -> R {
        //let start = Instant::now();
        //let mut lock = ASSET_LOADER.lock();
        //if start.elapsed().as_secs_f64() * 1000.0 > 1.2 {
        //    log::warn!(
        //        "Asset loader lock held for more than 1.2 ms: held for {:?}",
        //        start.elapsed()
        //    );
        //}
        //fun(lock.deref_mut())

        fun(&ASSET_LOADER)
    }

    pub fn get_asset(id: &str) -> Result<Arc<Vec<u8>>> {
        match Self::get_asset_from_raw_cache(id) {
            Some(x) => {
                info!(
                    "Asset {} loaded from cache, strong_count: {}",
                    id,
                    Arc::strong_count(&x)
                );
                return Ok(x);
            }
            None => info!("Asset {} not in cache", id),
        }

        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes());
        let data = Arc::new(Self::get_asset_uncached(id)?);
        let rdata = Arc::clone(&data);

        Self::with_loader(|loader| match loader.raw_cache.insert(uuid, data) {
            Some(_) => warn!("Cache already contained an entry for {}", id),
            None => (),
        });
        Self::insert_asset_name(id);

        return Ok(rdata);
    }

    /// doesn't insert into the cache
    fn get_asset_uncached(id: &str) -> Result<Vec<u8>> {
        info!("Loading asset {} without caching", id);
        match Self::with_loader(|loader| {
            for archive in loader.archives.iter() {
                if let Ok(resource) = archive.lock().fetch(id) {
                    if !resource.secured {
                        warn!("Resource {} isn't secured!", id);
                    }

                    return Some(resource.data);
                }
            }

            None
        }) {
            Some(x) => Ok(x),
            None => bail!("Asset {} not present in any loaded archive!", id),
        }
    }

    pub fn get_asset_from_raw_cache(id: &str) -> Option<Arc<Vec<u8>>> {
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes());

        Self::with_loader(|loader| match loader.raw_cache.get(&uuid) {
            Some(x) => Some(Arc::clone(x.value())),
            None => None,
        })
    }

    pub fn clean_cache() {
        puffin::profile_function!();
        info!("Starting a cache clean");
        let count = Self::with_loader(|loader| {
            let mut count = 0;
            count += clean_cache_inner(
                &loader.tex_cache,
                1 + loader.active_cache_debug_ui.load(Ordering::Relaxed) as usize,
            );
            count + clean_cache_inner(&loader.raw_cache, 1)
        });

        info!("Removed {} items from cache", count);
    }

    fn load_texture_inner(id: &str, format: Option<ImageFormat>) -> Result<GpuTextureRef> {
        match Self::load_texture_from_cache(id) {
            // TODO: optimize this
            Some(x) => return Ok(GpuTextureRef::new_shared(x.uuid())),
            None => info!("Texture {} not in cache", id),
        }

        let placeholder_uuid =
            Self::with_loader(
                |loader| match loader.tex_placeholder_uuid.load().deref().deref() {
                    Some(x) => x.clone(),
                    None => panic!("No placeholder texture set!"),
                },
            );

        let arc_uuid = Arc::new(ArcSwap::new(Arc::new(placeholder_uuid)));
        let tex_ref = GpuTextureRef::new_swappable(arc_uuid);

        let job = TextureLoadJob {
            id: id.to_string(),
            format,
            tex: tex_ref.clone(),
        };

        JobScheduler::submit(Box::new(job));
        Ok(tex_ref)
    }

    pub fn load_texture_with_format(id: &str, format: ImageFormat) -> Result<GpuTextureRef> {
        Self::load_texture_inner(id, Some(format))
    }

    pub fn load_texture(id: &str) -> Result<GpuTextureRef> {
        Self::load_texture_inner(id, None)
    }

    fn load_texture_from_cache(id: &str) -> Option<Arc<GpuTexture>> {
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes());

        Self::with_loader(|loader| match loader.tex_cache.get(&uuid) {
            Some(x) => Some(Arc::clone(x.value())),
            None => None,
        })
    }

    fn insert_into_texture_cache(id: &str, texture: Arc<GpuTexture>) {
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes());

        Self::with_loader(|loader| match loader.tex_cache.insert(uuid, texture) {
            Some(_) => warn!("Cache already contained an entry for {}", id),
            None => (),
        });
        Self::insert_asset_name(id);
    }

    fn insert_asset_name(id: &str) {
        if KEEP_ASSET_NAMES.load(Ordering::Relaxed) {
            Self::with_loader(|loader| match &loader.name_cache {
                Some(x) => {
                    x.insert(
                        ::uuid::Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes()),
                        id.to_string(),
                    );
                }
                None => panic!("KEEP_ASSET_NAMES is set to true but no cache was initialized"),
            })
        }
    }

    pub fn texture_from_cache(uuid: &Uuid) -> Result<Arc<GpuTexture>> {
        Self::with_loader(|loader| match loader.tex_cache.get(uuid) {
            Some(x) => Ok(Arc::clone(x.value())),
            None => bail!("Texture not loaded!"),
        })
    }

    pub(crate) fn add_compiled_lua_script(id: &str, data: Vec<u8>) {
        Self::with_loader(|loader| {
            loader.lua_script_cache.insert(
                Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes()),
                Arc::new(data),
            );
        });
    }

    pub(crate) fn get_precompiled_lua_script(id: &str) -> Option<Arc<Vec<u8>>> {
        Self::with_loader(|loader| {
            match loader
                .lua_script_cache
                .get(&Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes()))
            {
                Some(x) => Some(Arc::clone(x.value())),
                None => None,
            }
        })
    }
}

fn clean_cache_inner<T>(cache: &DashMap<Uuid, Arc<T>>, max_strong_ref: usize) -> usize {
    puffin::profile_function!();
    let mut to_remove = Vec::new();

    for v in cache.iter() {
        let (uuid, asset) = (v.key(), v.value());
        if Arc::strong_count(asset) <= max_strong_ref {
            // there's no references to this data besides the one we have in the hashmap
            // so, we get rid of it
            to_remove.push(uuid.to_owned());
        }
    }

    for uuid in to_remove.iter() {
        let r = cache.get(&uuid).unwrap();
        let arc = r.value();
        for _ in 0..Arc::strong_count(&arc) - 1 {
            unsafe { Arc::decrement_strong_count(Arc::as_ptr(&arc)) };
        }
        assert_eq!(Arc::strong_count(&arc), 1);
        cache.remove(uuid);
        info!("Removed {:?} from cache", uuid);
    }

    to_remove.len()
}
