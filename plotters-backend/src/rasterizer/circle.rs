use crate::math_errors::MathError;
use crate::math_guard::{
    ceil_f64_to_i32, checked_add_i32, checked_add_u32, checked_div_u32, checked_neg_i32,
    checked_sub_i32, checked_sub_u32, f64_to_i32_checked, floor_f64_to_i32, sqrt_f64_checked,
    u32_to_i32_checked,
};
use crate::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
fn draw_part_a<
    B: DrawingBackend,
    Draw: FnMut(i32, (f64, f64)) -> Result<(), DrawingErrorKind<B::ErrorType>>,
>(
    height: f64,
    radius: u32,
    mut draw: Draw,
) -> Result<(), DrawingErrorKind<B::ErrorType>> {
    let half_width = (radius as f64 * radius as f64
        - (radius as f64 - height) * (radius as f64 - height))
        .sqrt();

    let x0 = ceil_f64_to_i32(half_width)?;
    let x1 = floor_f64_to_i32(half_width)?;

    let y0 = (radius as f64 - height).ceil();

    for x in x0..=x1 {
        let y1 = (radius as f64 * radius as f64 - x as f64 * x as f64).sqrt();
        check_result!(draw(x, (y0, y1)));
    }

    Ok(())
}

fn draw_part_b<
    B: DrawingBackend,
    Draw: FnMut(i32, (f64, f64)) -> Result<(), DrawingErrorKind<B::ErrorType>>,
>(
    from: f64,
    size: f64,
    mut draw: Draw,
) -> Result<(), DrawingErrorKind<B::ErrorType>> {
    let len = floor_f64_to_i32(from - size)?;
    let from = floor_f64_to_i32(from)?;
    for x in len..=from {
        let neg_x = checked_neg_i32(x)?;
        check_result!(draw(x, (f64::from(neg_x), f64::from(x))));
    }
    Ok(())
}

fn draw_part_c<
    B: DrawingBackend,
    Draw: FnMut(i32, (f64, f64)) -> Result<(), DrawingErrorKind<B::ErrorType>>,
>(
    r: i32,
    r_limit: i32,
    mut draw: Draw,
) -> Result<(), DrawingErrorKind<B::ErrorType>> {
    if r < 0 || r_limit < 0 || r > r_limit {
        return Err(MathError::ValueOutOfRange.into());
    }
    let r_f = f64::from(r);
    let r_limit_f = f64::from(r_limit);
    let half_size = r_f / (2f64).sqrt();

    let x0 = ceil_f64_to_i32(-half_size)?;
    let x1 = floor_f64_to_i32(half_size)?;

    for x in x0..x1 {
        let x_f = f64::from(x);
        let outer_y0 = sqrt_f64_checked(r_limit_f * r_limit_f - x_f * x_f)?;
        let inner_y0 = r_f - 1.0;
        let mut y1 = outer_y0.min(inner_y0);
        let y0 = sqrt_f64_checked(r_f * r_f - x_f * x_f)?;

        if y0 > y1 {
            y1 = y0.ceil();
            if y1 >= r_f {
                continue;
            }
        }

        check_result!(draw(x, (y0, y1)));
    }
    let start = checked_add_i32(x1, 1)?;
    let end = checked_add_i32(x1, r)?;
    for x in start..end {
        let x_f = f64::from(x);
        let outer_radicand = r_limit_f * r_limit_f - x_f * x_f;
        if outer_radicand < 0.0 {
            continue;
        }
        let outer_y0 = sqrt_f64_checked(outer_radicand)?;
        let inner_y0 = r_f - 1.0;
        let y0 = outer_y0.min(inner_y0);
        let y1 = x_f;

        if y1 < y0 {
            check_result!(draw(x, (y0, y1 + 1.0)));
            let neg_x = checked_neg_i32(x)?;
            check_result!(draw(neg_x, (y0, y1 + 1.0)));
        }
    }

    Ok(())
}

