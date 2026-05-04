#![warn(clippy::arithmetic_side_effects)]
use crate::math_guard::{ceil_f64_to_i32, floor_f64_to_i32, sqrt_f64_checked, checked_neg_i32};
use crate::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
use crate::math_errors::MathError;
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
    let half_size = r as f64 / (2f64).sqrt();

    let x0 = ceil_f64_to_i32(-half_size)?;
    let x1 = floor_f64_to_i32(half_size)?;

    for x in x0..x1 {
        let outer_y0 = sqrt_f64_checked(r_limit_f * r_limit_f - x_f * x_f)?;
        let inner_y0 = r as f64 - 1.0;
        let mut y1 = outer_y0.min(inner_y0);
        let y0 = sqrt_f64_checked(r_f * r_f - x_f * x_f)?;

        if y0 > y1 {
            y1 = y0.ceil();
            if y1 >= r as f64 {
                continue;
            }
        }

        check_result!(draw(x, (y0, y1)));
    }
    let start = checked_add_i32(x1, 1)?;
    let end = checked_add_i32(x1, r)?;
    for x in start..end {
        let outer_y0 = sqrt_f64_checked(r_limit_f * r_limit_f - x_f * x_f)?;
        let inner_y0 = r as f64 - 1.0;
        let y0 = outer_y0.min(inner_y0);
        let y1 = x as f64;

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
    let rad_sub = checked_sub::<u32, MathError>(radius.0,radius.1, MathError::ValueOverflow)? as f64;
    let a0 = (rad_sub).min(radius.0 as f64 * (1.0 - 1.0 / (2f64).sqrt()));
    let a1 = (radius.0 as f64 - a0 - radius.1 as f64).max(0.0);

    check_result!(draw_part_a::<B, _>(a0, radius.0, |p, r| draw_sweep_line(
        b,
        style,
        center,
        (0, 1),
        p,
        r
    )));
    check_result!(draw_part_a::<B, _>(a0, radius.0, |p, r| draw_sweep_line(
        b,
        style,
        center,
        (0, -1),
        p,
        r
    )));
    check_result!(draw_part_a::<B, _>(a0, radius.0, |p, r| draw_sweep_line(
        b,
        style,
        center,
        (1, 0),
        p,
        r
    )));
    check_result!(draw_part_a::<B, _>(a0, radius.0, |p, r| draw_sweep_line(
        b,
        style,
        center,
        (-1, 0),
        p,
        r
    )));

    if a1 > 0.0 {
        check_result!(draw_part_b::<B, _>(
            radius.0 as f64 - a0,
            a1.floor(),
            |h, (f, t)| {
                let f = f as i32;
                let t = t as i32;
                let center_h = checked_add::<i32, MathError>(center.0, h, MathError::ValueUnderflow)?;
                let center_f = checked_add::<i32, MathError>(center.1, f, MathError::ValueUnderflow)?;
                let center_t = checked_add::<i32, MathError>(center.1, t, MathError::ValueUnderflow)?;
                check_result!(b.draw_line(
                    (center_h, center_f),
                    (center_h, center_t),
                    &style.color()
                ));
                let center_sub_h = checked_sub::<i32, MathError>(center.0, h, MathError::ValueOverflow)?;
                check_result!(b.draw_line(
                    (center_sub_h, center_f),
                    (center_sub_h, center_t),
                    &style.color()
                ));
                let center0_f = checked_add::<i32, MathError>(center.0, f, MathError::ValueUnderflow)?;
                let center0_f1 = checked_add::<i32, MathError>(center0_f, 1, MathError::ValueUnderflow)?;
                let center1_h = checked_add::<i32, MathError>(center.1, h, MathError::ValueUnderflow)?;
                check_result!(b.draw_line(
                    (center0_f1, center1_h),
                    (center.0 + t - 1, center1_h),
                    &style.color()
                ));
                let center1subh = checked_sub::<i32, MathError>(center.1, h, MathError::ValueOverflow)?;
                let center0_t = checked_add::<i32, MathError>(center.0, t, MathError::ValueUnderflow)?;
                let center0_tsub1 = checked_sub::<i32, MathError>(center0_t, 1, MathError::ValueOverflow)?;
                check_result!(b.draw_line(
                    (center0_f1, center1subh),
                    (center0_tsub1, center1subh),
                    &style.color()
                ));

                Ok(())
            }
        ));
    }

    check_result!(draw_part_c::<B, _>(
        radius.1 as i32,
        radius.0 as i32,
        |p, r| draw_sweep_line(b, style, center, (0, 1), p, r)
    ));
    check_result!(draw_part_c::<B, _>(
        radius.1 as i32,
        radius.0 as i32,
        |p, r| draw_sweep_line(b, style, center, (0, -1), p, r)
    ));
    check_result!(draw_part_c::<B, _>(
        radius.1 as i32,
        radius.0 as i32,
        |p, r| draw_sweep_line(b, style, center, (1, 0), p, r)
    ));
    check_result!(draw_part_c::<B, _>(
        radius.1 as i32,
        radius.0 as i32,
        |p, r| draw_sweep_line(b, style, center, (-1, 0), p, r)
    ));

    let d_inner = ((radius.1 as f64) / (2f64).sqrt()) as i32;
    let d_outer = (((radius.0 as f64) / (2f64).sqrt()) as i32).min(radius.1 as i32 - 1);
    let d_outer_actually = (radius.1 as i32).min(
        (radius.0 as f64 * radius.0 as f64 - radius.1 as f64 * radius.1 as f64 / 2.0)
            .sqrt()
            .ceil() as i32,
    );

    check_result!(b.draw_line(
        (center.0 - d_inner, center.1 - d_inner),
        (center.0 - d_outer, center.1 - d_outer),
        &style.color()
    ));
    check_result!(b.draw_line(
        (center.0 + d_inner, center.1 - d_inner),
        (center.0 + d_outer, center.1 - d_outer),
        &style.color()
    ));
    check_result!(b.draw_line(
        (center.0 - d_inner, center.1 + d_inner),
        (center.0 - d_outer, center.1 + d_outer),
        &style.color()
    ));
    check_result!(b.draw_line(
        (center.0 + d_inner, center.1 + d_inner),
        (center.0 + d_outer, center.1 + d_outer),
        &style.color()
    ));

    check_result!(b.draw_line(
        (center.0 - d_inner, center.1 + d_inner),
        (center.0 - d_outer_actually, center.1 + d_inner),
        &style.color()
    ));
    check_result!(b.draw_line(
        (center.0 + d_inner, center.1 - d_inner),
        (center.0 + d_inner, center.1 - d_outer_actually),
        &style.color()
    ));
    check_result!(b.draw_line(
        (center.0 + d_inner, center.1 + d_inner),
        (center.0 + d_inner, center.1 + d_outer_actually),
        &style.color()
    ));
    check_result!(b.draw_line(
        (center.0 + d_inner, center.1 + d_inner),
        (center.0 + d_outer_actually, center.1 + d_inner),
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
        let inner_radius = radius - (style.stroke_width() / 2).min(radius);
        radius += style.stroke_width() / 2;
        if inner_radius > 0 {
            return draw_annulus(b, center, (radius, inner_radius), style);
        } else {
            fill = true;
        }
    }

    let min = (f64::from(radius) * (1.0 - (2f64).sqrt() / 2.0)).ceil() as i32;
    let max = (f64::from(radius) * (1.0 + (2f64).sqrt() / 2.0)).floor() as i32;

    let range = min..=max;

    let (up, down) = (
        range.start() + center.1 - radius as i32,
        range.end() + center.1 - radius as i32,
    );

    for dy in range {
        let dy = dy - radius as i32;
        let y = center.1 + dy;

        let lx = (f64::from(radius) * f64::from(radius)
            - (f64::from(dy) * f64::from(dy)).max(1e-5))
        .sqrt();

        let left = center.0 - lx.floor() as i32;
        let right = center.0 + lx.floor() as i32;

        let v = lx - lx.floor();

        let x = center.0 + dy;
        let top = center.1 - lx.floor() as i32;
        let bottom = center.1 + lx.floor() as i32;

        if fill {
            check_result!(b.draw_line((left, y), (right, y), &style.color()));
            check_result!(b.draw_line((x, top), (x, up - 1), &style.color()));
            check_result!(b.draw_line((x, down + 1), (x, bottom), &style.color()));
        } else {
            check_result!(b.draw_pixel((left, y), style.color().mix(1.0 - v)));
            check_result!(b.draw_pixel((right, y), style.color().mix(1.0 - v)));

            check_result!(b.draw_pixel((x, top), style.color().mix(1.0 - v)));
            check_result!(b.draw_pixel((x, bottom), style.color().mix(1.0 - v)));
        }

        check_result!(b.draw_pixel((left - 1, y), style.color().mix(v)));
        check_result!(b.draw_pixel((right + 1, y), style.color().mix(v)));
        check_result!(b.draw_pixel((x, top - 1), style.color().mix(v)));
        check_result!(b.draw_pixel((x, bottom + 1), style.color().mix(v)));
    }

    Ok(())
}
