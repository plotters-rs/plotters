use crate::{DrawingBackend, BackendCoord, BackendStyle, DrawingErrorKind};

pub fn draw_circle<B: DrawingBackend, S: BackendStyle>(
    b: &mut B,
    center: BackendCoord,
    radius: u32,
    style: &S,
    fill: bool,
) -> Result<(), DrawingErrorKind<B::ErrorType>> {
    if style.color().alpha == 0.0 {
        return Ok(());
    }

    if !fill && style.stroke_width() != 1 {
        // FIXME: We are currently ignore the stroke width for circles
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
            check_result!(b.draw_line((x, top), (x, up), &style.color()));
            check_result!(b.draw_line((x, down), (x, bottom), &style.color()));
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