fn draw_sweep_line<B: DrawingBackend, S: BackendStyle>(
    b: &mut B,
    style: &S,
    (x0, y0): BackendCoord,
    (dx, dy): (i32, i32),
    p0: i32,
    (s, e): (f64, f64),
) -> Result<(), DrawingErrorKind<B::ErrorType>> {
    let mut s = if dx < 0 || dy < 0 { -s } else { s };
    let mut e = if dx < 0 || dy < 0 { -e } else { e };
    if !s.is_finite() || !e.is_finite() {
        return Err(MathError::NonFiniteCalculation.into());
    }
    if s > e {
        std::mem::swap(&mut s, &mut e);
    }

    let s_ceil = ceil_f64_to_i32(s)?;
    let e_floor = floor_f64_to_i32(e)?;

    let vs = s.ceil() - s;
    let ve = e - e.floor();

    if dx == 0 {
        let px0 = checked_add_i32(p0, x0)?;
        let sy0 = checked_add_i32(s_ceil, y0)?;
        let ey0 = checked_add_i32(e_floor, y0)?;

        check_result!(b.draw_line((px0, sy0), (px0, ey0), &style.color()));

        let sy0_sub_1 = checked_sub_i32(sy0, 1)?;
        let ey0_add_1 = checked_add_i32(ey0, 1)?;

        check_result!(b.draw_pixel((px0, sy0_sub_1), style.color().mix(vs)));
        check_result!(b.draw_pixel((px0, ey0_add_1), style.color().mix(ve)));
    } else {
        let sx0 = checked_add_i32(s_ceil, x0)?;
        let py0 = checked_add_i32(p0, y0)?;
        let ex0 = checked_add_i32(e_floor, x0)?;

        check_result!(b.draw_line((sx0, py0), (ex0, py0), &style.color()));

        let sx0_sub_1 = checked_sub_i32(sx0, 1)?;
        let ex0_add_1 = checked_add_i32(ex0, 1)?;

        check_result!(b.draw_pixel((sx0_sub_1, py0), style.color().mix(vs)));
        check_result!(b.draw_pixel((ex0_add_1, py0), style.color().mix(ve)));
    }

    Ok(())
}

