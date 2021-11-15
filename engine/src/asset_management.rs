use crate::scheduler::{Job, JobScheduler};
use crate::texture::GpuTexture;
use ahash::AHashMap;
use anyhow::{bail, Result};
use arc_swap::{ArcSwap, Guard};
use image::ImageFormat;
use lazy_static::lazy_static;
use log::{info, warn};
use parking_lot::Mutex;
use std::any::type_name;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;
use vach::archive::{Archive, HeaderConfig};
use vach::crypto::PublicKey;
use wgpu::{BindGroup, Device, Queue};

const NAMESPACE_ASSETS: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x15, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
];
pub const UUID_NAMESPACE_ASSETS: Uuid = Uuid::from_bytes(NAMESPACE_ASSETS);
const PUB_KEY: &[u8] = include_bytes!("../../res/keys/key.pub");

pub trait ToUuid {
    fn uuid(&self) -> Uuid {
        Uuid::new_v5(&UUID_NAMESPACE_ASSETS, self.type_name().as_bytes())
    }

    fn type_name(&self) -> String {
        type_name::<Self>().to_string()
    }
}

//pub enum GpuTextureRef {
//    Swappable(CachedGpuTexture),
//    Shared(Arc<GpuTexture>),
//}

pub enum GpuTextureRef {
    Swappable(Arc<ArcSwap<Uuid>>),
    Shared(Uuid),
}

impl GpuTextureRef {
    pub fn load(&self) -> Arc<GpuTexture> {
        AssetLoader::texture_from_cache(&self.uuid()).expect("Texture in cache")
    }
}

impl Clone for GpuTextureRef {
    fn clone(&self) -> Self {
        match self {
            GpuTextureRef::Swappable(x) => GpuTextureRef::Swappable(Arc::clone(x)),
            GpuTextureRef::Shared(x) => GpuTextureRef::Shared(x.clone()),
        }
    }
}

impl GpuTextureRef {
    pub fn uuid(&self) -> Uuid {
        match self {
            GpuTextureRef::Swappable(x) => **x.load(),
            GpuTextureRef::Shared(x) => x.clone(),
        }
    }
}

lazy_static! {
    static ref ASSET_LOADER: Mutex<AssetLoader> = Mutex::new(AssetLoader::init());
}

//pub type CachedGpuTexture = Arc<ArcSwap<GpuTexture>>;

pub struct AssetLoader {
    header_config: HeaderConfig,
    archives: AHashMap<Uuid, Archive<File>>,
    raw_cache: AHashMap<Uuid, Arc<Vec<u8>>>,
    tex_cache: AHashMap<Uuid, Arc<GpuTexture>>,
    tex_placeholder: Option<Arc<GpuTexture>>,
    tex_placeholder_uuid: Option<Uuid>,
}

impl AssetLoader {
    fn init() -> Self {
        Self {
            header_config: {
                let mut header_config = HeaderConfig::default();
                header_config.public_key =
                    Some(PublicKey::from_bytes(PUB_KEY).expect("a valid public key"));
                header_config
            },
            archives: AHashMap::new(),
            raw_cache: AHashMap::new(),
            tex_cache: AHashMap::new(),
            tex_placeholder: None,
            tex_placeholder_uuid: None,
        }
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
        Self::with_lock(|loader| {
            loader.tex_placeholder = Some(atex);
            loader.tex_placeholder_uuid = Some(uuid);
        });
        Ok(())
    }

    pub fn add_archive<T: AsRef<Path> + Into<PathBuf>>(path: T) -> Result<()> {
        let archive_path = path.into();
        let archive_file = File::open(&archive_path)?;
        let archive_name = archive_path.file_name().unwrap().to_string_lossy();
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, archive_name.as_bytes());

        {
            let mut lock = ASSET_LOADER.lock();
            let archive: Archive<File> = Archive::with_config(archive_file, &lock.header_config)?;
            lock.archives.insert(uuid, archive);
        }

        info!("Loaded archive {} with UUID {}", archive_name, uuid);

