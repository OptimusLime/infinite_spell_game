use std::collections::HashMap;
use std::path::PathBuf;

use atelico_component::tiny_skia::{Color, Mask, Pixmap};
use atelico_component::{parse_color, Rect, Surface, Value};
use fontdue::{Font, FontSettings, Metrics};

static DEFAULT: &[u8] = include_bytes!("../fonts/Inter-Regular.ttf");

pub struct TextSpec {
    pub text: String,
    font: usize,
    size: f32,
    color: Color,
    wrap: bool,
    align: Align,
}

#[derive(Clone, Copy, PartialEq)]
enum Align {
    Start,
    Center,
    End,
}

impl TextSpec {
    pub fn of(node: &Value, fonts: &mut Fonts, s: &Surface) -> Self {
        let st = node.get("style").filter(|v| v.is_object()).cloned().unwrap_or(Value::Null);
        let font = st.get("font").and_then(Value::as_str).and_then(|p| s.asset(p)).map_or(0, |p| fonts.load(p));
        TextSpec {
            text: node.get("text").and_then(Value::as_str).unwrap_or("").to_string(),
            font,
            size: st.get("font_size").and_then(Value::as_f64).unwrap_or(18.0) as f32,
            color: st.get("color").and_then(Value::as_str).and_then(parse_color).unwrap_or(Color::WHITE),
            wrap: st.get("text_wrap").and_then(Value::as_str) != Some("none"),
            align: match st.get("text_align").and_then(Value::as_str) {
                Some("center") => Align::Center,
                Some("right" | "end") => Align::End,
                _ => Align::Start,
            },
        }
    }

    pub fn line_height(&self) -> f32 {
        (self.size * 1.25).ceil()
    }
}

pub struct Fonts {
    fonts: Vec<Font>,
    paths: HashMap<PathBuf, usize>,
    glyphs: HashMap<(usize, char, u32), (Metrics, Vec<u8>)>,
}

impl Default for Fonts {
    fn default() -> Self {
        let inter = Font::from_bytes(DEFAULT, FontSettings::default()).expect("bundled font");
        Self { fonts: vec![inter], paths: HashMap::new(), glyphs: HashMap::new() }
    }
}

impl Fonts {
    fn load(&mut self, path: PathBuf) -> usize {
        if let Some(i) = self.paths.get(&path) {
            return *i;
        }
        let i = match std::fs::read(&path).ok().and_then(|b| Font::from_bytes(b, FontSettings::default()).ok()) {
            Some(f) => {
                self.fonts.push(f);
                self.fonts.len() - 1
            }
            None => 0,
        };
        self.paths.insert(path, i);
        i
    }

    fn width(&self, font: usize, s: &str, size: f32) -> f32 {
        let f = &self.fonts[font];
        s.chars().map(|c| f.metrics(c, size).advance_width).sum()
    }

    /// Lines and their widths for text laid into `max` logical px.
    pub fn wrap(&self, t: &TextSpec, max: f32) -> Vec<(String, f32)> {
        let mut lines = Vec::new();
        for para in t.text.split('\n') {
            if !t.wrap {
                lines.push((para.to_string(), self.width(t.font, para, t.size)));
                continue;
            }
            let mut line = String::new();
            for word in para.split_inclusive(' ') {
                let candidate = format!("{line}{word}");
                if !line.is_empty() && self.width(t.font, candidate.trim_end(), t.size) > max + 0.5 {
                    let done = line.trim_end().to_string();
                    let w = self.width(t.font, &done, t.size);
                    lines.push((done, w));
                    line = word.to_string();
                } else {
                    line = candidate;
                }
            }
            let done = line.trim_end().to_string();
            let w = self.width(t.font, &done, t.size);
            lines.push((done, w));
        }
        lines
    }

    pub fn draw(&mut self, pm: &mut Pixmap, scale: f32, t: &TextSpec, r: Rect, clip: Option<&Mask>, caret: bool) {
        let lines = self.wrap(t, r.w);
        let lh = t.line_height();
        let ascent = self.fonts[t.font].horizontal_line_metrics(t.size).map_or(t.size * 0.8, |m| m.ascent);
        let top = r.y + ((lh - t.size * 1.2) / 2.0).max(0.0);
        let mut end = (r.x, top);
        for (i, (line, w)) in lines.iter().enumerate() {
            let x = match t.align {
                Align::Start => r.x,
                Align::Center => r.x + (r.w - w) / 2.0,
                Align::End => r.x + r.w - w,
            };
            let baseline = top + i as f32 * lh + ascent;
            let mut pen = x * scale;
            for ch in line.chars() {
                let (m, bitmap) = self.glyph(t.font, ch, t.size * scale);
                let gx = (pen + m.xmin as f32).round() as i32;
                let gy = (baseline * scale).round() as i32 - m.height as i32 - m.ymin;
                blit(pm, gx, gy, m.width, m.height, &bitmap, t.color, clip);
                pen += m.advance_width;
            }
            end = (pen / scale, top + i as f32 * lh);
        }
        if caret {
            let mut p = atelico_component::tiny_skia::Paint::default();
            p.set_color(t.color);
            if let Some(rect) = atelico_component::tiny_skia::Rect::from_xywh(end.0 + 1.0, end.1 + 2.0, 2.0, lh - 4.0) {
                pm.fill_rect(rect, &p, atelico_component::tiny_skia::Transform::from_scale(scale, scale), clip);
            }
        }
    }

    fn glyph(&mut self, font: usize, ch: char, px: f32) -> (Metrics, Vec<u8>) {
        let key = (font, ch, px.to_bits());
        let fonts = &self.fonts;
        self.glyphs
            .entry(key)
            .or_insert_with(|| {
                let f = if fonts[font].lookup_glyph_index(ch) != 0 { &fonts[font] } else { &fonts[0] };
                f.rasterize(ch, px)
            })
            .clone()
    }
}

#[allow(clippy::too_many_arguments)]
fn blit(pm: &mut Pixmap, x0: i32, y0: i32, w: usize, h: usize, coverage: &[u8], color: Color, clip: Option<&Mask>) {
    let (pw, ph) = (pm.width() as i32, pm.height() as i32);
    let c = color.to_color_u8();
    let mask = clip.map(|m| m.data());
    let data = pm.data_mut();
    for row in 0..h as i32 {
        let y = y0 + row;
        if y < 0 || y >= ph {
            continue;
        }
        for col in 0..w as i32 {
            let x = x0 + col;
            if x < 0 || x >= pw {
                continue;
            }
            let mut a = coverage[(row * w as i32 + col) as usize] as u32 * c.alpha() as u32 / 255;
            if let Some(m) = mask {
                a = a * m[(y * pw + x) as usize] as u32 / 255;
            }
            if a == 0 {
                continue;
            }
            let i = ((y * pw + x) * 4) as usize;
            let src = [c.red(), c.green(), c.blue()];
            for k in 0..3 {
                data[i + k] = ((src[k] as u32 * a + data[i + k] as u32 * (255 - a)) / 255) as u8;
            }
            data[i + 3] = (a + data[i + 3] as u32 * (255 - a) / 255) as u8;
        }
    }
}