fn draw_annulus<B: DrawingBackend, S: BackendStyle>(
    b: &mut B,
    center: BackendCoord,
    radius: (u32, u32),
    style: &S,
) -> Result<(), DrawingErrorKind<B::ErrorType>> {
    let radius0_f = f64::from(radius.0);
    let radius1_f = f64::from(radius.1);

    let radius0_i32 = u32_to_i32_checked(radius.0)?;
    let radius1_i32 = u32_to_i32_checked(radius.1)?;

    let rad_sub = f64::from(checked_sub_u32(radius.0, radius.1)?);
    let a0 = rad_sub.min(radius0_f * (1.0 - 1.0 / (2f64).sqrt()));
    let a1 = (radius0_f - a0 - radius1_f).max(0.0);

    check_result!(draw_part_a::<B, _>(a0, radius.0, |p, r| {
        draw_sweep_line(b, style, center, (0, 1), p, r)
    }));

    check_result!(draw_part_a::<B, _>(a0, radius.0, |p, r| {
        draw_sweep_line(b, style, center, (0, -1), p, r)
    }));

    check_result!(draw_part_a::<B, _>(a0, radius.0, |p, r| {
        draw_sweep_line(b, style, center, (1, 0), p, r)
    }));

    check_result!(draw_part_a::<B, _>(a0, radius.0, |p, r| {
        draw_sweep_line(b, style, center, (-1, 0), p, r)
    }));

    if a1 > 0.0 {
        check_result!(draw_part_b::<B, _>(
            radius0_f - a0,
            a1.floor(),
            |h, (f, t)| {
                let f = f64_to_i32_checked(f)?;
                let t = f64_to_i32_checked(t)?;

                let center_h = checked_add_i32(center.0, h)?;
                let center_f = checked_add_i32(center.1, f)?;
                let center_t = checked_add_i32(center.1, t)?;

                check_result!(b.draw_line(
                    (center_h, center_f),
                    (center_h, center_t),
                    &style.color()
                ));

                let center_sub_h = checked_sub_i32(center.0, h)?;

                check_result!(b.draw_line(
                    (center_sub_h, center_f),
                    (center_sub_h, center_t),
                    &style.color()
                ));

                let center0_f = checked_add_i32(center.0, f)?;
                let center0_f1 = checked_add_i32(center0_f, 1)?;
                let center0_t = checked_add_i32(center.0, t)?;
                let center0_tsub1 = checked_sub_i32(center0_t, 1)?;
                let center1_h = checked_add_i32(center.1, h)?;

                check_result!(b.draw_line(
                    (center0_f1, center1_h),
                    (center0_tsub1, center1_h),
                    &style.color()
                ));

                let center1_sub_h = checked_sub_i32(center.1, h)?;

                check_result!(b.draw_line(
                    (center0_f1, center1_sub_h),
                    (center0_tsub1, center1_sub_h),
                    &style.color()
                ));

                Ok(())
            }
        ));
    }

    check_result!(draw_part_c::<B, _>(radius1_i32, radius0_i32, |p, r| {
        draw_sweep_line(b, style, center, (0, 1), p, r)
    }));

    check_result!(draw_part_c::<B, _>(radius1_i32, radius0_i32, |p, r| {
        draw_sweep_line(b, style, center, (0, -1), p, r)
    }));

    check_result!(draw_part_c::<B, _>(radius1_i32, radius0_i32, |p, r| {
        draw_sweep_line(b, style, center, (1, 0), p, r)
    }));

    check_result!(draw_part_c::<B, _>(radius1_i32, radius0_i32, |p, r| {
        draw_sweep_line(b, style, center, (-1, 0), p, r)
    }));

    let d_inner = floor_f64_to_i32(radius1_f / (2f64).sqrt())?;

    let d_outer_limit = checked_sub_i32(radius1_i32, 1)?;
    let d_outer = floor_f64_to_i32(radius0_f / (2f64).sqrt())?.min(d_outer_limit);

    let d_outer_actual_value =
        sqrt_f64_checked(radius0_f * radius0_f - radius1_f * radius1_f / 2.0)?;
    let d_outer_actually = radius1_i32.min(ceil_f64_to_i32(d_outer_actual_value)?);

    let cx_sub_d_inner = checked_sub_i32(center.0, d_inner)?;
    let cx_add_d_inner = checked_add_i32(center.0, d_inner)?;
    let cy_sub_d_inner = checked_sub_i32(center.1, d_inner)?;
    let cy_add_d_inner = checked_add_i32(center.1, d_inner)?;

    let cx_sub_d_outer = checked_sub_i32(center.0, d_outer)?;
    let cx_add_d_outer = checked_add_i32(center.0, d_outer)?;
    let cy_sub_d_outer = checked_sub_i32(center.1, d_outer)?;
    let cy_add_d_outer = checked_add_i32(center.1, d_outer)?;

    let cx_sub_d_outer_actually = checked_sub_i32(center.0, d_outer_actually)?;
    let cx_add_d_outer_actually = checked_add_i32(center.0, d_outer_actually)?;
    let cy_sub_d_outer_actually = checked_sub_i32(center.1, d_outer_actually)?;
    let cy_add_d_outer_actually = checked_add_i32(center.1, d_outer_actually)?;

    check_result!(b.draw_line(
        (cx_sub_d_inner, cy_sub_d_inner),
        (cx_sub_d_outer, cy_sub_d_outer),
        &style.color()
    ));

    check_result!(b.draw_line(
        (cx_add_d_inner, cy_sub_d_inner),
        (cx_add_d_outer, cy_sub_d_outer),
        &style.color()
    ));

    check_result!(b.draw_line(
        (cx_sub_d_inner, cy_add_d_inner),
        (cx_sub_d_outer, cy_add_d_outer),
        &style.color()
    ));

    check_result!(b.draw_line(
        (cx_add_d_inner, cy_add_d_inner),
        (cx_add_d_outer, cy_add_d_outer),
        &style.color()
    ));

    check_result!(b.draw_line(
        (cx_sub_d_inner, cy_add_d_inner),
        (cx_sub_d_outer_actually, cy_add_d_inner),
        &style.color()
    ));

    check_result!(b.draw_line(
        (cx_add_d_inner, cy_sub_d_inner),
        (cx_add_d_inner, cy_sub_d_outer_actually),
        &style.color()
    ));

    check_result!(b.draw_line(
        (cx_add_d_inner, cy_add_d_inner),
        (cx_add_d_inner, cy_add_d_outer_actually),
        &style.color()
    ));

    check_result!(b.draw_line(
        (cx_add_d_inner, cy_add_d_inner),
        (cx_add_d_outer_actually, cy_add_d_inner),
        &style.color()
    ));

    Ok(())
}

