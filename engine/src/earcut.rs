//! Port of mapbox/earcut 2.2.x (the version three.js r160 bundles as `Earcut`).

#[derive(Clone)]
struct Node { i: usize, x: f64, y: f64, prev: usize, next: usize, z: i64, prev_z: usize, next_z: usize, steiner: bool }
const NIL: usize = usize::MAX;

struct Ctx { nodes: Vec<Node> }
impl Ctx {
    fn n(&self, i: usize) -> &Node { &self.nodes[i] }
    fn nm(&mut self, i: usize) -> &mut Node { &mut self.nodes[i] }
    fn insert(&mut self, i: usize, x: f64, y: f64, last: usize) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node { i, x, y, prev: id, next: id, z: 0, prev_z: NIL, next_z: NIL, steiner: false });
        if last != NIL {
            let ln = self.n(last).next;
            self.nm(id).next = ln;
            self.nm(id).prev = last;
            self.nm(ln).prev = id;
            self.nm(last).next = id;
        }
        id
    }
    fn remove(&mut self, p: usize) {
        let (pn, pp, pz, ppz) = (self.n(p).next, self.n(p).prev, self.n(p).next_z, self.n(p).prev_z);
        self.nm(pn).prev = pp;
        self.nm(pp).next = pn;
        if ppz != NIL { self.nm(ppz).next_z = pz; }
        if pz != NIL { self.nm(pz).prev_z = ppz; }
    }
    fn eq(&self, a: usize, b: usize) -> bool { self.n(a).x == self.n(b).x && self.n(a).y == self.n(b).y }
    fn area(&self, p: usize, q: usize, r: usize) -> f64 {
        let (p, q, r) = (self.n(p), self.n(q), self.n(r));
        (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y)
    }
}

pub fn earcut(data: &[f64], hole_indices: &[usize], dim: usize) -> Vec<usize> {
    let has_holes = !hole_indices.is_empty();
    let outer_len = if has_holes { hole_indices[0] * dim } else { data.len() };
    let mut c = Ctx { nodes: Vec::with_capacity(data.len() / dim + 8) };
    let mut outer = linked_list(&mut c, data, 0, outer_len, dim, true);
    let mut tris = vec![];
    if outer == NIL || c.n(outer).next == c.n(outer).prev { return tris; }
    let (mut min_x, mut min_y, mut inv) = (0.0, 0.0, 0.0);
    if has_holes { outer = eliminate_holes(&mut c, data, hole_indices, outer, dim); }
    if data.len() > 80 * dim {
        min_x = data[0]; min_y = data[1];
        let (mut max_x, mut max_y) = (min_x, min_y);
        let mut i = dim;
        while i < outer_len {
            let (x, y) = (data[i], data[i + 1]);
            if x < min_x { min_x = x; } if y < min_y { min_y = y; }
            if x > max_x { max_x = x; } if y > max_y { max_y = y; }
            i += dim;
        }
        inv = (max_x - min_x).max(max_y - min_y);
        inv = if inv != 0.0 { 32767.0 / inv } else { 0.0 };
    }
    earcut_linked(&mut c, outer, &mut tris, dim, min_x, min_y, inv, 0);
    tris
}

fn signed_area(data: &[f64], start: usize, end: usize, dim: usize) -> f64 {
    let mut sum = 0.0;
    let mut j = end - dim;
    let mut i = start;
    while i < end { sum += (data[j] - data[i]) * (data[i + 1] + data[j + 1]); j = i; i += dim; }
    sum
}

fn linked_list(c: &mut Ctx, data: &[f64], start: usize, end: usize, dim: usize, clockwise: bool) -> usize {
    let mut last = NIL;
    if clockwise == (signed_area(data, start, end, dim) > 0.0) {
        let mut i = start;
        while i < end { last = c.insert(i, data[i], data[i + 1], last); i += dim; }
    } else {
        let mut i = end as isize - dim as isize;
        while i >= start as isize { last = c.insert(i as usize, data[i as usize], data[i as usize + 1], last); i -= dim as isize; }
    }
    if last != NIL && c.eq(last, c.n(last).next) { let nx = c.n(last).next; c.remove(last); last = nx; }
    last
}

