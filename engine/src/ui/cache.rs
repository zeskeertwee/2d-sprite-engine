use super::EguiWindow;
use crate::asset_management::GpuTextureRef;
use crate::asset_management::Uuid;
use crate::ui::MenuCategory;
use crate::AssetLoader;
use ahash::AHashMap;
use egui::Ui;
use wgpu::{Device, FilterMode};

pub struct CacheDebugUi {
    egui_textures: AHashMap<Uuid, egui::TextureId>,
}

impl CacheDebugUi {
    pub fn update(&mut self, device: &Device, render_pass: &mut egui_wgpu_backend::RenderPass) {
        puffin::profile_function!();
        let mut loaded_uuids = Vec::new();
        let mut to_remove = Vec::new();

        AssetLoader::with_loader(|loader| {
            for v in loader.tex_cache.iter() {
                let (uuid, tex) = (v.key(), v.value());
                if !self.egui_textures.contains_key(uuid) {
                    let tex_id = render_pass.egui_texture_from_wgpu_texture(
                        device,
                        tex,
                        FilterMode::Nearest,
                    );
                    self.egui_textures.insert(*uuid, tex_id);
                    loaded_uuids.push(*uuid);
                }
            }

            for uuid in self.egui_textures.keys() {
                if !loader.tex_cache.contains_key(uuid) {
                    to_remove.push(*uuid);
                }
            }
        });

        for uuid in loaded_uuids {
            GpuTextureRef::register_inner(&uuid);
        }

        for uuid in to_remove {
            self.egui_textures.remove(&uuid).unwrap();
        }
    }
}

impl Default for CacheDebugUi {
    fn default() -> Self {
        AssetLoader::add_to_active_cache_debug_ui();

        Self {
            egui_textures: AHashMap::new(),
        }
    }
}

impl Drop for CacheDebugUi {
    fn drop(&mut self) {
        AssetLoader::remove_from_active_cache_debug_ui();
    }
}

impl EguiWindow for CacheDebugUi {
    fn title(&self) -> &'static str {
        "Texture cache"
    }

    fn menu_category(&self) -> MenuCategory {
        MenuCategory::Debug
    }

    fn draw(&mut self, ui: &mut Ui) {
        puffin::profile_function!("CacheDebugUi");
        ui.vertical(|ui| {
            for (uuid, tex) in self.egui_textures.iter() {
                ui.horizontal(|ui| {
                    ui.image(*tex, [50.0, 50.0]);
                    ui.label(format!("{}", uuid));
                });
            }
        });
    }
}
