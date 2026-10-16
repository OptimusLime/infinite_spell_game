//! Typed preview frames: GPU texture first; CPU pixels only for a backend/capture that needs them.
use crate::gpu;
use atelico_component::atelico_paint::{Filter, Transform};
use atelico_component::tiny_skia::{FilterQuality, Pixmap, PixmapPaint};
use atelico_component::{num, On, Surface, Value};
use atelico_gpu::wgpu;
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct FrameKey {
    id: String,
    source: String,
    parameter_bits: Vec<u32>,
    milliseconds: u32,
    width: u32,
    height: u32,
}
struct Preview {
    key: FrameKey,
    time: f32,
}
impl Preview {
    fn of(node: &Value, rect: atelico_component::Rect, scale: f32, now: f64) -> Option<Self> {
        let source = node.get("wgsl")?.as_str()?.to_owned();
        let time = num(node, "time", now as f32);
        let res = scale.min(1.).max(0.5);
        Some(Self {
            key: FrameKey {
                id: node
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("preview")
                    .to_owned(),
                source,
                parameter_bits: node
                    .get("params")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_f64)
                            .map(|v| (v as f32).to_bits())
                            .collect()
                    })
                    .unwrap_or_default(),
                milliseconds: (time * 1000.) as u32,
                width: ((rect.w * res) as u32).clamp(1, 640),
                height: ((rect.h * res) as u32).clamp(1, 640),
            },
            time,
        })
    }
    fn frame(self) -> Result<Arc<Frame>, String> {
        static FRAMES: Mutex<Frames> = Mutex::new(Frames::new());
        FRAMES.lock().unwrap().get(self)
    }
}
struct Frame {
    texture: wgpu::Texture,
    pixels: Mutex<Option<Pixmap>>,
    version: u64,
}
impl Frame {
    fn with_pixels<T>(&self, f: impl FnOnce(&Pixmap) -> T) -> Result<T, String> {
        let mut pixels = self.pixels.lock().unwrap();
        if pixels.is_none() {
            let bytes = atelico_gpu::read_texture(atelico_gpu::gpu()?, &self.texture)?;
            *pixels = Pixmap::from_vec(
                bytes,
                tiny_size(self.texture.width(), self.texture.height())?,
            );
        }
        let pm = pixels.as_ref().ok_or("shader preview pixels unavailable")?;
        Ok(f(pm))
    }
    fn transform(&self, rect: atelico_component::Rect, scale: f32) -> Transform {
        Transform::from_translate(rect.x, rect.y)
            .post_scale(scale, scale)
            .pre_scale(
                rect.w / self.texture.width() as f32,
                rect.h / self.texture.height() as f32,
            )
    }
}
fn tiny_size(w: u32, h: u32) -> Result<atelico_component::tiny_skia::IntSize, String> {
    atelico_component::tiny_skia::IntSize::from_wh(w, h)
        .ok_or_else(|| "shader preview has no size".into())
}
struct Frames {
    entries: VecDeque<(FrameKey, Arc<Frame>)>,
}
impl Frames {
    const fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }
    fn get(&mut self, preview: Preview) -> Result<Arc<Frame>, String> {
        if let Some(index) = self.entries.iter().position(|(k, _)| *k == preview.key) {
            let item = self.entries.remove(index).expect("located frame");
            let frame = Arc::clone(&item.1);
            self.entries.push_back(item);
            return Ok(frame);
        }
        let k = &preview.key;
        let params: Vec<f32> = k
            .parameter_bits
            .iter()
            .map(|v| f32::from_bits(*v))
            .collect();
        let texture = gpu::render_texture(&k.source, k.width, k.height, preview.time, &params)?;
        static VERSION: AtomicU64 = AtomicU64::new(1);
        let frame = Arc::new(Frame {
            texture,
            pixels: Mutex::new(None),
            version: VERSION.fetch_add(1, Ordering::Relaxed),
        });
        if self.entries.len() == 16 {
            self.entries.pop_front();
        }
        self.entries.push_back((preview.key, Arc::clone(&frame)));
        Ok(frame)
    }
}
fn handles(node: &Value) -> bool {
    node.get("kind").and_then(Value::as_str) == Some("shader_preview")
}
pub(crate) fn paint(node: &Value, s: &mut Surface<'_>) -> bool {
    if !handles(node) {
        return false;
    }
    let Some(p) = Preview::of(node, s.rect, s.scale, s.time) else {
        return true;
    };
    let Ok(frame) = p.frame() else {
        return true;
    };
    let t = frame.transform(s.rect, s.scale);
    let _ = frame.with_pixels(|pm| {
        s.pixmap.draw_pixmap(
            0,
            0,
            pm.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Bilinear,
                ..Default::default()
            },
            t,
            s.clip,
        )
    });
    true
}
pub(crate) fn draw(node: &Value, on: &mut On<'_>) -> bool {
    if !handles(node) {
        return false;
    }
    let Some(p) = Preview::of(node, on.rect, on.canvas.scale(), on.time) else {
        return true;
    };
    let Ok(frame) = p.frame() else {
        return true;
    };
    let t = frame.transform(on.rect, on.canvas.scale());
    if !on
        .canvas
        .draw_texture(&frame.texture, t, Filter::Bilinear, 1.)
    {
        let _ = frame.with_pixels(|pm| {
            on.canvas
                .draw_pixmap(pm, frame.version, t, Filter::Bilinear, 1.)
        });
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_frame_key_keeps_uniform_bits_and_original_resolution_time_rules() {
        let rect = atelico_component::Rect {
            x: 1.25,
            y: 2.5,
            w: 900.,
            h: 400.,
        };
        let mut node =
            serde_json::json!({"wgsl":"source", "id":"same", "time":1.0001,"params":[0.0]});
        let a = Preview::of(&node, rect, 2., 0.).unwrap();
        node["params"] = serde_json::json!([-0.0]);
        let b = Preview::of(&node, rect, 2., 0.).unwrap();
        assert_ne!(
            a.key, b.key,
            "bit-level uniforms may distinguish signed zero"
        );
        assert_eq!((a.key.width, a.key.height), (640, 400));
        assert_eq!(a.key.milliseconds, 1000);
        node["params"] = serde_json::json!([0.0]);
        node["time"] = serde_json::json!(1.0009);
        assert_eq!(
            a.key,
            Preview::of(&node, rect, 2., 0.).unwrap().key,
            "preserve existing millisecond cache bucket"
        );
    }
}
