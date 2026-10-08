use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use atelico_component::tiny_skia::Color;
use atelico_component::{parse_color, Rect, Value};
use atelico_paint::{Canvas, GlyphMask, Paint};
use fontdue::{Font, FontSettings, Metrics};

static DEFAULT: &[u8] = include_bytes!("../fonts/Inter-Regular.ttf");

pub struct TextSpec {
    pub text: String,
    font: usize,
    size: f32,
    color: Color,
    pub wrap: bool,
    align: Align,
    /// `text_overflow = "ellipsis"`: a line too long for its box is cut short and ends in "…".
    pub ellipsis: bool,
    /// An input's value: set on the face's cap band, so typing never moves the line.
    pub editable: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Align {
    Start,
    Center,
    End,
}

impl TextSpec {
    pub fn of(node: &Value, fonts: &mut Fonts, assets: &[PathBuf]) -> Self {
        let st = node.get("style").filter(|v| v.is_object()).cloned().unwrap_or(Value::Null);
        let font = st.get("font").and_then(Value::as_str).and_then(|p| atelico_component::asset(assets, p)).map_or(0, |p| fonts.load(p));
        TextSpec {
            text: node.get("text").and_then(Value::as_str).unwrap_or("").to_string(),
            font,
            size: st.get("font_size").and_then(Value::as_f64).unwrap_or(18.0) as f32,
            color: st.get("color").and_then(Value::as_str).and_then(parse_color).unwrap_or(Color::WHITE),
            wrap: st.get("text_wrap").and_then(Value::as_str) != Some("none"),
            ellipsis: st.get("text_overflow").and_then(Value::as_str) == Some("ellipsis"),
            align: match st.get("text_align").and_then(Value::as_str) {
                Some("center") => Align::Center,
                Some("right" | "end") => Align::End,
                _ => Align::Start,
            },
            editable: false,
        }
    }

    /// Whether `other` takes the same room as this (what a layout measures: the words, face, size, wrapping), whatever
    /// its colour or alignment.
    pub fn measures_as(&self, other: &TextSpec) -> bool {
        self.text == other.text && self.font == other.font && self.size == other.size && self.wrap == other.wrap && self.ellipsis == other.ellipsis && self.editable == other.editable
    }

    /// Whether `other` is set in the same face and size, wrapping and cut the same way (its words aside).
    pub fn same_face(&self, other: &TextSpec) -> bool {
        self.font == other.font && self.size == other.size && self.wrap == other.wrap && self.ellipsis == other.ellipsis && self.editable == other.editable
    }

