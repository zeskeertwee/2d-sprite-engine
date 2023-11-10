use crate::asset_management::{AssetLoader, Uuid};
use crate::render_engine::texture::GpuTexture;
use arc_swap::ArcSwap;
use log::trace;
use std::sync::Arc;

pub enum GpuTextureRef {
    Swappable(Arc<ArcSwap<Uuid>>),
    Shared(Uuid),
}

impl GpuTextureRef {
    pub(super) fn new_shared(uuid: Uuid) -> Self {
        let v = GpuTextureRef::Shared(uuid);
        v.register();
        v
    }

    pub(super) fn new_swappable(uuid: Arc<ArcSwap<Uuid>>) -> Self {
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
