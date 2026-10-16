//! Deterministic orthogonal presentation geometry. IDs are opaque caller references;
//! this module knows nothing about task status, dependencies, grouping or persistence.
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

// Reserve room for a 48×30 count and its six-pixel separation.
const CLEARANCE_X: f64 = 36.0;
const CLEARANCE_Y: f64 = 21.0;
const BEND: f64 = 24.0;
const EPS: f64 = 0.000_001;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodeId(pub String);
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EdgeId(pub String);
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
impl Rect {
    fn right(self) -> f64 {
        self.x + self.w
    }
    fn bottom(self) -> f64 {
        self.y + self.h
    }
    fn center(self) -> Point {
        Point {
            x: self.x + self.w / 2.,
            y: self.y + self.h / 2.,
        }
    }
    fn inflate(self, n: f64) -> Self {
        Self {
            x: self.x - n,
            y: self.y - n,
            w: self.w + 2. * n,
            h: self.h + 2. * n,
        }
    }
    fn inside(self, p: Point) -> bool {
        p.x > self.x + EPS
            && p.x < self.right() - EPS
            && p.y > self.y + EPS
            && p.y < self.bottom() - EPS
    }
    fn overlaps(self, b: Self) -> bool {
        self.x < b.right() - EPS
            && self.right() > b.x + EPS
            && self.y < b.bottom() - EPS
            && self.bottom() > b.y + EPS
    }
    fn valid(self) -> bool {
        [
            self.x,
            self.y,
            self.w,
            self.h,
            self.right(),
            self.bottom(),
            self.x - CLEARANCE_X,
            self.y - CLEARANCE_Y,
            self.right() + CLEARANCE_X,
            self.bottom() + CLEARANCE_Y,
        ]
        .into_iter()
        .all(f64::is_finite)
            && self.w > 0.
            && self.h > 0.
            && self.right() > self.x
            && self.bottom() > self.y
    }
    fn union(self, b: Self) -> Self {
        let x = self.x.min(b.x);
        let y = self.y.min(b.y);
        Self {
            x,
            y,
            w: self.right().max(b.right()) - x,
            h: self.bottom().max(b.bottom()) - y,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub rect: Rect,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    pub id: EdgeId,
    pub points: Vec<Point>,
    pub badge: Rect,
    /// Only present when a short route has no room for its count. Also obstacle-routed.
    pub leader: Vec<Point>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub routes: Vec<Route>,
    pub bounds: Rect,
}

/// Whether an axis-aligned segment crosses a rectangle's open interior. Running
/// along an inflated obstacle boundary is safe, including rounded wire corners.
fn crosses(a: Point, b: Point, r: Rect) -> bool {
    if (a.x - b.x).abs() < EPS {
        a.x > r.x + EPS
            && a.x < r.right() - EPS
            && a.y.min(b.y) < r.bottom() - EPS
            && a.y.max(b.y) > r.y + EPS
    } else {
        a.y > r.y + EPS
            && a.y < r.bottom() - EPS
            && a.x.min(b.x) < r.right() - EPS
            && a.x.max(b.x) > r.x + EPS
    }
}
fn clear(a: Point, b: Point, obstacles: &[Rect]) -> bool {
    obstacles.iter().all(|r| !crosses(a, b, *r))
}
fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
fn simplify(points: Vec<Point>) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::new();
    for p in points {
        if out.last() == Some(&p) {
            continue;
        }
        while out.len() >= 2 {
            let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
            if ((a.x - b.x).abs() < EPS && (b.x - p.x).abs() < EPS)
                || ((a.y - b.y).abs() < EPS && (b.y - p.y).abs() < EPS)
            {
                out.pop();
            } else {
                break;
            }
        }
        out.push(p);
    }
    out
}

#[derive(Clone, Copy)]
struct Port {
    tip: Point,
    outside: Point,
    penalty: f64,
}
fn ports(a: Rect, b: Rect) -> Vec<Port> {
    let ac = a.center();
    let bc = b.center();
    let vertical = (ac.y - bc.y).abs() > (ac.x - bc.x).abs();
    let preferred = if vertical {
        if bc.y > ac.y {
            3
        } else {
            2
        }
    } else if bc.x > ac.x {
        1
    } else {
        0
    };
    // Opposite directions have distinct attachment points even when endpoints align.
    let shift: f64 = if bc.x > ac.x || (bc.x == ac.x && bc.y > ac.y) {
        12.
    } else {
        -12.
    };
    let px = ac.x + shift.clamp(-a.w / 4., a.w / 4.);
    let py = ac.y - shift.clamp(-a.h / 4., a.h / 4.);
    [
        (
            Point { x: a.x, y: py },
            Point {
                x: a.x - CLEARANCE_X,
                y: py,
            },
        ),
        (
            Point {
                x: a.right(),
                y: py,
            },
            Point {
                x: a.right() + CLEARANCE_X,
                y: py,
            },
        ),
        (
            Point { x: px, y: a.y },
            Point {
                x: px,
                y: a.y - CLEARANCE_Y,
            },
        ),
        (
            Point {
                x: px,
                y: a.bottom(),
            },
            Point {
                x: px,
                y: a.bottom() + CLEARANCE_Y,
            },
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (tip, outside))| Port {
        tip,
        outside,
        penalty: if i == preferred { 0. } else { 48. },
    })
    .collect()
}

#[derive(Clone, Copy)]
struct Visit {
    cost: f64,
    state: usize,
}
impl PartialEq for Visit {
    fn eq(&self, b: &Self) -> bool {
        self.cost == b.cost && self.state == b.state
    }
}
impl Eq for Visit {}
impl PartialOrd for Visit {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        Some(self.cmp(b))
    }
}
impl Ord for Visit {
    fn cmp(&self, b: &Self) -> Ordering {
        b.cost
            .total_cmp(&self.cost)
            .then_with(|| b.state.cmp(&self.state))
    }
}

