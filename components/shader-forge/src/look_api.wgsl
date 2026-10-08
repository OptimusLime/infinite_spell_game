
// ---- the look API's stand-ins: a look graph (a post pass, its output a `look_output`) calls the renderer's
// `wl_*` stages (crates/render-3d/src/look_api.wgsl), which read the frame it drew. Standing alone (a preview with no
// scene, a validation) they draw a plain test card instead. In function mode these lines are left out and the calls
// go to the renderer's own. One line each. ----
fn wl_scene(uv: vec2<f32>) -> vec3<f32> { return mix(vec3<f32>(0.18, 0.22, 0.4), vec3<f32>(0.9, 0.7, 0.45), smoothstep(0.2, 0.8, uv.y)) * (0.6 + 0.4 * step(0.5, fract((floor(uv.x * 8.0) + floor(uv.y * 8.0)) * 0.5))); }
fn wl_surface(uv: vec2<f32>) -> f32 { return step(uv.y, 0.6); }
fn wl_pixelate(uv: vec2<f32>, texels: f32) -> vec2<f32> { let s = vec2<f32>(320.0, 180.0) / max(texels, 1.0); return (floor(uv * s) + 0.5) / s; }
fn wl_ink(c: vec3<f32>, uv: vec2<f32>, width: f32, strength: f32, ink: vec3<f32>, fade: f32, self_tone: f32) -> vec3<f32> { return mix(c, ink, strength * step(abs(uv.y - 0.6), 0.004 * max(width, 1.0))); }
fn wl_ao(c: vec3<f32>, uv: vec2<f32>, strength: f32, threshold: f32) -> vec3<f32> { return c; }
fn wl_bands(c: vec3<f32>, steps: f32, amount: f32) -> vec3<f32> { let m = max(max(c.r, c.g), max(c.b, 0.0001)); let n = max(steps, 1.0); return mix(c, c * (pow((floor(pow(min(m, 4.0), 1.0 / 2.2) * n) + 0.5) / n, 2.2) / m), amount); }
fn wl_rim(c: vec3<f32>, uv: vec2<f32>, width: f32, colour: vec3<f32>, strength: f32) -> vec3<f32> { return c + colour * strength * smoothstep(1.0 - width, 1.0, abs(uv.x * 2.0 - 1.0)) * step(uv.y, 0.6); }
fn wl_sky(c: vec3<f32>, uv: vec2<f32>, top: vec3<f32>, horizon: vec3<f32>, amount: f32) -> vec3<f32> { return mix(c, mix(horizon, top, smoothstep(0.0, 1.0, uv.y)), amount * step(0.6, uv.y)); }
fn wl_exposure(c: vec3<f32>, exposure: f32) -> vec3<f32> { return c * exposure; }
fn wl_bloom(c: vec3<f32>, uv: vec2<f32>, threshold: f32, amount: f32) -> vec3<f32> { return c + max(c - vec3<f32>(threshold), vec3<f32>(0.0)) * amount; }
fn wl_tonemap(c: vec3<f32>, amount: f32) -> vec3<f32> { return mix(c, c / (vec3<f32>(1.0) + c), amount); }
fn wl_display(c: vec3<f32>) -> vec3<f32> { return clamp(pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2)), vec3<f32>(0.0), vec3<f32>(1.0)); }
fn wl_look(shown: vec3<f32>, strength: f32) -> vec3<f32> { return shown; }
fn wl_saturation(shown: vec3<f32>, s: f32) -> vec3<f32> { let l = dot(shown, vec3<f32>(0.2126, 0.7152, 0.0722)); return clamp(mix(vec3<f32>(l), shown, s), vec3<f32>(0.0), vec3<f32>(1.0)); }
fn wl_bayer(uv: vec2<f32>) -> f32 { let q = vec2<u32>(floor(uv * vec2<f32>(320.0, 180.0))) % vec2<u32>(2u); return f32(q.x * 2u + q.y) / 4.0 - 0.375; }
fn wl_steps(shown: vec3<f32>, uv: vec2<f32>, steps: f32, dither: f32) -> vec3<f32> { let m = max(max(shown.r, shown.g), max(shown.b, 0.0001)); return clamp(shown * (floor(m * max(steps, 1.0) + 0.5 + wl_bayer(uv) * dither) / max(steps, 1.0) / m), vec3<f32>(0.0), vec3<f32>(1.0)); }
fn wl_palette(shown: vec3<f32>, uv: vec2<f32>, dither: f32, amount: f32) -> vec3<f32> { return mix(shown, floor(shown * 6.0 + 0.5 + wl_bayer(uv) * dither) / 6.0, amount); }
