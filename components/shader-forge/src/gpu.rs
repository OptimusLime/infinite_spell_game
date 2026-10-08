//! Offscreen wgpu on the shared device (`atelico-gpu`): a compiled fragment shader drawn over a w x h RGBA
//! texture and read back, so the pixels land in the same tiny-skia frame as everything else (window, screenshots
//! and recordings all see them).
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

use atelico_gpu::wgpu;

pub use atelico_gpu::png;

struct Pipelines {
    pipeline_layout: wgpu::PipelineLayout,
    uniforms: wgpu::Buffer,
    bind: wgpu::BindGroup,
    by_source: HashMap<u64, wgpu::RenderPipeline>,
}

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// time, aspect, 2 pad, then 16 vec4 slots of animated params.
const UNIFORM_BYTES: u64 = 16 + 16 * 16;

fn open() -> Result<Pipelines, String> {
    let device = &atelico_gpu::gpu()?.device;
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("shader-forge uniforms"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        }],
    });
    let uniforms = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: UNIFORM_BYTES, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniforms.as_entire_binding() }] });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[&layout], push_constant_ranges: &[] });
    Ok(Pipelines { pipeline_layout, uniforms, bind, by_source: HashMap::new() })
}

fn pipelines() -> &'static Mutex<Result<Pipelines, String>> {
    static P: OnceLock<Mutex<Result<Pipelines, String>>> = OnceLock::new();
    P.get_or_init(|| Mutex::new(open()))
}

fn hash(s: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

impl Pipelines {
    fn pipeline(&mut self, device: &wgpu::Device, wgsl: &str) -> &wgpu::RenderPipeline {
        if self.by_source.len() > 64 {
            self.by_source.clear();
        }
        let layout = &self.pipeline_layout;
        self.by_source.entry(hash(wgsl)).or_insert_with(|| {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(wgsl.into()) });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shader-forge"),
                layout: Some(layout),
                vertex: wgpu::VertexState { module: &module, entry_point: Some("vs"), buffers: &[], compilation_options: Default::default() },
                fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("fs"), targets: &[Some(FORMAT.into())], compilation_options: Default::default() }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        })
    }
}

/// Draws `wgsl` (already validated) at `time` with animated `params` (slot i → `u.p[i].x`) into w x h
/// premultiplied RGBA8 pixels, row-major, top row first.
pub fn render(wgsl: &str, w: u32, h: u32, time: f32, params: &[f32]) -> Result<Vec<u8>, String> {
    let g = atelico_gpu::gpu()?;
    let mut guard = pipelines().lock().unwrap();
    let p = guard.as_mut().map_err(|e| e.clone())?;
    let (w, h) = (w.max(1), h.max(1));
    let mut bytes = [time.to_le_bytes(), (w as f32 / h as f32).to_le_bytes(), [0; 4], [0; 4]].concat();
    for i in 0..16 {
        bytes.extend(params.get(i).copied().unwrap_or(0.0).to_le_bytes());
        bytes.extend([0u8; 12]);
    }
    g.queue.write_buffer(&p.uniforms, 0, &bytes);
    let texture = g.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    p.pipeline(&g.device, wgsl);
    let pipeline = &p.by_source[&hash(wgsl)];
    let mut enc = g.device.create_command_encoder(&Default::default());
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &p.bind, &[]);
        pass.draw(0..3, 0..1);
    }
    atelico_gpu::read(g, enc, &texture, w, h)
}