/// A small rectilinear visibility grid over obstacle boundaries and port positions.
/// Empty intersections are searched lazily, with stable coordinate/direction ties.
fn path(
    starts: &[Port],
    ends: &[Port],
    obstacles: &[Rect],
    previous: &[Route],
) -> Option<Vec<Point>> {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for r in obstacles {
        xs.extend([r.x, r.right()]);
        ys.extend([r.y, r.bottom()]);
    }
    for p in starts.iter().chain(ends) {
        xs.push(p.outside.x);
        ys.push(p.outside.y);
    }
    let sort = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v.dedup_by(|a, b| (*a - *b).abs() < EPS);
    };
    sort(&mut xs);
    sort(&mut ys);
    xs.extend([xs.first()? - 48., xs.last()? + 48.]);
    ys.extend([ys.first()? - 48., ys.last()? + 48.]);
    sort(&mut xs);
    sort(&mut ys);
    let w = xs.len();
    let h = ys.len();
    let point = |n: usize| Point {
        x: xs[n % w],
        y: ys[n / w],
    };
    let index = |p: Point| {
        ys.iter().position(|y| (*y - p.y).abs() < EPS).unwrap() * w
            + xs.iter().position(|x| (*x - p.x).abs() < EPS).unwrap()
    };
    let allowed: Vec<bool> = (0..w * h)
        .map(|i| !obstacles.iter().any(|r| r.inside(point(i))))
        .collect();
    let mut costs = vec![f64::INFINITY; w * h * 3];
    let mut back = vec![None; costs.len()];
    let mut start_at = vec![None; costs.len()];
    let mut heap = BinaryHeap::new();
    for (i, p) in starts.iter().enumerate() {
        let at = index(p.outside) * 3 + 2;
        if allowed[at / 3] && p.penalty < costs[at] {
            costs[at] = p.penalty;
            start_at[at] = Some(i);
            heap.push(Visit {
                cost: p.penalty,
                state: at,
            });
        }
    }
    let goals: BTreeMap<usize, usize> = ends
        .iter()
        .enumerate()
        .filter(|(_, p)| allowed[index(p.outside)])
        .map(|(i, p)| (index(p.outside), i))
        .collect();
    let mut best: Option<(f64, usize, usize)> = None;
    while let Some(Visit { cost, state }) = heap.pop() {
        if cost > costs[state] {
            continue;
        }
        if best.is_some_and(|b| cost > b.0) {
            break;
        }
        let n = state / 3;
        let direction = state % 3;
        if let Some(&i) = goals.get(&n) {
            let total = cost + ends[i].penalty;
            if best.is_none_or(|b| total < b.0) {
                best = Some((total, state, i));
            }
        }
        let x = n % w;
        let y = n / w;
        let neighbors = [
            x.checked_sub(1).map(|x| y * w + x),
            (x + 1 < w).then_some(n + 1),
            y.checked_sub(1).map(|y| y * w + x),
            (y + 1 < h).then_some(n + w),
        ];
        for (j, next) in neighbors.into_iter().enumerate() {
            let Some(next) = next.filter(|n| allowed[*n]) else {
                continue;
            };
            let a = point(n);
            let b = point(next);
            if !clear(a, b, obstacles) {
                continue;
            }
            let axis = usize::from(j >= 2);
            let mut extra = 0.;
            // Avoid coincident reverse bundles without changing their semantic identity.
            for r in previous {
                for pair in r.points.windows(2) {
                    let (c, d) = (pair[0], pair[1]);
                    if axis == 0 && (a.y - c.y).abs() < EPS && (c.y - d.y).abs() < EPS {
                        extra += (a.x.max(b.x).min(c.x.max(d.x)) - a.x.min(b.x).max(c.x.min(d.x)))
                            .max(0.)
                            * 4.;
                    }
                    if axis == 1 && (a.x - c.x).abs() < EPS && (c.x - d.x).abs() < EPS {
                        extra += (a.y.max(b.y).min(c.y.max(d.y)) - a.y.min(b.y).max(c.y.min(d.y)))
                            .max(0.)
                            * 4.;
                    }
                }
            }
            let total = cost
                + distance(a, b)
                + extra
                + if direction != 2 && direction != axis {
                    BEND
                } else {
                    0.
                };
            let at = next * 3 + axis;
            if total + EPS < costs[at] {
                costs[at] = total;
                back[at] = Some(state);
                heap.push(Visit {
                    cost: total,
                    state: at,
                });
            }
        }
    }
    let (_, mut at, end) = best?;
    let mut points = vec![ends[end].tip];
    loop {
        points.push(point(at / 3));
        if let Some(prev) = back[at] {
            at = prev;
        } else {
            break;
        }
    }
    points.push(starts[start_at[at]?].tip);
    points.reverse();
    Some(simplify(points))
}

