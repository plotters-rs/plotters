use crate::{
    math_guard::{checked_add_i64, checked_mul_i64, checked_sub_i32, non_zero_f64},
    BackendCoord, MathError,
};
use std::convert::From;

// Compute the tanginal and normal vectors of the given straight line.
fn get_dir_vector(
    from: BackendCoord,
    to: BackendCoord,
    flag: bool,
) -> Result<((f64, f64), (f64, f64)), MathError> {
    let dx = i64::from(checked_sub_i32(to.0, from.0)?);
    let dy = i64::from(checked_sub_i32(to.1, from.1)?);

    let x2 = checked_mul_i64(dx, dx)?;
    let y2 = checked_mul_i64(dy, dy)?;
    let sum = checked_add_i64(x2, y2)? as f64;

    let len = non_zero_f64(sum.sqrt())?;

    let v = (dx as f64 / len, dy as f64 / len);

    Ok(if flag {
        (v, (v.1, -v.0))
    } else {
        (v, (-v.1, v.0))
    })
}

// Compute the polygonized vertex of the given angle
// d is the distance between the polygon edge and the actual line.
// d can be negative, this will emit a vertex on the other side of the line.
fn compute_polygon_vertex(
    triple: &[BackendCoord; 3],
    d: f64,
    buf: &mut Vec<BackendCoord>,
) -> Result<(), MathError> {
    buf.clear();

    // Compute the tanginal and normal vectors of the given straight line.
    let (a_t, a_n) = get_dir_vector(triple[0], triple[1], false)?;
    let (b_t, b_n) = get_dir_vector(triple[2], triple[1], true)?;

    // Compute a point that is d away from the line for line a and line b.
    let a_p = (
        f64::from(triple[1].0) + d * a_n.0,
        f64::from(triple[1].1) + d * a_n.1,
    );
    let b_p = (
        f64::from(triple[1].0) + d * b_n.0,
        f64::from(triple[1].1) + d * b_n.1,
    );

    // Check if 3 points are colinear, up to precision. If so, just emit the point.
    if (a_t.1 * b_t.0 - a_t.0 * b_t.1).abs() <= f64::EPSILON {
        buf.push((a_p.0 as i32, a_p.1 as i32));
        return Ok(());
    }

    // So we are actually computing the intersection of two lines:
    // a_p + u * a_t and b_p + v * b_t.
    // We can solve the following vector equation:
    // u * a_t + a_p = v * b_t + b_p
    //
    // which is actually a equation system:
    // u * a_t.0 - v * b_t.0 = b_p.0 - a_p.0
    // u * a_t.1 - v * b_t.1 = b_p.1 - a_p.1

    // The following vars are coefficients of the linear equation system.
    // a0*u + b0*v = c0
    // a1*u + b1*v = c1
    // in which x and y are the coordinates that two polygon edges intersect.

    let a0 = a_t.0;
    let b0 = -b_t.0;
    let c0 = b_p.0 - a_p.0;
    let a1 = a_t.1;
    let b1 = -b_t.1;
    let c1 = b_p.1 - a_p.1;

    // Since the points are not collinear, the determinant is not 0, and we can get a intersection point.
    let u = (c0 * b1 - c1 * b0) / (a0 * b1 - a1 * b0);
    let x = a_p.0 + u * a_t.0;
    let y = a_p.1 + u * a_t.1;

    let cross_product = a_t.0 * b_t.1 - a_t.1 * b_t.0;
    if (cross_product < 0.0 && d < 0.0) || (cross_product > 0.0 && d > 0.0) {
        // Then we are at the outer side of the angle, so we need to consider a cap.
        let dist_square = (x - triple[1].0 as f64).powi(2) + (y - triple[1].1 as f64).powi(2);
        // If the point is too far away from the line, we need to cap it.
        if dist_square > d * d * 16.0 {
            buf.push((a_p.0.round() as i32, a_p.1.round() as i32));
            buf.push((b_p.0.round() as i32, b_p.1.round() as i32));
            return Ok(());
        }
    }

    buf.push((x.round() as i32, y.round() as i32));
    Ok(())
}

