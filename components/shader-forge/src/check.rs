//! Does a rendered fire frame read as one fire? Measured on the output's alpha and the core's mask:
//! - `bodies`: pieces of flame larger than 1% of the frame (a flame sliced into slabs has several),
//! - `stripe`: the tallest hole (see-through: under 85% alpha) enclosed by flame, over the flame's height (a cut
//!   straight through the body),
//! - `translucent`: the share of the flame's row spans that is see-through (ghosted cards, faded cuts),
//! - `core_pieces` / `core_holes`: the core must be one unbroken shape,
//! - `core_outside`: the share of the core not covered by flame (the core must sit inside the envelope),
//! - `stray`: flame alpha in pieces below 1% of the frame, lower than the flame's middle (embers without a fire).
//! Delivery checks, not beauty: the frames still get looked at.
use serde::Serialize;

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Fire {
    pub bodies: u32,
    pub body_area: f32,
    pub stripe: f32,
    pub translucent: f32,
    pub core_pieces: u32,
    pub core_holes: u32,
    pub core_outside: f32,
    pub stray: f32,
}

#[derive(Serialize, Clone, Copy, Debug)]
pub struct Limits {
    pub bodies: u32,
    pub min_body_area: f32,
    pub stripe: f32,
    pub translucent: f32,
    pub core_outside: f32,
    pub stray: f32,
}

pub const LIMITS: Limits = Limits { bodies: 1, min_body_area: 0.04, stripe: 0.2, translucent: 0.03, core_outside: 0.02, stray: 0.004 };

impl Fire {
    /// What fails, in words; empty when the frame passes.
    pub fn failures(&self, l: &Limits) -> Vec<String> {
        let mut f = Vec::new();
        if self.bodies > l.bodies {
            f.push(format!("{} separate bodies", self.bodies));
        }
        if self.body_area < l.min_body_area {
            f.push(format!("body covers {:.1}% of the frame", self.body_area * 100.0));
        }
        if self.stripe > l.stripe {
            f.push(format!("a hole {:.0}% of the flame's height cuts through it", self.stripe * 100.0));
        }
        if self.translucent > l.translucent {
            f.push(format!("{:.0}% of the flame is see-through", self.translucent * 100.0));
        }
        if self.core_pieces != 1 || self.core_holes != 0 {
            f.push(format!("core in {} pieces with {} holes", self.core_pieces, self.core_holes));
        }
        if self.core_outside > l.core_outside {
            f.push(format!("{:.1}% of the core lies outside the flame", self.core_outside * 100.0));
        }
        if self.stray > l.stray {
            f.push(format!("{:.2}% of the frame is embers with no flame", self.stray * 100.0));
        }
        f
    }
}

/// Connected components (4-neighbour) of `on`: a label per pixel (0 = off) and each label's pixel count.
fn components(on: &[bool], w: usize, h: usize) -> (Vec<u32>, Vec<usize>) {
    let mut label = vec![0u32; on.len()];
    let mut sizes = vec![0usize];
    let mut stack = Vec::new();
    for start in 0..on.len() {
        if !on[start] || label[start] != 0 {
            continue;
        }
        let id = sizes.len() as u32;
        sizes.push(0);
        label[start] = id;
        stack.push(start);
        while let Some(k) = stack.pop() {
            sizes[id as usize] += 1;
            let (x, y) = (k % w, k / w);
            let mut visit = |n: usize| {
                if on[n] && label[n] == 0 {
                    label[n] = id;
                    stack.push(n);
                }
            };
            if x > 0 {
                visit(k - 1);
            }
            if x + 1 < w {
                visit(k + 1);
            }
            if y > 0 {
                visit(k - w);
            }
            if y + 1 < h {
                visit(k + w);
            }
        }
    }
    (label, sizes)
}

/// Pixels that are off and not reachable from the frame's border through off pixels: holes enclosed by `on`.
fn enclosed(on: &[bool], w: usize, h: usize) -> Vec<bool> {
    let off: Vec<bool> = on.iter().map(|x| !x).collect();
    let (label, _) = components(&off, w, h);
    let mut open = std::collections::BTreeSet::new();
    for x in 0..w {
        open.insert(label[x]);
        open.insert(label[(h - 1) * w + x]);
    }
    for y in 0..h {
        open.insert(label[y * w]);
        open.insert(label[y * w + w - 1]);
    }
    label.iter().map(|l| *l != 0 && !open.contains(l)).collect()
}

/// `out`: the fire's RGBA (alpha is the flame); `core`: the core node's render (red channel is its mask).
pub fn fire(out: &[u8], core: &[u8], w: u32, h: u32) -> Fire {
    let (w, h) = (w as usize, h as usize);
    let n = w * h;
    let alpha: Vec<bool> = out.chunks(4).map(|p| p[3] > 127).collect();
    let (label, sizes) = components(&alpha, w, h);
    let big = n / 100;
    let is_body: Vec<bool> = label.iter().map(|l| *l != 0 && sizes[*l as usize] >= big).collect();
    let bodies = sizes.iter().skip(1).filter(|s| **s >= big).count() as u32;
    let body_px = is_body.iter().filter(|x| **x).count();
    // the flame's rows: from the lowest to the highest body pixel
    let rows: Vec<usize> = (0..h).filter(|y| is_body[y * w..(y + 1) * w].iter().any(|x| *x)).collect();
    let (top, bottom) = (rows.first().copied().unwrap_or(0), rows.last().copied().unwrap_or(0));
    let height = (bottom + 1).saturating_sub(top).max(1);
    let solid: Vec<bool> = (0..n).map(|k| is_body[k] && out[k * 4 + 3] > 216).collect();
    let holes = enclosed(&solid, w, h);
    // within each row's span of solid flame: half to mostly opaque pixels (ghosted layers; not the glow, not the
    // anti-aliased rim 2 px in)
    let (mut span, mut ghost) = (0usize, 0usize);
    for y in 0..h {
        let row = &solid[y * w..(y + 1) * w];
        if let (Some(a), Some(b)) = (row.iter().position(|x| *x), row.iter().rposition(|x| *x)) {
            for x in a + 2..b.saturating_sub(1) {
                span += 1;
                let al = out[(y * w + x) * 4 + 3];
                ghost += (al > 127 && al <= 216) as usize;
            }
        }
    }
    let mut stripe = 0usize;
    for x in 0..w {
        let mut run = 0;
        for y in 0..h {
            run = if holes[y * w + x] { run + 1 } else { 0 };
            stripe = stripe.max(run);
        }
    }
    let middle = (top + bottom) / 2;
    let stray = (0..n).filter(|k| alpha[*k] && !is_body[*k] && k / w > middle).count();
    // the core node as the preview shows it: sRGB, so half its mask (0.5 linear) is 188
    let core_on: Vec<bool> = core.chunks(4).map(|p| p[0] > 187).collect();
    let (_, core_sizes) = components(&core_on, w, h);
    let core_px = core_on.iter().filter(|x| **x).count();
    let core_holes = {
        let e = enclosed(&core_on, w, h);
        let (_, s) = components(&e, w, h);
        s.len() as u32 - 1
    };
    let core_outside = (0..n).filter(|k| core_on[*k] && !alpha[*k]).count() as f32 / core_px.max(1) as f32;
    Fire {
        bodies,
        body_area: body_px as f32 / n as f32,
        stripe: stripe as f32 / height as f32,
        translucent: ghost as f32 / span.max(1) as f32,
        core_pieces: core_sizes.len() as u32 - 1,
        core_holes,
        core_outside,
        stray: stray as f32 / n as f32,
    }
}