fn filter_points(c: &mut Ctx, start: usize, end: usize) -> usize {
    if start == NIL { return start; }
    let mut end = if end == NIL { start } else { end };
    let mut p = start;
    loop {
        let mut again = false;
        let pn = c.n(p).next; let pp = c.n(p).prev;
        if !c.n(p).steiner && (c.eq(p, pn) || c.area(pp, p, pn) == 0.0) {
            c.remove(p);
            p = pp; end = pp;
            if p == c.n(p).next { break; }
            again = true;
        } else {
            p = pn;
        }
        if !again && p == end { break; }
    }
    end
}

fn earcut_linked(c: &mut Ctx, ear: usize, tris: &mut Vec<usize>, dim: usize, min_x: f64, min_y: f64, inv: f64, pass: u32) {
    if ear == NIL { return; }
    let mut ear = ear;
    if pass == 0 && inv != 0.0 { index_curve(c, ear, min_x, min_y, inv); }
    let mut stop = ear;
    while c.n(ear).prev != c.n(ear).next {
        let prev = c.n(ear).prev; let next = c.n(ear).next;
        let is = if inv != 0.0 { is_ear_hashed(c, ear, min_x, min_y, inv) } else { is_ear(c, ear) };
        if is {
            tris.push(c.n(prev).i / dim); tris.push(c.n(ear).i / dim); tris.push(c.n(next).i / dim);
            c.remove(ear);
            ear = c.n(next).next;
            stop = c.n(next).next;
            continue;
        }
        ear = next;
        if ear == stop {
            if pass == 0 {
                let f = filter_points(c, ear, NIL);
                earcut_linked(c, f, tris, dim, min_x, min_y, inv, 1);
            } else if pass == 1 {
                let f = filter_points(c, ear, NIL);
                let e = cure_local_intersections(c, f, tris, dim);
                earcut_linked(c, e, tris, dim, min_x, min_y, inv, 2);
            } else if pass == 2 {
                split_earcut(c, ear, tris, dim, min_x, min_y, inv);
            }
            break;
        }
    }
}

fn is_ear(c: &Ctx, ear: usize) -> bool {
    let (a, b, cc) = (c.n(ear).prev, ear, c.n(ear).next);
    if c.area(a, b, cc) >= 0.0 { return false; }
    let (ax, bx, cx, ay, by, cy) = (c.n(a).x, c.n(b).x, c.n(cc).x, c.n(a).y, c.n(b).y, c.n(cc).y);
    let x0 = ax.min(bx).min(cx); let y0 = ay.min(by).min(cy); let x1 = ax.max(bx).max(cx); let y1 = ay.max(by).max(cy);
    let mut p = c.n(cc).next;
    while p != a {
        let pn = c.n(p);
        if pn.x >= x0 && pn.x <= x1 && pn.y >= y0 && pn.y <= y1 && point_in_triangle(ax, ay, bx, by, cx, cy, pn.x, pn.y) && c.area(pn.prev, p, pn.next) >= 0.0 { return false; }
        p = pn.next;
    }
    true
}

fn is_ear_hashed(c: &Ctx, ear: usize, min_x: f64, min_y: f64, inv: f64) -> bool {
    let (a, b, cc) = (c.n(ear).prev, ear, c.n(ear).next);
    if c.area(a, b, cc) >= 0.0 { return false; }
    let (ax, bx, cx, ay, by, cy) = (c.n(a).x, c.n(b).x, c.n(cc).x, c.n(a).y, c.n(b).y, c.n(cc).y);
    let x0 = ax.min(bx).min(cx); let y0 = ay.min(by).min(cy); let x1 = ax.max(bx).max(cx); let y1 = ay.max(by).max(cy);
    let min_z = z_order(x0, y0, min_x, min_y, inv); let max_z = z_order(x1, y1, min_x, min_y, inv);
    let mut p = c.n(ear).prev_z; let mut n = c.n(ear).next_z;
    while p != NIL && c.n(p).z >= min_z && n != NIL && c.n(n).z <= max_z {
        let pp = c.n(p);
        if pp.x >= x0 && pp.x <= x1 && pp.y >= y0 && pp.y <= y1 && p != a && p != cc && point_in_triangle(ax, ay, bx, by, cx, cy, pp.x, pp.y) && c.area(pp.prev, p, pp.next) >= 0.0 { return false; }
        p = pp.prev_z;
        let nn = c.n(n);
        if nn.x >= x0 && nn.x <= x1 && nn.y >= y0 && nn.y <= y1 && n != a && n != cc && point_in_triangle(ax, ay, bx, by, cx, cy, nn.x, nn.y) && c.area(nn.prev, n, nn.next) >= 0.0 { return false; }
        n = nn.next_z;
    }
    while p != NIL && c.n(p).z >= min_z {
        let pp = c.n(p);
        if pp.x >= x0 && pp.x <= x1 && pp.y >= y0 && pp.y <= y1 && p != a && p != cc && point_in_triangle(ax, ay, bx, by, cx, cy, pp.x, pp.y) && c.area(pp.prev, p, pp.next) >= 0.0 { return false; }
        p = pp.prev_z;
    }
    while n != NIL && c.n(n).z <= max_z {
        let nn = c.n(n);
        if nn.x >= x0 && nn.x <= x1 && nn.y >= y0 && nn.y <= y1 && n != a && n != cc && point_in_triangle(ax, ay, bx, by, cx, cy, nn.x, nn.y) && c.area(nn.prev, n, nn.next) >= 0.0 { return false; }
        n = nn.next_z;
    }
    true
}

