use crate::math_errors::MathError;

pub(crate) fn float_to_i32_checked(v: f64) -> Result<i32, MathError> {
    if !v.is_finite() {
        return Err(MathError::NonFiniteCalculation);
    }

    if v < f64::from(i32::MIN) || v > f64::from(i32::MAX) {
        return Err(MathError::ValueOutOfRange);
    }

    Ok(v as i32)
}

pub(crate) fn ceil_f64_to_i32(v: f64) -> Result<i32, MathError> {
    float_to_i32_checked(v.ceil())
}

pub(crate) fn floor_f64_to_i32(v: f64) -> Result<i32, MathError> {
    float_to_i32_checked(v.floor())
}

pub(crate) fn f64_to_f32_checked(v: f64) -> Result<f32, MathError> {
    if !v.is_finite() {
        return Err(MathError::NonFiniteCalculation);
    }

    if v < f64::from(f32::MIN) || v > f64::from(f32::MAX) {
        return Err(MathError::ValueOutOfRange);
    }

    let out = v as f32;

    if !out.is_finite() {
        return Err(MathError::NonFiniteCalculation);
    }

    Ok(out)
}

pub(crate) fn non_zero_i32(v: i32) -> Result<i32, MathError> {
    if v == 0 {
        Err(MathError::ZeroDivision)
    } else {
        Ok(v)
    }
}

pub(crate) fn non_zero_u32(v: u32) -> Result<u32, MathError> {
    if v == 0 {
        Err(MathError::ZeroDivision)
    } else {
        Ok(v)
    }
}

pub(crate) fn non_zero_f64(v: f64) -> Result<f64, MathError> {
    if !v.is_finite() {
        return Err(MathError::NonFiniteCalculation);
    }

    if v == 0.0 {
        Err(MathError::ZeroDivision)
    } else {
        Ok(v)
    }
}

