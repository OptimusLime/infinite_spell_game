//! Immutable presentation snapshots of the renderer's mutable per-view output.
use atelico_component::{
    atelico_paint::{Canvas, Filter},
    tiny_skia::{Pixmap, Transform},
};
use atelico_gpu::wgpu;
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

pub(super) fn project_root() -> PathBuf {
    let root = atelico_node_cache::shared().map_or_else(
        || std::env::current_dir().unwrap_or_default(),
        |c| c.root().to_path_buf(),
    );
    root.canonicalize().unwrap_or(root)
}
/// `texture` owns a complete, immutable picture; `art` keeps the original overlay/bar boundary.
pub(super) struct GpuPicture {
    pub texture: wgpu::Texture,
    pub art: [u32; 4],
    pub nearest: bool,
}
pub(super) struct Picture {
    pub gpu: Option<GpuPicture>,
    pub seen: Option<crate::hooks::Seen>,
    pixels: Mutex<Option<Arc<Pixmap>>>,
    size: (u32, u32),
    version: u64,
}
impl Picture {
    fn version() -> u64 {
        static VERSION: AtomicU64 = AtomicU64::new(1);
        VERSION.fetch_add(1, Ordering::Relaxed)
    }
    pub fn pixels_only(pixels: Pixmap) -> Self {
        let size = (pixels.width(), pixels.height());
        Self {
            gpu: None,
            seen: None,
            pixels: Mutex::new(Some(Arc::new(pixels))),
            size,
            version: Self::version(),
        }
    }
    pub fn snapshot(on: crate::world::OnGpu, w: u32, h: u32) -> Result<Self, String> {
        let g = atelico_gpu::gpu()?;
        let texture = if on.dst == [0, 0, w, h]
            && on.src == [0., 0., w as f32, h as f32]
            && (on.texture.width(), on.texture.height()) == (w, h)
        {
            // The renderer lends a reusable target. A GPU copy gives this frame ownership without a CPU trip.
            let texture = g.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("immutable World picture"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let mut enc = g.device.create_command_encoder(&Default::default());
            enc.copy_texture_to_texture(
                on.texture.as_image_copy(),
                texture.as_image_copy(),
                texture.size(),
            );
            g.queue.submit([enc.finish()]);
            texture
        } else {
            // One new target both owns and materializes the pixel-lens grid; no preceding redundant copy.
            atelico_gpu::region::NearestRegion::new(w, h)?.draw(&on.texture, on.src, on.dst)?
        };
        Ok(Self {
            gpu: Some(GpuPicture {
                texture,
                art: on.dst,
                nearest: on.nearest,
            }),
            seen: None,
            pixels: Mutex::new(None),
            size: (w, h),
            version: Self::version(),
        })
    }
    pub fn pixels(&self) -> Result<Arc<Pixmap>, String> {
        let mut pixels = self.pixels.lock().unwrap();
        if let Some(p) = &*pixels {
            return Ok(Arc::clone(p));
        }
        let on = self
            .gpu
            .as_ref()
            .ok_or("picture has no pixels or texture")?;
        let bytes = atelico_gpu::read_texture(atelico_gpu::gpu()?, &on.texture)?;
        let p = Arc::new(crate::pixels_of(&bytes, self.size.0, self.size.1)?);
        *pixels = Some(Arc::clone(&p));
        Ok(p)
    }
    pub fn draw(
        &self,
        canvas: &mut dyn Canvas,
        transform: Transform,
        filter: Filter,
    ) -> Result<(), String> {
        if let Some(gpu) = &self.gpu {
            if canvas.draw_texture(&gpu.texture, transform, filter, 1.) {
                return Ok(());
            }
        }
        let pixels = self.pixels()?;
        canvas.draw_pixmap(&pixels, self.version, transform, filter, 1.);
        Ok(())
    }
    fn reserved_bytes(&self) -> u64 {
        self.size.0 as u64 * self.size.1 as u64 * 8
    } // GPU plus optional CPU capture
}
/// Up to eight recent requests and 64 MiB of retained picture/capture storage. Renderer targets have their own bound.
pub(super) struct Cache<K> {
    entries: VecDeque<(K, Arc<Picture>)>,
}
pub(super) trait ViewKey: PartialEq {
    fn same_view(&self, other: &Self) -> bool;
}
impl<K: ViewKey> Cache<K> {
    pub const fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }
    pub fn get_or_render(
        &mut self,
        key: K,
        render: impl FnOnce() -> Result<Picture, String>,
    ) -> Result<Arc<Picture>, String> {
        if let Some(i) = self.entries.iter().position(|(k, _)| *k == key) {
            let e = self.entries.remove(i).unwrap();
            let p = Arc::clone(&e.1);
            self.entries.push_back(e);
            return Ok(p);
        }
        let picture = Arc::new(render()?);
        let size = picture.reserved_bytes();
        const BUDGET: u64 = 64 * 1024 * 1024;
        if size <= BUDGET {
            self.entries.retain(|(k, _)| !key.same_view(k));
            while self.entries.len() >= 8
                || self
                    .entries
                    .iter()
                    .map(|(_, p)| p.reserved_bytes())
                    .sum::<u64>()
                    + size
                    > BUDGET
            {
                self.entries.pop_front();
            }
            self.entries.push_back((key, Arc::clone(&picture)));
        }
        Ok(picture)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(PartialEq)]
    struct Key(u8, u8);
    impl ViewKey for Key {
        fn same_view(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    fn image() -> Picture {
        Picture::pixels_only(Pixmap::new(8, 8).unwrap())
    }
    #[test]
    fn cache_bounds_views_revisions_and_never_publishes_a_failed_revision() {
        let mut cache = Cache::new();
        let original = cache.get_or_render(Key(0, 0), || Ok(image())).unwrap();
        let version = original.version;
        let hit = cache
            .get_or_render(Key(0, 0), || panic!("cache hit must not render"))
            .unwrap();
        assert!(Arc::ptr_eq(&original, &hit));
        assert!(cache
            .get_or_render(Key(0, 1), || Err("invalid source".into()))
            .is_err());
        assert_eq!(cache.entries.len(), 1);
        let next = cache.get_or_render(Key(0, 1), || Ok(image())).unwrap();
        assert_ne!(next.version, version);
        assert_eq!(
            cache.entries.len(),
            1,
            "replace older revision of the same view"
        );
        for i in 1..12 {
            cache.get_or_render(Key(i, 0), || Ok(image())).unwrap();
        }
        assert_eq!(cache.entries.len(), 8);
        assert_eq!(original.version, version);
        assert_eq!(
            original.pixels().unwrap().width(),
            8,
            "eviction cannot invalidate a held frame"
        );
        let mut budget = Cache::new();
        for i in 0..6 {
            budget
                .get_or_render(Key(i, 0), || {
                    Ok(Picture::pixels_only(Pixmap::new(1024, 2048).unwrap()))
                })
                .unwrap();
        }
        assert_eq!(
            budget.entries.len(),
            4,
            "64 MiB counts the optional capture too"
        );
    }
}