fn cure_local_intersections(c: &mut Ctx, start: usize, tris: &mut Vec<usize>, dim: usize) -> usize {
    let mut start = start;
    let mut p = start;
    loop {
        let a = c.n(p).prev; let b = c.n(c.n(p).next).next;
        let pn = c.n(p).next;
        if !c.eq(a, b) && intersects(c, a, p, pn, b) && locally_inside(c, a, b) && locally_inside(c, b, a) {
            tris.push(c.n(a).i / dim); tris.push(c.n(p).i / dim); tris.push(c.n(b).i / dim);
            c.remove(p);
            let pn2 = c.n(p).next;
            c.remove(pn2);
            p = b; start = b;
        }
        p = c.n(p).next;
        if p == start { break; }
    }
    filter_points(c, p, NIL)
}

fn split_earcut(c: &mut Ctx, start: usize, tris: &mut Vec<usize>, dim: usize, min_x: f64, min_y: f64, inv: f64) {
    let mut a = start;
    loop {
        let mut b = c.n(c.n(a).next).next;
        while b != c.n(a).prev {
            if c.n(a).i != c.n(b).i && is_valid_diagonal(c, a, b) {
                let mut cc = split_polygon(c, a, b);
                let an = c.n(a).next;
                let a2 = filter_points(c, a, an);
                let ccn = c.n(cc).next;
                cc = filter_points(c, cc, ccn);
                earcut_linked(c, a2, tris, dim, min_x, min_y, inv, 0);
                earcut_linked(c, cc, tris, dim, min_x, min_y, inv, 0);
                return;
            }
            b = c.n(b).next;
        }
        a = c.n(a).next;
        if a == start { break; }
    }
}

fn eliminate_holes(c: &mut Ctx, data: &[f64], hole_indices: &[usize], outer: usize, dim: usize) -> usize {
    let mut queue = vec![];
    for (i, &hi) in hole_indices.iter().enumerate() {
        let start = hi * dim;
        let end = if i < hole_indices.len() - 1 { hole_indices[i + 1] * dim } else { data.len() };
        let list = linked_list(c, data, start, end, dim, false);
        if list == c.n(list).next { c.nm(list).steiner = true; }
        queue.push(get_leftmost(c, list));
    }
    queue.sort_by(|&a, &b| c.n(a).x.partial_cmp(&c.n(b).x).unwrap_or(std::cmp::Ordering::Equal));
    let mut outer = outer;
    for q in queue { outer = eliminate_hole(c, q, outer); }
    outer
}

fn eliminate_hole(c: &mut Ctx, hole: usize, outer: usize) -> usize {
    let bridge = find_hole_bridge(c, hole, outer);
    if bridge == NIL { return outer; }
    let bridge_reverse = split_polygon(c, bridge, hole);
    let brn = c.n(bridge_reverse).next;
    filter_points(c, bridge_reverse, brn);
    let bn = c.n(bridge).next;
    filter_points(c, bridge, bn)
}

