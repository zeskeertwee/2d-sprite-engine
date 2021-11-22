use crate::scheduler::{Job, JobScheduler};
use crate::texture::GpuTexture;
use ahash::AHashMap;
use anyhow::{bail, Result};
use arc_swap::ArcSwap;
use image::ImageFormat;
use lazy_static::lazy_static;
use log::{info, trace, warn};
use parking_lot::Mutex;
use std::any::type_name;
use std::fmt::{Debug, Display, Formatter, Write};
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use vach::archive::{Archive, HeaderConfig};
use vach::crypto::PublicKey;
use wgpu::{Device, Queue};

const NAMESPACE_ASSETS: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x15, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
];
pub const UUID_NAMESPACE_ASSETS: Uuid = Uuid::from_bytes(NAMESPACE_ASSETS);
const PUB_KEY: &[u8] = include_bytes!("../../res/keys/key.pub");
pub static KEEP_ASSET_NAMES: AtomicBool = AtomicBool::new(false);

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Uuid {
    inner: uuid::Uuid,
}

impl Deref for Uuid {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if KEEP_ASSET_NAMES.load(Ordering::Relaxed) {
            f.write_str(
                AssetLoader::with_lock(|loader| match &loader.name_cache {
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

pub enum GpuTextureRef {
    Swappable(Arc<ArcSwap<Uuid>>),
    Shared(Uuid),
}

impl GpuTextureRef {
    fn new_shared(uuid: Uuid) -> Self {
        let v = GpuTextureRef::Shared(uuid);
        v.register();
        v
    }

    fn new_swappable(uuid: Arc<ArcSwap<Uuid>>) -> Self {
        let v = GpuTextureRef::Swappable(uuid);
        v.register();
        v
    }

    pub(crate) fn swap(&self, new_uuid: Arc<Uuid>) {
        match self {
            GpuTextureRef::Shared(_) => panic!("Attempt to swap shared GpuTextureRef"),
            GpuTextureRef::Swappable(x) => {
                trace!("Swapping GpuTextureRef");

                let count = Arc::strong_count(&x);
                // we need to register and deregister multiple times because there are multiple references to a swappable GpuTextureRef

                for _ in 0..count {
                    self.deregister();
                }

                x.swap(new_uuid);

                for _ in 0..count {
                    self.register();
                }
            }
        }
    }

    pub fn load(&self) -> Arc<GpuTexture> {
        AssetLoader::texture_from_cache(&self.uuid()).expect("Texture in cache")
    }

    fn register(&self) {
        Self::register_inner(&self.uuid());
    }

    fn deregister(&self) {
        Self::deregister_inner(&self.uuid())
    }

    pub(crate) fn register_inner(uuid: &Uuid) {
        trace!("Registring {}", uuid);
        let arc = AssetLoader::texture_from_cache(uuid).unwrap();
        unsafe { Arc::increment_strong_count(Arc::as_ptr(&arc)) }
    }

    pub(crate) fn deregister_inner(uuid: &Uuid) {
        trace!("Deregistring {}", uuid);
        let arc = AssetLoader::texture_from_cache(uuid)
            .expect(format!("Texture {} to be loaded", uuid).as_str());
        unsafe { Arc::decrement_strong_count(Arc::as_ptr(&arc)) }
    }
}

impl Drop for GpuTextureRef {
    fn drop(&mut self) {
        trace!(
            "Drop GpuTextureRef {}, count {}",
            self.uuid(),
            Arc::strong_count(&self.load())
        );
        if Arc::strong_count(&self.load()) > 0 {
            self.deregister()
        }
    }
}

impl Clone for GpuTextureRef {
    fn clone(&self) -> Self {
        trace!("Cloning GpuTextureRef");
        let v = match self {
            GpuTextureRef::Swappable(x) => GpuTextureRef::Swappable(Arc::clone(x)),
            GpuTextureRef::Shared(x) => GpuTextureRef::Shared(x.clone()),
        };

        v.register();
        v
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

pub struct AssetLoader {
    pub(crate) name_cache: Option<AHashMap<uuid::Uuid, String>>,
    pub(crate) active_cache_debug_ui: u8,
    pub(crate) header_config: Arc<HeaderConfig>,
    pub(crate) archives: AHashMap<Uuid, Archive<File>>,
    pub(crate) raw_cache: AHashMap<Uuid, Arc<Vec<u8>>>,
    pub(crate) tex_cache: AHashMap<Uuid, Arc<GpuTexture>>,
    pub(crate) tex_placeholder: Option<Arc<GpuTexture>>,
    pub(crate) tex_placeholder_uuid: Option<Uuid>,
}

impl AssetLoader {
    fn init() -> Self {
        Self {
            active_cache_debug_ui: 0,
            name_cache: if KEEP_ASSET_NAMES.load(Ordering::Relaxed) {
                Some(AHashMap::new())
            } else {
                None
            },
            header_config: Arc::new({
                let mut header_config = HeaderConfig::default();
                header_config.public_key =
                    Some(PublicKey::from_bytes(PUB_KEY).expect("a valid public key"));
                header_config
            }),
            archives: AHashMap::new(),
            raw_cache: AHashMap::new(),
            tex_cache: AHashMap::new(),
            tex_placeholder: None,
            tex_placeholder_uuid: None,
        }
    }

    pub(crate) fn add_to_active_cache_debug_ui() {
        Self::with_lock(|loader| {
            loader.active_cache_debug_ui += 1;
        })
    }

    pub(crate) fn remove_from_active_cache_debug_ui() {
        Self::with_lock(|loader| {
            loader.active_cache_debug_ui -= 1;
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

        let header_config = Self::with_lock(|loader| Arc::clone(&loader.header_config));
        let archive = Archive::with_config(archive_file, &header_config)?;
        Self::with_lock(|loader| loader.archives.insert(uuid, archive));
        Self::insert_asset_name(archive_name.to_string().as_str());

        info!("Loaded archive {} with UUID {}", archive_name, uuid);

        Ok(())
    }

    pub(crate) fn with_lock<R, F: FnOnce(&mut AssetLoader) -> R>(fun: F) -> R {
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
        Self::insert_asset_name(id);

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
        let count = Self::with_lock(|loader| {
            let mut count = 0;
            count += clean_cache_inner(
                &mut loader.tex_cache,
                1 + loader.active_cache_debug_ui as usize,
            );
            count + clean_cache_inner(&mut loader.raw_cache, 1)
        });

        info!("Removed {} items from cache", count);
    }

    fn load_texture_inner(id: &str, format: Option<ImageFormat>) -> Result<GpuTextureRef> {
        match Self::load_texture_from_cache(id) {
            // TODO: optimize this
            Some(x) => return Ok(GpuTextureRef::new_shared(x.uuid())),
            None => info!("Texture {} not in cache", id),
        }

        let placeholder_uuid = Self::with_lock(|loader| match &loader.tex_placeholder_uuid {
            Some(x) => x.clone(),
            None => panic!("No placeholder texture set!"),
        });

        let arc_uuid = Arc::new(ArcSwap::new(Arc::new(placeholder_uuid)));
        let tex_ref = GpuTextureRef::new_swappable(arc_uuid);

        let job = TextureLoadJob {
            id: id.to_string(),
            format,
            tex: tex_ref.clone(),
        };

        JobScheduler::submit(box job);
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
        Self::insert_asset_name(id);
    }

    fn insert_asset_name(id: &str) {
        if KEEP_ASSET_NAMES.load(Ordering::Relaxed) {
            Self::with_lock(|loader| match &mut loader.name_cache {
                Some(x) => {
                    x.insert(
                        uuid::Uuid::new_v5(&UUID_NAMESPACE_ASSETS, id.as_bytes()),
                        id.to_string(),
                    );
                }
                None => panic!("KEEP_ASSET_NAMES is set to true but no cache was initialized"),
            })
        }
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
    tex: GpuTextureRef,
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

fn clean_cache_inner<T>(cache: &mut AHashMap<Uuid, Arc<T>>, max_strong_ref: usize) -> usize {
    let mut to_remove = Vec::new();

    for (uuid, asset) in cache.iter() {
        if Arc::strong_count(asset) <= max_strong_ref {
            // there's no references to this data besides the one we have in the hashmap
            // so, we get rid of it
            to_remove.push(uuid.to_owned());
        }
    }

    for uuid in to_remove.iter() {
        let arc = cache.get(&uuid).unwrap();
        for _ in 0..Arc::strong_count(&arc) - 1 {
            unsafe { Arc::decrement_strong_count(Arc::as_ptr(&arc)) };
        }
        assert_eq!(Arc::strong_count(&arc), 1);
        cache.remove(uuid);
        info!("Removed {:?} from cache", uuid);
    }

    to_remove.len()
}