pub fn draw_circle<B: DrawingBackend, S: BackendStyle>(
    b: &mut B,
    center: BackendCoord,
    mut radius: u32,
    style: &S,
    mut fill: bool,
) -> Result<(), DrawingErrorKind<B::ErrorType>> {
    if style.color().alpha == 0.0 {
        return Ok(());
    }

    if !fill && style.stroke_width() != 1 {
        let half_stroke = checked_div_u32(style.stroke_width(), 2)?;
        let inner_delta = half_stroke.min(radius);
        let inner_radius = checked_sub_u32(radius, inner_delta)?;
        radius = checked_add_u32(radius, half_stroke)?;
        if inner_radius > 0 {
            return draw_annulus(b, center, (radius, inner_radius), style);
        } else {
            fill = true;
        }
    }
    let radius_f = f64::from(radius);
    let radius_i32 = u32_to_i32_checked(radius)?;
    let sqrt_2 = (2f64).sqrt();
    let min = ceil_f64_to_i32(radius_f * (1.0 - sqrt_2 / 2.0))?;
    let max = floor_f64_to_i32(radius_f * (1.0 + sqrt_2 / 2.0))?;
    let up = checked_sub_i32(checked_add_i32(min, center.1)?, radius_i32)?;
    let down = checked_sub_i32(checked_add_i32(max, center.1)?, radius_i32)?;

    let range = min..=max;

    for dy in range {
        let dy = checked_sub_i32(dy, radius_i32)?;
        let dy_f = f64::from(dy);

        let y = checked_add_i32(center.1, dy)?;

        let lx = sqrt_f64_checked(radius_f * radius_f - (dy_f * dy_f).max(1e-5))?;
        let lx_floor = floor_f64_to_i32(lx)?;

        let left = checked_sub_i32(center.0, lx_floor)?;
        let right = checked_add_i32(center.0, lx_floor)?;

        let v = lx - lx.floor();

        let x = checked_add_i32(center.0, dy)?;
        let top = checked_sub_i32(center.1, lx_floor)?;
        let bottom = checked_add_i32(center.1, lx_floor)?;

        if fill {
            let up_minus_one = checked_sub_i32(up, 1)?;
            let down_plus_one = checked_add_i32(down, 1)?;

            check_result!(b.draw_line((left, y), (right, y), &style.color()));
            check_result!(b.draw_line((x, top), (x, up_minus_one), &style.color()));
            check_result!(b.draw_line((x, down_plus_one), (x, bottom), &style.color()));
        } else {
            let inverse_v = 1.0 - v;

            check_result!(b.draw_pixel((left, y), style.color().mix(inverse_v)));
            check_result!(b.draw_pixel((right, y), style.color().mix(inverse_v)));

            check_result!(b.draw_pixel((x, top), style.color().mix(inverse_v)));
            check_result!(b.draw_pixel((x, bottom), style.color().mix(inverse_v)));
        }

        let left_minus_one = checked_sub_i32(left, 1)?;
        let right_plus_one = checked_add_i32(right, 1)?;
        let top_minus_one = checked_sub_i32(top, 1)?;
        let bottom_plus_one = checked_add_i32(bottom, 1)?;

        check_result!(b.draw_pixel((left_minus_one, y), style.color().mix(v)));
        check_result!(b.draw_pixel((right_plus_one, y), style.color().mix(v)));
        check_result!(b.draw_pixel((x, top_minus_one), style.color().mix(v)));
        check_result!(b.draw_pixel((x, bottom_plus_one), style.color().mix(v)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BackendColor;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "test backend error")
        }
    }

    impl Error for TestError {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestOp {
        Pixel(BackendCoord),
        Line(BackendCoord, BackendCoord),
    }

    #[derive(Default)]
    struct TestBackend {
        ops: Vec<TestOp>,
    }

    impl DrawingBackend for TestBackend {
        type ErrorType = TestError;
        fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            Ok(())
        }
        fn get_size(&self) -> (u32, u32) {
            (100, 100)
        }

        fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            Ok(())
        }

        fn draw_pixel(
            &mut self,
            point: BackendCoord,
            _color: BackendColor,
        ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            self.ops.push(TestOp::Pixel(point));
            Ok(())
        }

        fn draw_line<S: BackendStyle>(
            &mut self,
            from: BackendCoord,
            to: BackendCoord,
            _style: &S,
        ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            self.ops.push(TestOp::Line(from, to));
            Ok(())
        }
    }

    #[derive(Clone, Copy)]
    struct TestStyle {
        color: BackendColor,
        stroke_width: u32,
    }

    impl TestStyle {
        fn new(stroke_width: u32, alpha: f64) -> Self {
            Self {
                color: BackendColor {
                    rgb: (0, 0, 0),
                    alpha,
                },
                stroke_width,
            }
        }
    }

    impl BackendStyle for TestStyle {
        fn color(&self) -> BackendColor {
            self.color
        }

        fn stroke_width(&self) -> u32 {
            self.stroke_width
        }
    }

    #[test]
    fn draw_part_a_draws_expected_integer_width_slice() {
        let mut points = Vec::new();

        draw_part_a::<TestBackend, _>(5.0, 5, |x, range| {
            points.push((x, range));
            Ok(())
        })
        .unwrap();

        assert_eq!(points, vec![(5, (0.0, 0.0))]);
    }

    #[test]
    fn draw_part_a_rejects_non_finite_height() {
        let result = draw_part_a::<TestBackend, _>(f64::NAN, 5, |_x, _range| Ok(()));

        assert!(result.is_err());
    }

    #[test]
    fn draw_part_b_draws_expected_reflected_ranges() {
        let mut points = Vec::new();

        draw_part_b::<TestBackend, _>(3.0, 2.0, |x, range| {
            points.push((x, range));
            Ok(())
        })
        .unwrap();

        assert_eq!(
            points,
            vec![(1, (-1.0, 1.0)), (2, (-2.0, 2.0)), (3, (-3.0, 3.0)),]
        );
    }

    #[test]
    fn draw_part_b_rejects_negation_overflow() {
        let result = draw_part_b::<TestBackend, _>(f64::from(i32::MIN), 0.0, |_x, _range| Ok(()));

        assert!(result.is_err());
    }

    #[test]
    fn draw_part_c_draws_points_for_valid_annulus_segment() {
        let mut points = Vec::new();

        draw_part_c::<TestBackend, _>(7, 8, |x, range| {
            points.push((x, range));
            Ok(())
        })
        .unwrap();

        assert!(!points.is_empty());

        for (_x, (y0, y1)) in points {
            assert!(y0.is_finite());
            assert!(y1.is_finite());
        }
    }

    #[test]
    fn draw_part_c_rejects_invalid_radius_geometry() {
        let result = draw_part_c::<TestBackend, _>(5, 3, |_x, _range| Ok(()));

        assert!(result.is_err());
    }

    #[test]
    fn draw_sweep_line_draws_vertical_line_and_edge_pixels() {
        let mut backend = TestBackend::default();
        let style = TestStyle::new(1, 1.0);

        draw_sweep_line(&mut backend, &style, (10, 20), (0, 1), 2, (1.2, 3.7)).unwrap();

        assert_eq!(
            backend.ops,
            vec![
                TestOp::Line((12, 22), (12, 23)),
                TestOp::Pixel((12, 21)),
                TestOp::Pixel((12, 24)),
            ]
        );
    }

    #[test]
    fn draw_sweep_line_draws_horizontal_line_and_edge_pixels() {
        let mut backend = TestBackend::default();
        let style = TestStyle::new(1, 1.0);

        draw_sweep_line(&mut backend, &style, (10, 20), (1, 0), 2, (1.2, 3.7)).unwrap();

        assert_eq!(
            backend.ops,
            vec![
                TestOp::Line((12, 22), (13, 22)),
                TestOp::Pixel((11, 22)),
                TestOp::Pixel((14, 22)),
            ]
        );
    }

    #[test]
    fn draw_sweep_line_rejects_non_finite_range() {
        let mut backend = TestBackend::default();
        let style = TestStyle::new(1, 1.0);

        let result = draw_sweep_line(&mut backend, &style, (10, 20), (0, 1), 2, (f64::NAN, 3.7));

        assert!(result.is_err());
        assert!(backend.ops.is_empty());
    }

    #[test]
    fn draw_annulus_rejects_inner_radius_larger_than_outer_radius() {
        let mut backend = TestBackend::default();
        let style = TestStyle::new(1, 1.0);

        let result = draw_annulus(&mut backend, (20, 20), (2, 3), &style);

        assert!(result.is_err());
        assert!(backend.ops.is_empty());
    }

    #[test]
    fn draw_circle_with_transparent_style_draws_nothing() {
        let mut backend = TestBackend::default();
        let style = TestStyle::new(1, 0.0);

        draw_circle(&mut backend, (20, 20), 5, &style, false).unwrap();

        assert!(backend.ops.is_empty());
    }

    #[test]
    fn draw_circle_outline_draws_pixels() {
        let mut backend = TestBackend::default();
        let style = TestStyle::new(1, 1.0);

        draw_circle(&mut backend, (20, 20), 5, &style, false).unwrap();

        assert!(!backend.ops.is_empty());
        assert!(backend.ops.iter().all(|op| matches!(op, TestOp::Pixel(_))));
    }

    #[test]
    fn draw_circle_fill_draws_lines_and_pixels() {
        let mut backend = TestBackend::default();
        let style = TestStyle::new(1, 1.0);

        draw_circle(&mut backend, (20, 20), 5, &style, true).unwrap();

        assert!(backend
            .ops
            .iter()
            .any(|op| matches!(op, TestOp::Line(_, _))));

        assert!(backend.ops.iter().any(|op| matches!(op, TestOp::Pixel(_))));
    }

    #[test]
    fn draw_circle_rejects_coordinate_overflow() {
        let mut backend = TestBackend::default();
        let style = TestStyle::new(1, 1.0);

        let result = draw_circle(&mut backend, (i32::MAX, 20), 5, &style, false);

        assert!(result.is_err());
    }

    #[test]
    fn draw_circle_rejects_radius_expansion_overflow() {
        let mut backend = TestBackend::default();
        let style = TestStyle::new(2, 1.0);

        let result = draw_circle(&mut backend, (20, 20), u32::MAX, &style, false);

        assert!(result.is_err());
        assert!(backend.ops.is_empty());
    }
}