fn find_hole_bridge(c: &Ctx, hole: usize, outer: usize) -> usize {
    let mut p = outer;
    let (hx, hy) = (c.n(hole).x, c.n(hole).y);
    let mut qx = f64::NEG_INFINITY;
    let mut m = NIL;
    loop {
        let pn = c.n(p); let nn = c.n(pn.next);
        if hy <= pn.y && hy >= nn.y && nn.y != pn.y {
            let x = pn.x + (hy - pn.y) * (nn.x - pn.x) / (nn.y - pn.y);
            if x <= hx && x > qx {
                qx = x;
                m = if pn.x < nn.x { p } else { pn.next };
                if x == hx { return m; }
            }
        }
        p = pn.next;
        if p == outer { break; }
    }
    if m == NIL { return NIL; }
    let stop = m;
    let (mx, my) = (c.n(m).x, c.n(m).y);
    let mut tan_min = f64::INFINITY;
    p = m;
    loop {
        let pn = c.n(p);
        if hx >= pn.x && pn.x >= mx && hx != pn.x
            && point_in_triangle(if hy < my { hx } else { qx }, hy, mx, my, if hy < my { qx } else { hx }, hy, pn.x, pn.y) {
            let tan = (hy - pn.y).abs() / (hx - pn.x);
            if locally_inside(c, p, hole) && (tan < tan_min || (tan == tan_min && (pn.x > c.n(m).x || (pn.x == c.n(m).x && sector_contains_sector(c, m, p))))) {
                m = p; tan_min = tan;
            }
        }
        p = pn.next;
        if p == stop { break; }
    }
    m
}

fn sector_contains_sector(c: &Ctx, m: usize, p: usize) -> bool {
    c.area(c.n(m).prev, m, c.n(p).prev) < 0.0 && c.area(c.n(p).next, m, c.n(m).next) < 0.0
}

fn index_curve(c: &mut Ctx, start: usize, min_x: f64, min_y: f64, inv: f64) {
    let mut p = start;
    loop {
        if c.n(p).z == 0 { let (x, y) = (c.n(p).x, c.n(p).y); c.nm(p).z = z_order(x, y, min_x, min_y, inv); }
        let pp = c.n(p).prev; let pn = c.n(p).next;
        c.nm(p).prev_z = pp;
        c.nm(p).next_z = pn;
        p = pn;
        if p == start { break; }
    }
    let pz = c.n(p).prev_z;
    c.nm(pz).next_z = NIL;
    c.nm(p).prev_z = NIL;
    sort_linked(c, p);
}

fn sort_linked(c: &mut Ctx, list: usize) -> usize {
    let mut list = list;
    let mut in_size = 1;
    loop {
        let mut p = list;
        list = NIL;
        let mut tail = NIL;
        let mut num_merges = 0;
        while p != NIL {
            num_merges += 1;
            let mut q = p;
            let mut p_size = 0;
            for _ in 0..in_size { p_size += 1; q = c.n(q).next_z; if q == NIL { break; } }
            let mut q_size = in_size;
            while p_size > 0 || (q_size > 0 && q != NIL) {
                let e;
                if p_size != 0 && (q_size == 0 || q == NIL || c.n(p).z <= c.n(q).z) {
                    e = p; p = c.n(p).next_z; p_size -= 1;
                } else {
                    e = q; q = c.n(q).next_z; q_size -= 1;
                }
                if tail != NIL { c.nm(tail).next_z = e; } else { list = e; }
                c.nm(e).prev_z = tail;
                tail = e;
            }
            p = q;
        }
        c.nm(tail).next_z = NIL;
        in_size *= 2;
        if num_merges <= 1 { break; }
    }
    list
}

fn z_order(x: f64, y: f64, min_x: f64, min_y: f64, inv: f64) -> i64 {
    let mut x = (((x - min_x) * inv) as i64) & 0xffffffff;
    let mut y = (((y - min_y) * inv) as i64) & 0xffffffff;
    x = (x | (x << 8)) & 0x00FF00FF; x = (x | (x << 4)) & 0x0F0F0F0F; x = (x | (x << 2)) & 0x33333333; x = (x | (x << 1)) & 0x55555555;
    y = (y | (y << 8)) & 0x00FF00FF; y = (y | (y << 4)) & 0x0F0F0F0F; y = (y | (y << 2)) & 0x33333333; y = (y | (y << 1)) & 0x55555555;
    x | (y << 1)
}

fn get_leftmost(c: &Ctx, start: usize) -> usize {
    let mut p = start; let mut left = start;
    loop {
        if c.n(p).x < c.n(left).x || (c.n(p).x == c.n(left).x && c.n(p).y < c.n(left).y) { left = p; }
        p = c.n(p).next;
        if p == start { break; }
    }
    left
}

fn point_in_triangle(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64, px: f64, py: f64) -> bool {
    (cx - px) * (ay - py) >= (ax - px) * (cy - py) && (ax - px) * (by - py) >= (bx - px) * (ay - py) && (bx - px) * (cy - py) >= (cx - px) * (by - py)
}

