use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingErrorKind};
use crate::drawing::DrawingBackend;

use crate::style::Color;

pub fn draw_line<DB: DrawingBackend, S: BackendStyle>(
    back: &mut DB,
    mut from: BackendCoord,
    mut to: BackendCoord,
    style: &S,
) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
    if style.as_color().alpha() == 0.0 {
        return Ok(());
    }

    if style.stroke_width() != 1 {
        // If the line is wider than 1px, then we need to make it a polygon
        let v = (i64::from(to.0 - from.0), i64::from(to.1 - from.1));
        let l = ((v.0 * v.0 + v.1 * v.1) as f64).sqrt();

        if l < 1e-5 {
            return Ok(());
        }

        let v = (v.0 as f64 / l, v.1 as f64 / l);

        let r = f64::from(style.stroke_width()) / 2.0;
        let mut trans = [(v.1 * r, -v.0 * r), (-v.1 * r, v.0 * r)];
        let mut vertices = vec![];

        for point in [from, to].iter() {
            for t in trans.iter() {
                vertices.push((
                    (f64::from(point.0) + t.0) as i32,
                    (f64::from(point.1) + t.1) as i32,
                ))
            }

            trans.swap(0, 1);
        }

        return back.fill_polygon(vertices, &style.as_color());
    }

    if from.0 == to.0 {
        if from.1 > to.1 {
            std::mem::swap(&mut from, &mut to);
        }
        for y in from.1..=to.1 {
            back.draw_pixel((from.0, y), &style.as_color())?;
        }
        return Ok(());
    }

    if from.1 == to.1 {
        if from.0 > to.0 {
            std::mem::swap(&mut from, &mut to);
        }
        for x in from.0..=to.0 {
            back.draw_pixel((x, from.1), &style.as_color())?;
        }
        return Ok(());
    }

    let steep = (from.0 - to.0).abs() < (from.1 - to.1).abs();

    if steep {
        from = (from.1, from.0);
        to = (to.1, to.0);
    }

    let (from, to) = if from.0 > to.0 {
        (to, from)
    } else {
        (from, to)
    };

    let grad = f64::from(to.1 - from.1) / f64::from(to.0 - from.0);

    let mut put_pixel = |(x, y): BackendCoord, b: f64| {
        if steep {
            back.draw_pixel((y, x), &style.as_color().mix(b))
        } else {
            back.draw_pixel((x, y), &style.as_color().mix(b))
        }
    };

    let mut y = f64::from(from.1);

    for x in from.0..=to.0 {
        put_pixel((x, y as i32), 1.0 + y.floor() - y)?;
        put_pixel((x, y as i32 + 1), y - y.floor())?;

        y += grad;
    }

    Ok(())
}