pub(crate) fn checked_add_i32(lhs: i32, rhs: i32) -> Result<i32, MathError> {
    lhs.checked_add(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_sub_i32(lhs: i32, rhs: i32) -> Result<i32, MathError> {
    lhs.checked_sub(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_mul_i32(lhs: i32, rhs: i32) -> Result<i32, MathError> {
    lhs.checked_mul(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_div_i32(lhs: i32, rhs: i32) -> Result<i32, MathError> {
    lhs.checked_div(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_neg_i32(v: i32) -> Result<i32, MathError> {
    v.checked_neg().ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_add_u32(lhs: u32, rhs: u32) -> Result<u32, MathError> {
    lhs.checked_add(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_sub_u32(lhs: u32, rhs: u32) -> Result<u32, MathError> {
    lhs.checked_sub(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_mul_u32(lhs: u32, rhs: u32) -> Result<u32, MathError> {
    lhs.checked_mul(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_div_u32(lhs: u32, rhs: u32) -> Result<u32, MathError> {
    lhs.checked_div(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn u32_to_i32_checked(v: u32) -> Result<i32, MathError> {
    i32::try_from(v).map_err(|_| MathError::ValueOutOfRange)
}

pub(crate) fn sqrt_f64_checked(v: f64) -> Result<f64, MathError> {
    if !v.is_finite() || v < 0.0 {
        return Err(MathError::NonFiniteCalculation);
    }

    Ok(v.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_to_i32_checked_accepts_valid_value() {
        assert_eq!(float_to_i32_checked(42.0), Ok(42));
    }

    #[test]
    fn float_to_i32_checked_rejects_non_finite_values() {
        assert_eq!(
            float_to_i32_checked(f64::NAN),
            Err(MathError::NonFiniteCalculation)
        );
        assert_eq!(
            float_to_i32_checked(f64::INFINITY),
            Err(MathError::NonFiniteCalculation)
        );
        assert_eq!(
            float_to_i32_checked(f64::NEG_INFINITY),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn float_to_i32_checked_rejects_out_of_range_values() {
        assert_eq!(
            float_to_i32_checked(f64::from(i32::MAX) + 1.0),
            Err(MathError::ValueOutOfRange)
        );

        assert_eq!(
            float_to_i32_checked(f64::from(i32::MIN) - 1.0),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn ceil_f64_to_i32_rounds_up_before_conversion() {
        assert_eq!(ceil_f64_to_i32(1.2), Ok(2));
    }

    #[test]
    fn floor_f64_to_i32_rounds_down_before_conversion() {
        assert_eq!(floor_f64_to_i32(1.8), Ok(1));
    }

    #[test]
    fn f64_to_f32_checked_accepts_finite_value() {
        assert_eq!(f64_to_f32_checked(1.5), Ok(1.5_f32));
    }

    #[test]
    fn f64_to_f32_checked_rejects_non_finite_values() {
        assert_eq!(
            f64_to_f32_checked(f64::NAN),
            Err(MathError::NonFiniteCalculation)
        );
        assert_eq!(
            f64_to_f32_checked(f64::INFINITY),
            Err(MathError::NonFiniteCalculation)
        );
        assert_eq!(
            f64_to_f32_checked(f64::NEG_INFINITY),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn f64_to_f32_checked_rejects_out_of_range_values() {
        assert_eq!(
            f64_to_f32_checked(f64::from(f32::MAX) * 2.0),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn non_zero_i32_accepts_non_zero_value() {
        assert_eq!(non_zero_i32(7), Ok(7));
    }

    #[test]
    fn non_zero_i32_rejects_zero() {
        assert_eq!(non_zero_i32(0), Err(MathError::ZeroDivision));
    }

    #[test]
    fn non_zero_u32_accepts_non_zero_value() {
        assert_eq!(non_zero_u32(7), Ok(7));
    }

    #[test]
    fn non_zero_u32_rejects_zero() {
        assert_eq!(non_zero_u32(0), Err(MathError::ZeroDivision));
    }

    #[test]
    fn non_zero_f64_accepts_non_zero_value() {
        assert_eq!(non_zero_f64(7.0), Ok(7.0));
    }

    #[test]
    fn non_zero_f64_rejects_zero() {
        assert_eq!(non_zero_f64(0.0), Err(MathError::ZeroDivision));
    }

    #[test]
    fn non_zero_f64_rejects_non_finite_value() {
        assert_eq!(
            non_zero_f64(f64::NAN),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn checked_add_i32_accepts_valid_sum() {
        assert_eq!(checked_add_i32(2, 3), Ok(5));
    }

    #[test]
    fn checked_add_i32_rejects_overflow() {
        assert_eq!(
            checked_add_i32(i32::MAX, 1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_sub_i32_accepts_valid_difference() {
        assert_eq!(checked_sub_i32(5, 3), Ok(2));
    }

    #[test]
    fn checked_sub_i32_rejects_overflow() {
        assert_eq!(
            checked_sub_i32(i32::MIN, 1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_mul_i32_accepts_valid_product() {
        assert_eq!(checked_mul_i32(6, 7), Ok(42));
    }

    #[test]
    fn checked_mul_i32_rejects_overflow() {
        assert_eq!(
            checked_mul_i32(i32::MAX, 2),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_div_i32_accepts_valid_quotient() {
        assert_eq!(checked_div_i32(8, 2), Ok(4));
    }

    #[test]
    fn checked_div_i32_rejects_division_by_zero() {
        assert_eq!(
            checked_div_i32(8, 0),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_div_i32_rejects_min_divided_by_negative_one() {
        assert_eq!(
            checked_div_i32(i32::MIN, -1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_neg_i32_accepts_valid_negation() {
        assert_eq!(checked_neg_i32(7), Ok(-7));
    }

    #[test]
    fn checked_neg_i32_rejects_min_value() {
        assert_eq!(
            checked_neg_i32(i32::MIN),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_add_u32_accepts_valid_sum() {
        assert_eq!(checked_add_u32(2, 3), Ok(5));
    }

    #[test]
    fn checked_add_u32_rejects_overflow() {
        assert_eq!(
            checked_add_u32(u32::MAX, 1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_sub_u32_accepts_valid_difference() {
        assert_eq!(checked_sub_u32(5, 3), Ok(2));
    }

    #[test]
    fn checked_sub_u32_rejects_underflow() {
        assert_eq!(
            checked_sub_u32(0, 1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_mul_u32_accepts_valid_product() {
        assert_eq!(checked_mul_u32(6, 7), Ok(42));
    }

    #[test]
    fn checked_mul_u32_rejects_overflow() {
        assert_eq!(
            checked_mul_u32(u32::MAX, 2),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_div_u32_accepts_valid_quotient() {
        assert_eq!(checked_div_u32(8, 2), Ok(4));
    }

    #[test]
    fn checked_div_u32_rejects_division_by_zero() {
        assert_eq!(
            checked_div_u32(8, 0),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn u32_to_i32_checked_accepts_in_range_value() {
        assert_eq!(u32_to_i32_checked(42), Ok(42));
    }

    #[test]
    fn u32_to_i32_checked_rejects_out_of_range_value() {
        assert_eq!(
            u32_to_i32_checked(i32::MAX as u32 + 1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn sqrt_f64_checked_accepts_valid_value() {
        assert_eq!(sqrt_f64_checked(9.0), Ok(3.0));
    }

    #[test]
    fn sqrt_f64_checked_rejects_negative_value() {
        assert_eq!(
            sqrt_f64_checked(-1.0),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn sqrt_f64_checked_rejects_non_finite_value() {
        assert_eq!(
            sqrt_f64_checked(f64::NAN),
            Err(MathError::NonFiniteCalculation)
        );
    }
}