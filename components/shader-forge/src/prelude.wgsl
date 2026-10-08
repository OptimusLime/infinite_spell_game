// time, aspect; p: the animated params (a timeline track each), `u.p[i].x`
struct Uniforms { time: f32, aspect: f32, pad0: f32, pad1: f32, p: array<vec4<f32>, 16> };
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

// one triangle over the target; uv (0,0) bottom-left, (1,1) top-right
@vertex fn vs(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    var o: VOut;
    o.pos = vec4<f32>(p[i], 0.0, 1.0);
    o.uv = p[i] * 0.5 + 0.5;
    return o;
}

fn hash2(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 456.21));
    q = q + dot(q, q + 45.32);
    return fract(q.x * q.y);
}

fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash2(i);
    let b = hash2(i + vec2<f32>(1.0, 0.0));
    let c = hash2(i + vec2<f32>(0.0, 1.0));
    let d = hash2(i + vec2<f32>(1.0, 1.0));
    let s = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, s.x), mix(c, d, s.x), s.y);
}

fn fbm(p: vec2<f32>, octaves: f32) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var q = p;
    for (var i = 0; i < 8; i = i + 1) {
        if (f32(i) >= octaves) { break; }
        v = v + a * vnoise(q);
        q = q * 2.03 + vec2<f32>(1.7, 9.2);
        a = a * 0.5;
    }
    return v;
}

fn ramp4(x: f32, c0: vec3<f32>, c1: vec3<f32>, c2: vec3<f32>, c3: vec3<f32>) -> vec3<f32> {
    let t = clamp(x, 0.0, 1.0) * 3.0;
    if (t < 1.0) { return mix(c0, c1, t); }
    if (t < 2.0) { return mix(c1, c2, t - 1.0); }
    return mix(c2, c3, t - 2.0);
}

fn grad_vertical(uv: vec2<f32>) -> f32 { return uv.y; }
fn grad_horizontal(uv: vec2<f32>) -> f32 { return uv.x; }
fn grad_radial(uv: vec2<f32>) -> f32 { return clamp(1.0 - length(uv - vec2<f32>(0.5, 0.5)) * 2.0, 0.0, 1.0); }


// ---- compositing: `graph` gives straight rgba in linear light (HDR, as the 3D renderer takes it); the preview
// shows it as the engine displays it (preview_look), and draws are composited premultiplied ----
// the renderer's tonemap (a filmic shoulder on the brightest channel, hue kept), then sRGB for the screen
fn preview_look(c: vec3<f32>) -> vec3<f32> {
    let v = max(c, vec3<f32>(0.0));
    let m = max(max(v.r, v.g), v.b);
    var t = v;
    if m > 0.72 { t = v * ((0.72 + 0.28 * (1.0 - exp(-(m - 0.72) / 0.28))) / m); }
    return select(1.055 * pow(t, vec3<f32>(1.0 / 2.4)) - 0.055, t * 12.92, t <= vec3<f32>(0.0031308));
}
fn over(acc: vec4<f32>, top: vec4<f32>) -> vec4<f32> {
    let a = clamp(top.a, 0.0, 1.0);
    let rgb = clamp(preview_look(top.rgb), vec3<f32>(0.0), vec3<f32>(1.0)) * a;
    return vec4<f32>(rgb + acc.rgb * (1.0 - a), a + acc.a * (1.0 - a));
}

// straight-alpha "over" inside a graph (the layer node)
fn layer_over(base: vec4<f32>, top: vec4<f32>) -> vec4<f32> {
    let a = top.a + base.a * (1.0 - top.a);
    let rgb = (top.rgb * top.a + base.rgb * base.a * (1.0 - top.a)) / max(a, 0.0001);
    return vec4<f32>(rgb, a);
}
fn layer_add(base: vec4<f32>, top: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(base.rgb + top.rgb * top.a, max(base.a, top.a));
}

fn out_clip(c: vec4<f32>) -> vec4<f32> { return vec4<f32>(c.rgb, step(0.5, c.a)); }
fn out_blend(c: vec4<f32>) -> vec4<f32> { return c; }

// ---- painted noise and motion: faceted value noise, a centred point, a swirl, flat tonal bands ----
// (the flame's own stages are the fire package's: packages/fire/src/flame.wgsl)
fn hash1(p: vec2<f32>) -> f32 { return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453); }
fn smooth_noise(p: vec2<f32>) -> f32 {
    let i = floor(p); let f = fract(p); let s = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash1(i), hash1(i + vec2<f32>(1.0, 0.0)), s.x), mix(hash1(i + vec2<f32>(0.0, 1.0)), hash1(i + 1.0), s.x), s.y);
}
// linear, not smoothstep, interpolation: faceted, painted
fn angular(p: vec2<f32>) -> f32 {
    let i = floor(p); let f = fract(p);
    return mix(mix(hash1(i), hash1(i + vec2<f32>(1.0, 0.0)), f.x), mix(hash1(i + vec2<f32>(0.0, 1.0)), hash1(i + 1.0), f.x), f.y);
}
fn center_p(uv: vec2<f32>) -> vec2<f32> { return vec2<f32>((uv.x - 0.5) * 2.0, uv.y); }
fn swirl(p: vec2<f32>, t: f32, amount: f32, center: vec2<f32>, falloff: f32) -> vec2<f32> {
    let d = p - center;
    let angle = sin(t * 1.7 - p.y * 4.0) * exp(-dot(d, d) * falloff) * amount * 2.3;
    return vec2<f32>(cos(angle) * d.x - sin(angle) * d.y, sin(angle) * d.x + cos(angle) * d.y) + center;
}
fn angular_noise(p: vec2<f32>, t: f32, scale: f32, stretch: f32, scroll: vec2<f32>, seed: f32) -> f32 {
    let q = p * vec2<f32>(scale, scale * stretch) - scroll * t + vec2<f32>(seed * 13.7, seed * 7.1);
    return 0.62 * angular(q) + 0.28 * angular(q * 2.03 + 7.3) + 0.10 * smooth_noise(q * 4.1 - 3.8);
}
fn band_ramp(h: f32, c0: vec3<f32>, c1: vec3<f32>, c2: vec3<f32>, c3: vec3<f32>, t1: f32, t2: f32, t3: f32, soft: f32) -> vec3<f32> {
    var rgb = mix(c0, c1, smoothstep(t1 - soft, t1 + soft, h));
    rgb = mix(rgb, c2, smoothstep(t2 - soft, t2 + soft, h));
    return mix(rgb, c3, smoothstep(t3 - soft, t3 + soft, h));
}

// ---- shapes: a point placed into a shape's own space, a flat ellipse ----
fn place(p: vec2<f32>, at: vec2<f32>, size: vec2<f32>) -> vec2<f32> {
    return (p - at) / max(size, vec2<f32>(0.001, 0.001));
}
fn ellipse(p: vec2<f32>, at: vec2<f32>, radius: vec2<f32>, soft: f32) -> f32 {
    let r = length((p - at) / max(radius, vec2<f32>(0.001, 0.001)));
    return 1.0 - smoothstep(1.0 - soft, 1.0 + soft, r);
}

// ---- ink ----
// a band of ink just inside an edge (`depth` 0 on the edge): the cel outline
fn ink_rim(color: vec3<f32>, depth: f32, width: f32, ink: vec3<f32>) -> vec3<f32> {
    return mix(ink, color, step(width, depth));
}
