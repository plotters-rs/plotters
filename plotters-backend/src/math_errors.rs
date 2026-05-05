use core::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MathError {
    ValueOverflow,
    ValueUnderflow,
    NonFiniteCalculation,
    ValueOutOfRange,
    ZeroDivision,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathError::ValueOverflow => {
                write!(f, "value exceeds the target type's maximum")
            }
            MathError::ValueUnderflow => {
                write!(f, "value is below the target type's minimum")
            }
            MathError::NonFiniteCalculation => {
                write!(f, "calculation produced a non-finite value")
            }
            MathError::ValueOutOfRange => {
                write!(f, "value is out of range for the target type")
            }
            MathError::ZeroDivision => {
                write!(f, "attempted to divide by zero")
            }
        }
    }
}

impl std::error::Error for MathError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_error_is_copy_clone_debug_partial_eq_and_eq() {
        let err = MathError::ValueOutOfRange;
        let copied = err;
        let cloned = err;

        assert_eq!(err, copied);
        assert_eq!(err, cloned);
        assert_eq!(format!("{:?}", err), "ValueOutOfRange");
    }

    #[test]
    fn display_formats_value_overflow() {
        assert_eq!(
            MathError::ValueOverflow.to_string(),
            "value exceeds the target type's maximum"
        );
    }

    #[test]
    fn display_formats_value_underflow() {
        assert_eq!(
            MathError::ValueUnderflow.to_string(),
            "value is below the target type's minimum"
        );
    }

    #[test]
    fn display_formats_non_finite_calculation() {
        assert_eq!(
            MathError::NonFiniteCalculation.to_string(),
            "calculation produced a non-finite value"
        );
    }

    #[test]
    fn display_formats_value_out_of_range() {
        assert_eq!(
            MathError::ValueOutOfRange.to_string(),
            "value is out of range for the target type"
        );
    }

    #[test]
    fn display_formats_zero_division() {
        assert_eq!(
            MathError::ZeroDivision.to_string(),
            "attempted to divide by zero"
        );
    }

    #[test]
    fn math_error_implements_std_error() {
        fn assert_error<E: std::error::Error>() {}

        assert_error::<MathError>();
    }
}
