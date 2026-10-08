//! Whether text sits centred in its box, measured from the pixels painted. A text-in-a-box is a node that draws a
//! box (a background, gradient or border) around exactly one child, a text. Its text is meant to be centred on an
//! axis when the box centres its child on that axis (align / justify = "center") or hugs the text on it (the box's
//! content is as wide as the widest line, or as tall as the lines). The ink is every pixel the text changed; its
//! centre is compared with the centre of the box's content (inside padding and border).
use atelico_component::tiny_skia::Pixmap;
use atelico_component::{Rect, Value};

/// Off-centre by more than this many pixels of the frame, or this share of the box, whichever is more, fails.
pub const PIXELS: f32 = 1.0;
pub const SHARE: f32 = 0.03;

/// One text-in-a-box as painted: where it is, its content box and ink (logical), and how far the ink's centre is
/// from the content's centre on each axis it is meant to be centred on, in pixels of the frame.
#[derive(Clone, Debug)]
pub struct Centring {
    pub at: String,
    pub text: String,
    /// Its face (the style's font path; empty: the layout's own).
    pub font: String,
    pub content: Rect,
    pub ink: Rect,
    /// (x, y) offsets in frame pixels; None on an axis the text is not meant to be centred on.
    pub by: [Option<f32>; 2],
    /// The most each axis may be off, in frame pixels.
    pub limit: [f32; 2],
}

impl Centring {
    pub fn ok(&self) -> bool {
        self.by.iter().zip(self.limit).all(|(b, l)| b.is_none_or(|b| b.abs() <= l))
    }
}

/// The pixels of `r` (logical, at `scale`) before the text is painted, to diff against after.
pub struct Before {
    x0: u32,
    y0: u32,
    w: u32,
    h: u32,
    pixels: Vec<u8>,
}

pub fn before(pm: &Pixmap, r: Rect, scale: f32) -> Option<Before> {
    let x0 = (r.x * scale).floor().max(0.0) as u32;
    let y0 = (r.y * scale).floor().max(0.0) as u32;
    let x1 = ((r.x + r.w) * scale).ceil().min(pm.width() as f32) as u32;
    let y1 = ((r.y + r.h) * scale).ceil().min(pm.height() as f32) as u32;
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let (w, h) = (x1 - x0, y1 - y0);
    let data = pm.data();
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for y in y0..y1 {
        let i = ((y * pm.width() + x0) * 4) as usize;
        pixels.extend_from_slice(&data[i..i + (w * 4) as usize]);
    }
    Some(Before { x0, y0, w, h, pixels })
}

/// The ink the text painted since `before`, in frame pixels [x0, y0, x1, y1): every pixel whose color changed by
/// at least a quarter of the largest change (anti-aliased fringes fade out evenly on both sides).
pub fn ink(pm: &Pixmap, b: &Before) -> Option<[f32; 4]> {
    let data = pm.data();
    let change = |x: u32, y: u32| -> u32 {
        let i = (((b.y0 + y) * pm.width() + b.x0 + x) * 4) as usize;
        let j = ((y * b.w + x) * 4) as usize;
        (0..3).map(|k| (data[i + k] as i32 - b.pixels[j + k] as i32).unsigned_abs()).max().unwrap_or(0)
    };
    let mut most = 0;
    for y in 0..b.h {
        for x in 0..b.w {
            most = most.max(change(x, y));
        }
    }
    if most < 16 {
        return None;
    }
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for y in 0..b.h {
        for x in 0..b.w {
            if change(x, y) * 4 >= most {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
            }
        }
    }
    Some([(b.x0 + x0) as f32, (b.y0 + y0) as f32, (b.x0 + x1) as f32, (b.y0 + y1) as f32])
}

/// Which axes (x, y) a box means its only child to be centred on: the axes it centres it on, or where its content
/// is just the text's size (`lines`: the widest line and the lines' height, logical).
pub fn axes(st: &Value, content: Rect, lines: (f32, f32), text_align_centre: bool) -> [bool; 2] {
    let column = st.get("direction").and_then(Value::as_str) == Some("column");
    let centre = |k: &str| st.get(k).and_then(Value::as_str) == Some("center");
    let (main, cross) = (centre("justify"), centre("align"));
    let (x, y) = if column { (cross, main) } else { (main, cross) };
    [x || text_align_centre || lines.0 >= content.w - 1.5, y || lines.1 >= content.h - 1.5]
}

/// The record of one text-in-a-box: `ink` in frame pixels, `content` logical.
pub fn measure(at: String, text: String, font: String, content: Rect, ink: [f32; 4], scale: f32, axes: [bool; 2]) -> Centring {
    let [x0, y0, x1, y1] = ink;
    let dx = (x0 + x1) / 2.0 - (content.x + content.w / 2.0) * scale;
    let dy = (y0 + y1) / 2.0 - (content.y + content.h / 2.0) * scale;
    let limit = [PIXELS.max(SHARE * content.w * scale), PIXELS.max(SHARE * content.h * scale)];
    Centring {
        at,
        text,
        font,
        content,
        ink: Rect { x: x0 / scale, y: y0 / scale, w: (x1 - x0) / scale, h: (y1 - y0) / scale },
        by: [axes[0].then_some(dx), axes[1].then_some(dy)],
        limit,
    }
}