fn badge(points: &[Point], nodes: &[Rect], prior: &[Route]) -> Option<Rect> {
    let mut segments: Vec<_> = points.windows(2).enumerate().collect();
    segments.sort_by(|(ai, a), (bi, b)| {
        distance(b[0], b[1])
            .total_cmp(&distance(a[0], a[1]))
            .then_with(|| ai.cmp(bi))
    });
    for (_, p) in segments {
        for t in [0.5, 0.25, 0.75, 0.125, 0.875] {
            let c = Point {
                x: p[0].x + (p[1].x - p[0].x) * t,
                y: p[0].y + (p[1].y - p[0].y) * t,
            };
            let b = Rect {
                x: c.x - 24.,
                y: c.y - 15.,
                w: 48.,
                h: 30.,
            };
            let guard = b.inflate(6.);
            if nodes.iter().all(|r| !guard.overlaps(*r))
                && prior.iter().all(|r| {
                    !guard.overlaps(r.badge)
                        && r.points.windows(2).all(|p| !crosses(p[0], p[1], guard))
                })
            {
                return Some(b);
            }
        }
    }
    None
}

pub fn route(request: &Request) -> Result<Layout, String> {
    let mut nodes = BTreeMap::new();
    for n in &request.nodes {
        if n.id.0.is_empty() || !n.rect.valid() || nodes.insert(n.id.clone(), n.rect).is_some() {
            return Err("routing requires unique node IDs and finite, positive rectangles".into());
        }
    }
    let mut edges: Vec<_> = request.edges.iter().collect();
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    let mut seen = BTreeSet::new();
    let raw: Vec<_> = nodes.values().copied().collect();
    let mut bounds = raw.iter().copied().reduce(Rect::union).unwrap_or(Rect {
        x: 0.,
        y: 0.,
        w: 1.,
        h: 1.,
    });
    if !bounds.inflate(66.).valid() {
        return Err("combined route bounds exceed finite coordinates".into());
    }
    let mut routes: Vec<Route> = Vec::new();
    for e in edges {
        if e.id.0.is_empty() || !seen.insert(&e.id) {
            return Err("routing requires unique edge IDs".into());
        }
        let a = *nodes.get(&e.from).ok_or("route source is missing")?;
        let b = *nodes.get(&e.to).ok_or("route destination is missing")?;
        if e.from == e.to {
            return Err("internal links belong to their group, not an external route".into());
        }
        let mut obstacles: Vec<_> = raw
            .iter()
            .map(|r| Rect {
                x: r.x - CLEARANCE_X,
                y: r.y - CLEARANCE_Y,
                w: r.w + 2. * CLEARANCE_X,
                h: r.h + 2. * CLEARANCE_Y,
            })
            .collect();
        obstacles.extend(routes.iter().map(|r| r.badge.inflate(6.)));
        let starts = ports(a, b);
        let ends = ports(b, a);
        let mut points =
            path(&starts, &ends, &obstacles, &routes).ok_or("no clear route between card ports")?;
        let mut on_route = badge(&points, &raw, &routes);
        if on_route.is_none() {
            // A count is part of the route. Prefer a modest clear perimeter lane
            // over a distant callout that unnecessarily makes Fit shrink cards.
            let extent = raw.iter().copied().reduce(Rect::union).unwrap();
            let middle = Point {
                x: (a.center().x + b.center().x) / 2.,
                y: (a.center().y + b.center().y) / 2.,
            };
            let anchors = [
                Point {
                    x: extent.x - CLEARANCE_X,
                    y: middle.y,
                },
                Point {
                    x: extent.right() + CLEARANCE_X,
                    y: middle.y,
                },
                Point {
                    x: middle.x,
                    y: extent.y - CLEARANCE_Y,
                },
                Point {
                    x: middle.x,
                    y: extent.bottom() + CLEARANCE_Y,
                },
            ];
            let mut best: Option<(f64, Vec<Point>, Rect)> = None;
            for anchor in anchors {
                let port = Port {
                    tip: anchor,
                    outside: anchor,
                    penalty: 0.,
                };
                if let (Some(mut first), Some(last)) = (
                    path(&starts, &[port], &obstacles, &routes),
                    path(&[port], &ends, &obstacles, &routes),
                ) {
                    first.extend(last);
                    let candidate = simplify(first);
                    if let Some(label) = badge(&candidate, &raw, &routes) {
                        let length = candidate.windows(2).map(|p| distance(p[0], p[1])).sum();
                        if best.as_ref().is_none_or(|b| length < b.0) {
                            best = Some((length, candidate, label));
                        }
                    }
                }
            }
            if let Some((_, candidate, label)) = best {
                points = candidate;
                on_route = Some(label);
            }
        }
        if points
            .windows(2)
            .any(|p| raw.iter().any(|r| crosses(p[0], p[1], *r)))
        {
            return Err("route port stub crosses a card".into());
        }
        let mut leader = Vec::new();
        let badge = if let Some(badge) = on_route {
            badge
        } else {
            // Rare dense/short connections use an explicit callout, still within Fit.
            let mut b = Rect {
                x: bounds.right() + 48.,
                y: bounds.y,
                w: 48.,
                h: 30.,
            };
            while routes.iter().any(|r| b.inflate(6.).overlaps(r.badge)) {
                b.y += 42.;
            }
            let start = points[points.len() / 2];
            let end = Point {
                x: b.x,
                y: b.y + 15.,
            };
            let p = |at| Port {
                tip: at,
                outside: at,
                penalty: 0.,
            };
            leader = path(&[p(start)], &[p(end)], &obstacles, &routes)
                .ok_or("no clear route to count callout")?;
            b
        };
        bounds = bounds.union(badge);
        for p in points.iter().chain(&leader) {
            bounds = bounds.union(Rect {
                x: p.x,
                y: p.y,
                w: 0.,
                h: 0.,
            });
        }
        routes.push(Route {
            id: e.id.clone(),
            points,
            badge,
            leader,
        });
    }
    let bounds = bounds.inflate(18.);
    if !bounds.valid() {
        return Err("route bounds exceed finite coordinates".into());
    }
    Ok(Layout { routes, bounds })
}