        Ok(())
    }

    fn with_lock<R, F: FnOnce(&mut AssetLoader) -> R>(fun: F) -> R {
        let mut lock = ASSET_LOADER.lock();
        fun(lock.deref_mut())
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

        Self::with_lock(|loader| match loader.raw_cache.insert(uuid, data) {
            Some(_) => warn!("Cache already contained an entry for {}", id),
            None => (),
        });

        return Ok(rdata);
    }

    /// doesn't insert into the cache
    fn get_asset_uncached(id: &str) -> Result<Vec<u8>> {
        info!("Loading asset {} without caching", id);
        match Self::with_lock(|loader| {
            for (archive_hash, archive) in loader.archives.iter_mut() {
                if let Ok(resource) = archive.fetch(id) {
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

        Self::with_lock(|loader| match loader.raw_cache.get(&uuid) {
            Some(x) => Some(Arc::clone(x)),
            None => None,
        })
    }

    pub fn clean_cache() {
        info!("Starting a cache clean");
        let mut to_remove = Vec::new();

        Self::with_lock(|loader| {
            for (uuid, asset) in loader.raw_cache.iter() {
                if Arc::strong_count(asset) <= 1 {
                    // there's no references to this data besides the one we have in the hashmap
                    // so, we get rid of it
                    to_remove.push(uuid.to_owned());
                }
            }

            for uuid in to_remove.iter() {
                loader.raw_cache.remove(uuid);
                info!("Removed {} from cache", uuid);
            }
        });

        info!(
            "Cache cleaning finished: removed {} asset(s)",
            to_remove.len()
        );
    }

    fn load_texture_inner(id: &str, format: Option<ImageFormat>) -> Result<GpuTextureRef> {
        match Self::load_texture_from_cache(id) {
            //Some(x) => return Ok(GpuTextureRef::Shared(x)),
            // TODO: optimize this
            Some(x) => return Ok(GpuTextureRef::Shared(x.uuid())),
            None => info!("Texture {} not in cache", id),
        }

        let placeholder_uuid = Self::with_lock(|loader| match &loader.tex_placeholder_uuid {
            Some(x) => x.clone(),
            None => panic!("No placeholder texture set!"),
        });

        let arc_uuid = Arc::new(ArcSwap::new(Arc::new(placeholder_uuid)));

        let job = TextureLoadJob {
            id: id.to_string(),
            format,
            tex: Arc::clone(&arc_uuid),
        };

        JobScheduler::submit(box job);
        Ok(GpuTextureRef::Swappable(arc_uuid))
    }

    pub fn load_texture_with_format(id: &str, format: ImageFormat) -> Result<GpuTextureRef> {
        Self::load_texture_inner(id, Some(format))
    }

    pub fn load_texture(id: &str) -> Result<GpuTextureRef> {
        Self::load_texture_inner(id, None)
    }

    fn load_texture_from_cache(id: &str) -> Option<Arc<GpuTexture>> {
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes());

        Self::with_lock(|loader| match loader.tex_cache.get(&uuid) {
            Some(x) => Some(Arc::clone(x)),
            None => None,
        })
    }

    fn insert_into_texture_cache(id: &str, texture: Arc<GpuTexture>) {
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes());

        Self::with_lock(|loader| match loader.tex_cache.insert(uuid, texture) {
            Some(_) => warn!("Cache already contained an entry for {}", id),
            None => (),
        });
    }

    pub fn texture_from_cache(uuid: &Uuid) -> Result<Arc<GpuTexture>> {
        Self::with_lock(|loader| match loader.tex_cache.get(uuid) {
            Some(x) => Ok(Arc::clone(x)),
            None => bail!("Texture not loaded!"),
        })
    }
}

struct TextureLoadJob {
    id: String,
    tex: Arc<ArcSwap<Uuid>>,
    format: Option<ImageFormat>,
}

impl ToUuid for TextureLoadJob {}

impl Job for TextureLoadJob {
    fn run(&mut self, device: &Device, queue: &Queue) -> Result<()> {
        let data = AssetLoader::get_asset_uncached(&self.id)?;
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, self.id.as_bytes());
        let texture = match self.format {
            Some(format) => GpuTexture::new_from_data_with_format(
                device,
                queue,
                &data,
                format,
                Some(&self.id),
                uuid,
            ),
            None => GpuTexture::new_from_data(device, queue, &data, Some(&self.id), uuid),
        }?;

        let cached_texture = Arc::new(texture);
        AssetLoader::insert_into_texture_cache(&self.id, cached_texture);
        self.tex.swap(Arc::new(uuid));
        Ok(())
    }
}
