use crate::BackendCoord;

fn get_dir_vector(from: BackendCoord, to: BackendCoord, flag: bool) -> ((f64, f64), (f64, f64)) {
    let v = (i64::from(to.0 - from.0), i64::from(to.1 - from.1));
    let l = ((v.0 * v.0 + v.1 * v.1) as f64).sqrt();

    let v = (v.0 as f64 / l, v.1 as f64 / l);

    if flag {
        (v, (v.1, -v.0))
    } else {
        (v, (-v.1, v.0))
    }
}

fn compute_polygon_vertex(triple: &[BackendCoord; 3], d: f64) -> BackendCoord {
    let (a_t, a_n) = get_dir_vector(triple[0], triple[1], false);
    let (b_t, b_n) = get_dir_vector(triple[2], triple[1], true);

    let a_p = (
        f64::from(triple[1].0) + d * a_n.0,
        f64::from(triple[1].1) + d * a_n.1,
    );
    let b_p = (
        f64::from(triple[1].0) + d * b_n.0,
        f64::from(triple[1].1) + d * b_n.1,
    );

    // u * a_t + a_p = v * b_t + b_p
    // u * a_t.0 - v * b_t.0 = b_p.0 - a_p.0
    // u * a_t.1 - v * b_t.1 = b_p.1 - a_p.1
    if a_p.0 as i32 == b_p.0 as i32 && a_p.1 as i32 == b_p.1 as i32 {
        return (a_p.0 as i32, a_p.1 as i32);
    }

    let a0 = a_t.0;
    let b0 = -b_t.0;
    let c0 = b_p.0 - a_p.0;
    let a1 = a_t.1;
    let b1 = -b_t.1;
    let c1 = b_p.1 - a_p.1;

    // This is the coner case that
    if (a0 * b1 - a1 * b0).abs() < 1e-10 {
        return (a_p.0 as i32, a_p.1 as i32);
    }

    let u = (c0 * b1 - c1 * b0) / (a0 * b1 - a1 * b0);

    let x = a_p.0 + u * a_t.0;
    let y = a_p.1 + u * a_t.1;

    (x.round() as i32, y.round() as i32)
}

fn traverse_vertices<'a>(
    mut vertices: impl Iterator<Item = &'a BackendCoord>,
    width: u32,
    mut op: impl FnMut(BackendCoord),
) {
    let mut a = vertices.next().unwrap();
    let mut b = vertices.next().unwrap();

    while a == b {
        a = b;
        if let Some(new_b) = vertices.next() {
            b = new_b;
        } else {
            return;
        }
    }

    let (_, n) = get_dir_vector(*a, *b, false);

    op((
        (f64::from(a.0) + n.0 * f64::from(width) / 2.0).round() as i32,
        (f64::from(a.1) + n.1 * f64::from(width) / 2.0).round() as i32,
    ));

    let mut recent = [(0, 0), *a, *b];

    for p in vertices {
        if *p == recent[2] {
            continue;
        }
        recent.swap(0, 1);
        recent.swap(1, 2);
        recent[2] = *p;
        op(compute_polygon_vertex(&recent, f64::from(width) / 2.0));
    }

    let b = recent[1];
    let a = recent[2];

    let (_, n) = get_dir_vector(a, b, true);

    op((
        (f64::from(a.0) + n.0 * f64::from(width) / 2.0).round() as i32,
        (f64::from(a.1) + n.1 * f64::from(width) / 2.0).round() as i32,
    ));
}

/// Covert a path with >1px stroke width into polygon.
pub fn polygonize(vertices: &[BackendCoord], stroke_width: u32) -> Vec<BackendCoord> {
    if vertices.len() < 2 {
        return vec![];
    }

    let mut ret = vec![];

    traverse_vertices(vertices.iter(), stroke_width, |v| ret.push(v));
    traverse_vertices(vertices.iter().rev(), stroke_width, |v| ret.push(v));

    ret
}