/// One last request per Lua context. Panning, selection and zoom never alter the
/// route request, so pointer frames reuse geometry without a global growing map.
#[derive(Default)]
pub struct Cache {
    last: Option<(Request, Layout)>,
    #[cfg(test)]
    computed: usize,
}
impl Cache {
    pub fn layout(&mut self, request: Request) -> Result<&Layout, String> {
        if self.last.as_ref().is_none_or(|(old, _)| old != &request) {
            let layout = route(&request)?;
            self.last = Some((request, layout));
            #[cfg(test)]
            {
                self.computed += 1;
            }
        }
        Ok(&self.last.as_ref().unwrap().1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node(id: &str, x: f64, y: f64) -> Node {
        Node {
            id: NodeId(id.into()),
            rect: Rect {
                x,
                y,
                w: 340.,
                h: 164.,
            },
        }
    }
    fn edge(id: &str, from: &str, to: &str) -> Edge {
        Edge {
            id: EdgeId(id.into()),
            from: NodeId(from.into()),
            to: NodeId(to.into()),
        }
    }
    fn assert_clear(request: &Request, layout: &Layout) {
        assert_eq!(layout.routes.len(), request.edges.len());
        for r in &layout.routes {
            let e = request.edges.iter().find(|e| e.id == r.id).unwrap();
            assert!(r.points.len() >= 2);
            for segment in r.points.windows(2).chain(r.leader.windows(2)) {
                assert!(
                    (segment[0].x - segment[1].x).abs() < EPS
                        || (segment[0].y - segment[1].y).abs() < EPS
                );
                for n in &request.nodes {
                    assert!(
                        !crosses(segment[0], segment[1], n.rect),
                        "{:?} crosses {:?}: {:?}",
                        e.id,
                        n.id,
                        segment
                    );
                }
            }
            for n in &request.nodes {
                assert!(!r.badge.overlaps(n.rect));
            }
            for p in r.points.iter().chain(&r.leader) {
                assert!(
                    p.x >= layout.bounds.x
                        && p.y >= layout.bounds.y
                        && p.x <= layout.bounds.right()
                        && p.y <= layout.bounds.bottom()
                );
            }
            assert!(
                r.badge.x >= layout.bounds.x
                    && r.badge.right() <= layout.bounds.right()
                    && r.badge.y >= layout.bounds.y
                    && r.badge.bottom() <= layout.bounds.bottom()
            );
        }
        for (i, a) in layout.routes.iter().enumerate() {
            for b in &layout.routes[i + 1..] {
                assert!(!a.badge.inflate(3.).overlaps(b.badge.inflate(3.)));
            }
        }
    }
    #[test]
    fn skipped_rows_cross_column_and_reciprocal_bundles_avoid_cards() {
        let req = Request {
            nodes: vec![
                node("a", 48., 20.),
                node("b", 478., 20.),
                node("c", 48., 228.),
                node("d", 478., 228.),
                node("e", 48., 436.),
                node("f", 478., 436.),
            ],
            edges: vec![
                edge("ae", "a", "e"),
                edge("fb", "f", "b"),
                edge("fc", "f", "c"),
                edge("bf", "b", "f"),
                edge("ab", "a", "b"),
            ],
        };
        let result = route(&req).unwrap();
        assert_clear(&req, &result);
        let mut reordered = req;
        reordered.nodes.reverse();
        reordered.edges.reverse();
        assert_eq!(
            route(&reordered).unwrap(),
            result,
            "stable IDs, not input iteration order, determine routes"
        );
    }
    #[test]
    fn arbitrary_obstacles_detour_without_assuming_rows_or_columns() {
        let req = Request {
            nodes: vec![
                node("left", -70., 70.),
                node("right", 1050., 95.),
                Node {
                    id: NodeId("obstacle".into()),
                    rect: Rect {
                        x: 390.,
                        y: -90.,
                        w: 410.,
                        h: 480.,
                    },
                },
            ],
            edges: vec![edge("proof", "left", "right")],
        };
        let result = route(&req).unwrap();
        assert_clear(&req, &result);
        assert!(result.routes[0]
            .points
            .iter()
            .any(|p| p.y < -90. || p.y > 390.));
    }
    #[test]
    fn dense_short_bundles_and_every_badge_are_in_full_fit_bounds() {
        let mut req = Request {
            nodes: vec![node("a", 0., 0.), node("b", 400., 0.)],
            edges: Vec::new(),
        };
        for i in 0..8 {
            req.edges.push(edge(
                &format!("edge-{i}"),
                if i % 2 == 0 { "a" } else { "b" },
                if i % 2 == 0 { "b" } else { "a" },
            ));
        }
        assert_clear(&req, &route(&req).unwrap());
    }
    #[test]
    fn malformed_geometry_and_missing_or_duplicate_identity_are_rejected() {
        let mut req = Request {
            nodes: vec![node("a", 0., 0.), node("b", 500., 0.)],
            edges: vec![edge("ab", "a", "b")],
        };
        req.nodes[0].rect.x = f64::NAN;
        assert!(route(&req).is_err());
        req.nodes[0].rect.x = f64::MAX;
        req.nodes[0].rect.w = f64::MAX;
        assert!(route(&req).is_err());
        req.nodes[0].rect.x = 0.;
        req.nodes[0].rect.w = 340.;
        req.edges.push(req.edges[0].clone());
        assert!(route(&req).is_err());
        req.edges.pop();
        req.edges[0].to = NodeId("missing".into());
        assert!(route(&req).is_err());
    }
    #[test]
    fn tight_spacing_checks_port_stubs_and_blocked_cards_fail_clearly() {
        let req = Request {
            nodes: vec![node("a", 0., 0.), node("b", 345., 0.), node("c", 690., 0.)],
            edges: vec![edge("ac", "a", "c")],
        };
        assert_clear(&req, &route(&req).unwrap());
        let mut trapped = req;
        trapped.nodes.push(Node {
            id: NodeId("cover".into()),
            rect: Rect {
                x: -20.,
                y: -20.,
                w: 400.,
                h: 210.,
            },
        });
        assert!(
            route(&trapped).is_err(),
            "a fully covered card cannot have an honest visible port"
        );
    }
    #[test]
    fn unchanged_geometry_reuses_one_bounded_cache_entry() {
        let mut req = Request {
            nodes: vec![node("a", 0., 0.), node("b", 500., 0.)],
            edges: vec![edge("ab", "a", "b")],
        };
        let mut cache = Cache::default();
        let first = cache.layout(req.clone()).unwrap().clone();
        for _ in 0..50 {
            assert_eq!(*cache.layout(req.clone()).unwrap(), first);
        }
        assert_eq!(cache.computed, 1);
        req.nodes[1].rect.y = 400.;
        assert_ne!(*cache.layout(req.clone()).unwrap(), first);
        assert_eq!(cache.computed, 2);
        req.edges[0].id = EdgeId("new stable identity".into());
        cache.layout(req).unwrap();
        assert_eq!(cache.computed, 3);
    }
}