fn traverse_vertices<'a>(
    mut vertices: impl Iterator<Item = &'a BackendCoord>,
    width: u32,
    mut op: impl FnMut(BackendCoord),
) -> Result<(), MathError> {
    let mut a = vertices.next().unwrap();
    let mut b = vertices.next().unwrap();

    while a == b {
        a = b;
        if let Some(new_b) = vertices.next() {
            b = new_b;
        } else {
            return Ok(());
        }
    }

    let (_, n) = get_dir_vector(*a, *b, false)?;

    op((
        (f64::from(a.0) + n.0 * f64::from(width) / 2.0).round() as i32,
        (f64::from(a.1) + n.1 * f64::from(width) / 2.0).round() as i32,
    ));

    let mut recent = [(0, 0), *a, *b];
    let mut vertex_buf = Vec::with_capacity(3);

    for p in vertices {
        if *p == recent[2] {
            continue;
        }
        recent.swap(0, 1);
        recent.swap(1, 2);
        recent[2] = *p;
        compute_polygon_vertex(&recent, f64::from(width) / 2.0, &mut vertex_buf)?;
        vertex_buf.iter().cloned().for_each(&mut op);
    }

    let b = recent[1];
    let a = recent[2];

    let (_, n) = get_dir_vector(a, b, true)?;

    op((
        (f64::from(a.0) + n.0 * f64::from(width) / 2.0).round() as i32,
        (f64::from(a.1) + n.1 * f64::from(width) / 2.0).round() as i32,
    ));
    Ok(())
}

/// Covert a path with >1px stroke width into polygon.
pub fn polygonize(
    vertices: &[BackendCoord],
    stroke_width: u32,
) -> Result<Vec<BackendCoord>, MathError> {
    if vertices.len() < 2 {
        return Ok(vec![]);
    }

    let mut ret = vec![];

    traverse_vertices(vertices.iter(), stroke_width, |v| ret.push(v))?;
    traverse_vertices(vertices.iter().rev(), stroke_width, |v| ret.push(v))?;

    Ok(ret)
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test for regression with respect to https://github.com/plotters-rs/plotters/issues/562
    #[test]
    fn test_no_inf_in_compute_polygon_vertex() {
        let path = [(335, 386), (338, 326), (340, 286)];
        let mut buf = Vec::new();
        compute_polygon_vertex(&path, 2.0, buf.as_mut()).unwrap();
        assert!(!buf.is_empty());
        let nani32 = f64::INFINITY as i32;
        assert!(!buf.iter().any(|&v| v.0 == nani32 || v.1 == nani32));
    }

    /// Correct 90 degree turn to the right
    #[test]
    fn standard_corner() {
        let path = [(10, 10), (20, 10), (20, 20)];
        let mut buf = Vec::new();
        compute_polygon_vertex(&path, 2.0, buf.as_mut()).unwrap();
        assert!(!buf.is_empty());
        let buf2 = vec![(18, 12)];
        assert_eq!(buf, buf2);
    }
    #[test]
    fn get_dir_vector_rejects_zero_length_line() {
        assert_eq!(
            get_dir_vector((10, 10), (10, 10), false),
            Err(MathError::ZeroDivision)
        );
    }

    #[test]
    fn get_dir_vector_rejects_extreme_delta_out_of_range() {
        assert_eq!(
            get_dir_vector((i32::MIN, 0), (i32::MAX, 0), false),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn get_dir_vector_returns_unit_tangent_for_horizontal_line() {
        let result = get_dir_vector((0, 0), (10, 0), false);

        assert_eq!(result, Ok(((1.0, 0.0), (-0.0, 1.0))));
    }

    #[test]
    fn get_dir_vector_returns_unit_tangent_for_vertical_line() {
        let result = get_dir_vector((0, 0), (0, 10), false);

        assert_eq!(result, Ok(((0.0, 1.0), (-1.0, 0.0))));
    }

    #[test]
    fn compute_polygon_vertex_handles_colinear_points() {
        let path = [(0, 0), (10, 0), (20, 0)];
        let mut buf = Vec::new();

        assert_eq!(compute_polygon_vertex(&path, 2.0, &mut buf), Ok(()));
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn compute_polygon_vertex_rejects_repeated_adjacent_point() {
        let path = [(10, 10), (10, 10), (20, 20)];
        let mut buf = Vec::new();

        assert_eq!(
            compute_polygon_vertex(&path, 2.0, &mut buf),
            Err(MathError::ZeroDivision)
        );
    }

    #[test]
    fn traverse_vertices_with_only_repeated_points_returns_ok() {
        let vertices = [(1, 1), (1, 1), (1, 1)];
        let mut out = Vec::new();

        assert_eq!(
            traverse_vertices(vertices.iter(), 2, |v| out.push(v)),
            Ok(())
        );

        assert!(out.is_empty());
    }

    #[test]
    fn polygonize_returns_empty_for_less_than_two_vertices() {
        assert_eq!(polygonize(&[], 2), Ok(vec![]));
        assert_eq!(polygonize(&[(1, 1)], 2), Ok(vec![]));
    }

    #[test]
    fn polygonize_returns_vertices_for_simple_line() {
        let out = polygonize(&[(10, 10), (20, 10)], 2)
            .expect("polygonize should succeed for a simple line");

        assert!(!out.is_empty());
    }

    #[test]
    fn polygonize_rejects_extreme_coordinate_span() {
        assert_eq!(
            polygonize(&[(i32::MIN, 0), (i32::MAX, 0)], 2),
            Err(MathError::ValueOutOfRange)
        );
    }
}
