use crate::MathError;
use std::error::Error;
/// The error produced by a drawing backend.
#[derive(Debug)]
pub enum DrawingErrorKind<E: Error + Send + Sync> {
    /// A drawing backend error
    DrawingError(E),
    /// A font rendering error
    FontError(Box<dyn Error + Send + Sync + 'static>),
    /// A mathematical operation has failed
    MathError(MathError),
}

impl<E: Error + Send + Sync> From<MathError> for DrawingErrorKind<E> {
    fn from(err: MathError) -> Self {
        DrawingErrorKind::MathError(err)
    }
}

impl<E: Error + Send + Sync> std::fmt::Display for DrawingErrorKind<E> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            DrawingErrorKind::DrawingError(e) => write!(fmt, "Drawing backend error: {}", e),
            DrawingErrorKind::FontError(e) => write!(fmt, "Font loading error: {}", e),
            DrawingErrorKind::MathError(e) => write!(fmt, "Math error: {}", e),
        }
    }
}

impl<E: Error + Send + Sync> Error for DrawingErrorKind<E> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[derive(Debug)]
    struct TestBackendError;

    impl fmt::Display for TestBackendError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "backend exploded")
        }
    }

    impl std::error::Error for TestBackendError {}

    #[derive(Debug)]
    struct TestFontError;

    impl fmt::Display for TestFontError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "font exploded")
        }
    }

    impl std::error::Error for TestFontError {}

    #[test]
    fn from_math_error_creates_math_error_variant() {
        let err: DrawingErrorKind<TestBackendError> = MathError::ValueOutOfRange.into();

        assert!(matches!(
            err,
            DrawingErrorKind::MathError(MathError::ValueOutOfRange)
        ));
    }

    #[test]
    fn display_formats_drawing_backend_error() {
        let err: DrawingErrorKind<TestBackendError> =
            DrawingErrorKind::DrawingError(TestBackendError);

        assert_eq!(err.to_string(), "Drawing backend error: backend exploded");
    }

    #[test]
    fn display_formats_font_error() {
        let err: DrawingErrorKind<TestBackendError> =
            DrawingErrorKind::FontError(Box::new(TestFontError));

        assert_eq!(err.to_string(), "Font loading error: font exploded");
    }

    #[test]
    fn display_formats_math_error() {
        let math_error = MathError::ValueOutOfRange;
        let err: DrawingErrorKind<TestBackendError> = DrawingErrorKind::MathError(math_error);

        assert_eq!(err.to_string(), format!("Math error: {}", math_error));
    }

    #[test]
    fn drawing_error_kind_implements_error() {
        fn assert_error<E: std::error::Error + Send + Sync>() {}

        assert_error::<DrawingErrorKind<TestBackendError>>();
    }

    #[test]
    fn drawing_error_variant_can_be_matched() {
        let err: DrawingErrorKind<TestBackendError> =
            DrawingErrorKind::DrawingError(TestBackendError);

        match err {
            DrawingErrorKind::DrawingError(e) => {
                assert_eq!(e.to_string(), "backend exploded");
            }
            _ => panic!("expected DrawingError variant"),
        }
    }

    #[test]
    fn font_error_variant_can_be_matched() {
        let err: DrawingErrorKind<TestBackendError> =
            DrawingErrorKind::FontError(Box::new(TestFontError));

        match err {
            DrawingErrorKind::FontError(e) => {
                assert_eq!(e.to_string(), "font exploded");
            }
            _ => panic!("expected FontError variant"),
        }
    }

    #[test]
    fn math_error_variant_can_be_matched() {
        let err: DrawingErrorKind<TestBackendError> =
            DrawingErrorKind::MathError(MathError::ZeroDivision);

        match err {
            DrawingErrorKind::MathError(MathError::ZeroDivision) => {}
            _ => panic!("expected MathError::ZeroDivision variant"),
        }
    }
}
