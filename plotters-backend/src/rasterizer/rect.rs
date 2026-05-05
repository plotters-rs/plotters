use crate::{
    math_guard::checked_sub_i32, BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind,
};

pub fn draw_rect<B: DrawingBackend, S: BackendStyle>(
    b: &mut B,
    upper_left: BackendCoord,
    bottom_right: BackendCoord,
    style: &S,
    fill: bool,
) -> Result<(), DrawingErrorKind<B::ErrorType>> {
    if style.color().alpha == 0.0 {
        return Ok(());
    }

    let x0 = upper_left.0.min(bottom_right.0);
    let y0 = upper_left.1.min(bottom_right.1);
    let x1 = upper_left.0.max(bottom_right.0);
    let y1 = upper_left.1.max(bottom_right.1);

    let width = checked_sub_i32(x1, x0)?;
    let height = checked_sub_i32(y1, y0)?;

    if fill {
        if width < height {
            for x in x0..=x1 {
                check_result!(b.draw_line((x, y0), (x, y1), style));
            }
        } else {
            for y in y0..=y1 {
                check_result!(b.draw_line((x0, y), (x1, y), style));
            }
        }
    } else {
        b.draw_line((x0, y0), (x0, y1), style)?;
        b.draw_line((x0, y0), (x1, y0), style)?;
        b.draw_line((x1, y1), (x0, y1), style)?;
        b.draw_line((x1, y1), (x1, y0), style)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BackendColor, BackendStyle, MathError};

    #[derive(Debug)]
    struct TestBackendError;

    impl std::fmt::Display for TestBackendError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test backend error")
        }
    }

    impl std::error::Error for TestBackendError {}

    #[derive(Clone, Copy)]
    struct TestStyle {
        color: BackendColor,
    }

    impl BackendStyle for TestStyle {
        fn color(&self) -> BackendColor {
            self.color
        }

        fn stroke_width(&self) -> u32 {
            1
        }
    }

    #[derive(Default)]
    struct TestBackend {
        lines: Vec<(BackendCoord, BackendCoord)>,
    }

    impl DrawingBackend for TestBackend {
        type ErrorType = TestBackendError;

        fn get_size(&self) -> (u32, u32) {
            (100, 100)
        }

        fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            Ok(())
        }

        fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            Ok(())
        }

        fn draw_pixel(
            &mut self,
            _point: BackendCoord,
            _color: BackendColor,
        ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            Ok(())
        }

        fn draw_line<S: BackendStyle>(
            &mut self,
            from: BackendCoord,
            to: BackendCoord,
            _style: &S,
        ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            self.lines.push((from, to));
            Ok(())
        }
    }

    fn visible_style() -> TestStyle {
        TestStyle {
            color: BackendColor {
                rgb: (0, 0, 0),
                alpha: 1.0,
            },
        }
    }

    fn transparent_style() -> TestStyle {
        TestStyle {
            color: BackendColor {
                rgb: (0, 0, 0),
                alpha: 0.0,
            },
        }
    }

    #[test]
    fn transparent_rect_draws_nothing() {
        let mut backend = TestBackend::default();

        draw_rect(&mut backend, (0, 0), (10, 10), &transparent_style(), false).unwrap();

        assert!(backend.lines.is_empty());
    }

    #[test]
    fn unfilled_rect_draws_four_edges() {
        let mut backend = TestBackend::default();

        draw_rect(&mut backend, (1, 2), (4, 5), &visible_style(), false).unwrap();

        assert_eq!(
            backend.lines,
            vec![
                ((1, 2), (1, 5)),
                ((1, 2), (4, 2)),
                ((4, 5), (1, 5)),
                ((4, 5), (4, 2)),
            ]
        );
    }

    #[test]
    fn unfilled_rect_normalizes_reversed_coordinates() {
        let mut backend = TestBackend::default();

        draw_rect(&mut backend, (4, 5), (1, 2), &visible_style(), false).unwrap();

        assert_eq!(
            backend.lines,
            vec![
                ((1, 2), (1, 5)),
                ((1, 2), (4, 2)),
                ((4, 5), (1, 5)),
                ((4, 5), (4, 2)),
            ]
        );
    }

    #[test]
    fn filled_wide_rect_draws_horizontal_lines() {
        let mut backend = TestBackend::default();

        draw_rect(&mut backend, (1, 1), (4, 2), &visible_style(), true).unwrap();

        assert_eq!(backend.lines, vec![((1, 1), (4, 1)), ((1, 2), (4, 2)),]);
    }

    #[test]
    fn filled_tall_rect_draws_vertical_lines() {
        let mut backend = TestBackend::default();

        draw_rect(&mut backend, (1, 1), (2, 4), &visible_style(), true).unwrap();

        assert_eq!(backend.lines, vec![((1, 1), (1, 4)), ((2, 1), (2, 4)),]);
    }

    #[test]
    fn rect_with_extreme_coordinates_returns_out_of_range_math_error() {
        let mut backend = TestBackend::default();

        let err = draw_rect(
            &mut backend,
            (i32::MIN, 0),
            (i32::MAX, 1),
            &visible_style(),
            true,
        )
        .unwrap_err();

        assert!(matches!(
            err,
            DrawingErrorKind::MathError(MathError::ValueOutOfRange)
        ));

        assert!(backend.lines.is_empty());
    }
}