    /// Its text cut short to fit (ending in "…"): when it was cut, the line is set in the middle of its box.
    pub fn centre_cut(&mut self, cut: String) {
        if cut != self.text && self.align == Align::Start {
            self.align = Align::Center;
        }
        self.text = cut;
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn line_height(&self) -> f32 {
        (self.size * 1.25).ceil()
    }
}

/// A face's cap band, measured once from its 'H' glyph: how far its ink reaches above the baseline (`top`) and
/// below it (`bottom`), as fractions of the font size. Faces differ a lot: Inter's capitals stand on the baseline
/// and reach 0.73; Atelico Type's reach 0.80 and hang 0.15 below it; VT323's reach 0.56.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Caps {
    pub top: f32,
    pub bottom: f32,
}

impl Caps {
    fn of(font: &Font) -> Self {
        const AT: f32 = 100.0;
        let m = font.metrics('H', AT);
        if m.height == 0 {
            return Caps { top: 0.7, bottom: 0.0 };
        }
        Caps { top: (m.ymin + m.height as i32) as f32 / AT, bottom: m.ymin as f32 / AT }
    }
}

/// The ink of one line, from its glyphs' pixel boxes, in device pixels from its pen start and baseline: `left`
/// and `right` along it, `above` and `below` the baseline.
#[derive(Clone, Copy, Debug)]
struct Ink {
    left: f32,
    right: f32,
    above: f32,
    below: f32,
}

pub struct Fonts {
    fonts: Vec<Font>,
    caps: Vec<Caps>,
    paths: HashMap<PathBuf, usize>,
    glyphs: HashMap<(usize, char, u32), Glyph>,
    /// Each character's advance by (font, character, size bits), and each string's width by a hash of (font, size,
    /// string): a window's texts are measured several times every frame (laid out, checked, cut to fit), and a
    /// glyph's metrics cost an outline walk each time.
    advances: RefCell<HashMap<(usize, char, u32), f32>>,
    widths: RefCell<HashMap<u64, f32>>,
}

/// Widths kept at most (the cache starts over past this: texts typed or ticking leave old ones behind).
const WIDTHS_KEPT: usize = 50_000;

/// A rasterized glyph and the box of its visible pixels (coverage of a quarter or more) in its bitmap,
/// [x0, y0, x1, y1), or None when it has none.
#[derive(Clone)]
struct Glyph {
    metrics: Metrics,
    mask: GlyphMask,
    ink: Option<[i32; 4]>,
}

impl Glyph {
    fn new((metrics, coverage): (Metrics, Vec<u8>)) -> Self {
        let (w, h) = (metrics.width as i32, metrics.height as i32);
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for y in 0..h {
            for x in 0..w {
                if coverage[(y * w + x) as usize] >= 64 {
                    (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
                }
            }
        }
        let ink = (x1 > x0).then_some([x0, y0, x1, y1]);
        Glyph { metrics, mask: GlyphMask { width: metrics.width, height: metrics.height, coverage: coverage.into() }, ink }
    }
}

impl Default for Fonts {
    fn default() -> Self {
        let inter = Font::from_bytes(DEFAULT, FontSettings::default()).expect("bundled font");
        Self { caps: vec![Caps::of(&inter)], fonts: vec![inter], paths: HashMap::new(), glyphs: HashMap::new(), advances: RefCell::default(), widths: RefCell::default() }
    }
}

impl Fonts {
    fn load(&mut self, path: PathBuf) -> usize {
        if let Some(i) = self.paths.get(&path) {
            return *i;
        }
        let i = match std::fs::read(&path).ok().and_then(|b| Font::from_bytes(b, FontSettings::default()).ok()) {
            Some(f) => {
                self.caps.push(Caps::of(&f));
                self.fonts.push(f);
                self.fonts.len() - 1
            }
            None => 0,
        };
        self.paths.insert(path, i);
        i
    }

    /// The face that draws `ch`: the font's own glyph, else the default face's (a font may lack even the space).
    fn face(&self, font: usize, ch: char) -> &Font {
        if self.fonts[font].lookup_glyph_index(ch) != 0 {
            &self.fonts[font]
        } else {
            &self.fonts[0]
        }
    }

    fn advance(&self, font: usize, c: char, size: f32) -> f32 {
        *self.advances.borrow_mut().entry((font, c, size.to_bits())).or_insert_with(|| self.face(font, c).metrics(c, size).advance_width)
    }

    fn width(&self, font: usize, s: &str, size: f32) -> f32 {
        use std::hash::{Hash, Hasher};
        let key = {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            (font, size.to_bits(), s).hash(&mut h);
            h.finish()
        };
        if let Some(w) = self.widths.borrow().get(&key) {
            return *w;
        }
        let w = s.chars().map(|c| self.advance(font, c, size)).sum();
        let mut widths = self.widths.borrow_mut();
        if widths.len() >= WIDTHS_KEPT {
            widths.clear();
        }
        widths.insert(key, w);
        w
    }

    /// The ink of `line` set at `px` device pixels (its glyphs' visible pixels), or None when it has none.
    fn ink(&mut self, font: usize, line: &str, px: f32) -> Option<Ink> {
        let mut pen = 0.0;
        let mut ink: Option<Ink> = None;
        for ch in line.chars() {
            let g = self.glyph(font, ch, px);
            let m = g.metrics;
            if let Some([x0, y0, x1, y1]) = g.ink {
                let left = (pen + m.xmin as f32).round() + x0 as f32;
                let right = (pen + m.xmin as f32).round() + x1 as f32;
                let top = m.ymin + m.height as i32;
                let (above, below) = ((top - y0) as f32, (y1 - top) as f32);
                ink = Some(match ink {
                    Some(i) => Ink { left: i.left.min(left), right: i.right.max(right), above: i.above.max(above), below: i.below.max(below) },
                    None => Ink { left, right, above, below },
                });
            }
            pen += m.advance_width;
        }
        ink
    }

