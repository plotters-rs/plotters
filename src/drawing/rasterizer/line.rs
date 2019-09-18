use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingErrorKind};
use crate::drawing::DrawingBackend;

use crate::style::Color;

pub(crate) fn draw_line<DB: DrawingBackend, S: BackendStyle>(
    back: &mut DB,
    mut from: BackendCoord,
    mut to: BackendCoord,
    style: &S,
) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
    if style.as_color().alpha() == 0.0 {
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