fn is_valid_diagonal(c: &Ctx, a: usize, b: usize) -> bool {
    let (an, ap, bn, bp) = (c.n(a).next, c.n(a).prev, c.n(b).next, c.n(b).prev);
    c.n(an).i != c.n(b).i && c.n(ap).i != c.n(b).i && !intersects_polygon(c, a, b)
        && ((locally_inside(c, a, b) && locally_inside(c, b, a) && middle_inside(c, a, b) && (c.area(ap, a, bp) != 0.0 || c.area(a, bp, b) != 0.0))
            || (c.eq(a, b) && c.area(ap, a, an) > 0.0 && c.area(bp, b, bn) > 0.0))
}

fn intersects(c: &Ctx, p1: usize, q1: usize, p2: usize, q2: usize) -> bool {
    let o1 = sign(c.area(p1, q1, p2)); let o2 = sign(c.area(p1, q1, q2));
    let o3 = sign(c.area(p2, q2, p1)); let o4 = sign(c.area(p2, q2, q1));
    if o1 != o2 && o3 != o4 { return true; }
    if o1 == 0 && on_segment(c, p1, p2, q1) { return true; }
    if o2 == 0 && on_segment(c, p1, q2, q1) { return true; }
    if o3 == 0 && on_segment(c, p2, p1, q2) { return true; }
    if o4 == 0 && on_segment(c, p2, q1, q2) { return true; }
    false
}
fn on_segment(c: &Ctx, p: usize, q: usize, r: usize) -> bool {
    let (p, q, r) = (c.n(p), c.n(q), c.n(r));
    q.x <= p.x.max(r.x) && q.x >= p.x.min(r.x) && q.y <= p.y.max(r.y) && q.y >= p.y.min(r.y)
}
fn sign(v: f64) -> i32 { if v > 0.0 { 1 } else if v < 0.0 { -1 } else { 0 } }

fn intersects_polygon(c: &Ctx, a: usize, b: usize) -> bool {
    let mut p = a;
    loop {
        let pn = c.n(p).next;
        if c.n(p).i != c.n(a).i && c.n(pn).i != c.n(a).i && c.n(p).i != c.n(b).i && c.n(pn).i != c.n(b).i && intersects(c, p, pn, a, b) { return true; }
        p = pn;
        if p == a { break; }
    }
    false
}
fn locally_inside(c: &Ctx, a: usize, b: usize) -> bool {
    let (ap, an) = (c.n(a).prev, c.n(a).next);
    if c.area(ap, a, an) < 0.0 { c.area(a, b, an) >= 0.0 && c.area(a, ap, b) >= 0.0 } else { c.area(a, b, ap) < 0.0 || c.area(a, an, b) < 0.0 }
}
fn middle_inside(c: &Ctx, a: usize, b: usize) -> bool {
    let mut p = a; let mut inside = false;
    let px = (c.n(a).x + c.n(b).x) / 2.0; let py = (c.n(a).y + c.n(b).y) / 2.0;
    loop {
        let pn = c.n(p); let nn = c.n(pn.next);
        if ((pn.y > py) != (nn.y > py)) && nn.y != pn.y && (px < (nn.x - pn.x) * (py - pn.y) / (nn.y - pn.y) + pn.x) { inside = !inside; }
        p = pn.next;
        if p == a { break; }
    }
    inside
}
fn split_polygon(c: &mut Ctx, a: usize, b: usize) -> usize {
    let a2 = c.nodes.len();
    let an = c.n(a).clone();
    c.nodes.push(Node { i: an.i, x: an.x, y: an.y, prev: NIL, next: NIL, z: 0, prev_z: NIL, next_z: NIL, steiner: false });
    let b2 = c.nodes.len();
    let bn = c.n(b).clone();
    c.nodes.push(Node { i: bn.i, x: bn.x, y: bn.y, prev: NIL, next: NIL, z: 0, prev_z: NIL, next_z: NIL, steiner: false });
    let an_next = c.n(a).next; let bp = c.n(b).prev;
    c.nm(a).next = b; c.nm(b).prev = a;
    c.nm(a2).next = an_next; c.nm(an_next).prev = a2;
    c.nm(b2).next = a2; c.nm(a2).prev = b2;
    c.nm(bp).next = b2; c.nm(b2).prev = bp;
    b2
}