    /// Lines and their widths for text laid into `max` logical px.
    /// `text` cut short to fit `max` wide, ending in "…" (unchanged when it fits).
    pub fn ellipsize(&self, t: &TextSpec, text: &str, max: f32) -> String {
        if self.width(t.font, text, t.size) <= max + 0.5 {
            return text.to_string();
        }
        let chars: Vec<char> = text.chars().collect();
        let mut n = chars.len();
        while n > 0 {
            let cut: String = chars[..n].iter().collect::<String>().trim_end().to_string() + "…";
            if self.width(t.font, &cut, t.size) <= max + 0.5 {
                return cut;
            }
            n -= 1;
        }
        String::new()
    }

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

    /// Sets `t` in `r`. Its ink is centred on the lines' middle: the block of lines sits so the ink of its glyphs
    /// (top of the tallest to foot of the deepest) is centred in the lines' height, whatever the face's ascent and
    /// descent say; a centred line, or one as wide as its box, has its ink centred across too. An input's value is
    /// set on the face's cap band instead, so it stays put as it is typed.
    pub fn draw(&mut self, c: &mut dyn Canvas, t: &TextSpec, r: Rect, caret: bool) {
        let scale = c.scale();
        let lines = self.wrap(t, r.w);
        let lh = t.line_height();
        let px = t.size * scale;
        let caps = self.caps[t.font];
        let band = Ink { left: 0.0, right: 0.0, above: caps.top * px, below: -caps.bottom * px };
        let inks: Vec<Option<Ink>> = lines.iter().map(|(line, _)| if t.editable { None } else { self.ink(t.font, line, px) }).collect();
        // the block's ink, from the first baseline (device px), and the first baseline that centres it
        let (mut top, mut bottom) = (f32::MAX, f32::MIN);
        for (i, ink) in inks.iter().enumerate() {
            let k = ink.unwrap_or(band);
            top = top.min(i as f32 * lh * scale - k.above);
            bottom = bottom.max(i as f32 * lh * scale + k.below);
        }
        let first = (r.y + lines.len() as f32 * lh / 2.0) * scale - (top + bottom) / 2.0;
        let mut end = (r.x, r.y);
        for (i, ((line, w), ink)) in lines.iter().zip(&inks).enumerate() {
            let centred = t.align == Align::Center || (t.align == Align::Start && !t.editable && r.w - w < 1.0);
            let x = match (ink, centred, t.align) {
                (Some(k), true, _) => (r.x + r.w / 2.0) * scale - (k.left + k.right) / 2.0,
                (None, true, _) => (r.x + (r.w - w) / 2.0) * scale,
                (_, false, Align::End) => (r.x + r.w - w) * scale,
                _ => r.x * scale,
            };
            let baseline = (first + i as f32 * lh * scale).round() as i32;
            let mut pen = x;
            // whole pixels from the pen's start, as `ink` measured them
            let start = x.round();
            for ch in line.chars() {
                let g = self.glyph(t.font, ch, px);
                let m = g.metrics;
                let gx = (start + (pen - x + m.xmin as f32).round()) as i32;
                let gy = baseline - m.height as i32 - m.ymin;
                c.glyph(gx, gy, &g.mask, t.color);
                pen += m.advance_width;
            }
            end = (pen / scale, r.y + i as f32 * lh);
        }
        if caret {
            c.fill_rect(Rect { x: end.0 + 1.0, y: end.1 + 2.0, w: 2.0, h: lh - 4.0 }, &Paint::Solid(t.color), true);
        }
    }

    fn glyph(&mut self, font: usize, ch: char, px: f32) -> &Glyph {
        let key = (font, ch, px.to_bits());
        if !self.glyphs.contains_key(&key) {
            let g = Glyph::new(self.face(font, ch).rasterize(ch, px));
            self.glyphs.insert(key, g);
        }
        &self.glyphs[&key]
    }
}
