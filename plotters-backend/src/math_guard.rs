use crate::math_errors::MathError;
use std::convert::TryFrom;
pub(crate) fn f64_to_i32_checked(v: f64) -> Result<i32, MathError> {
    if !v.is_finite() {
        return Err(MathError::NonFiniteCalculation);
    }

    if v < f64::from(i32::MIN) || v > f64::from(i32::MAX) {
        return Err(MathError::ValueOutOfRange);
    }

    Ok(v as i32)
}

pub(crate) fn ceil_f64_to_i32(v: f64) -> Result<i32, MathError> {
    f64_to_i32_checked(v.ceil())
}

pub(crate) fn floor_f64_to_i32(v: f64) -> Result<i32, MathError> {
    f64_to_i32_checked(v.floor())
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

pub(crate) fn checked_add_i64(lhs: i64, rhs: i64) -> Result<i64, MathError> {
    lhs.checked_add(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_sub_i64(lhs: i64, rhs: i64) -> Result<i64, MathError> {
    lhs.checked_sub(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_mul_i64(lhs: i64, rhs: i64) -> Result<i64, MathError> {
    lhs.checked_mul(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_div_i64(lhs: i64, rhs: i64) -> Result<i64, MathError> {
    lhs.checked_div(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_neg_i64(v: i64) -> Result<i64, MathError> {
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

pub(crate) fn u32_to_usize_checked(v: u32) -> Result<usize, MathError> {
    if u64::from(v) > usize::MAX as u64 {
        return Err(MathError::ValueOutOfRange);
    }

    Ok(v as usize)
}

pub(crate) fn i32_to_u32_checked(v: i32) -> Result<u32, MathError> {
    u32::try_from(v).map_err(|_| MathError::ValueOutOfRange)
}
pub(crate) fn sqrt_f64_checked(v: f64) -> Result<f64, MathError> {
    if !v.is_finite() || v < 0.0 {
        return Err(MathError::NonFiniteCalculation);
    }

    Ok(v.sqrt())
}

pub(crate) fn checked_div_f64(lhs: f64, rhs: f64) -> Result<f64, MathError> {
    if !lhs.is_finite() || !rhs.is_finite() {
        return Err(MathError::NonFiniteCalculation);
    }

    if rhs == 0.0 {
        return Err(MathError::ZeroDivision);
    }

    let out = lhs / rhs;

    if !out.is_finite() {
        return Err(MathError::NonFiniteCalculation);
    }

    Ok(out)
}

pub(crate) fn checked_add_usize(lhs: usize, rhs: usize) -> Result<usize, MathError> {
    lhs.checked_add(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_sub_usize(lhs: usize, rhs: usize) -> Result<usize, MathError> {
    lhs.checked_sub(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_mul_usize(lhs: usize, rhs: usize) -> Result<usize, MathError> {
    lhs.checked_mul(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn checked_div_usize(lhs: usize, rhs: usize) -> Result<usize, MathError> {
    lhs.checked_div(rhs).ok_or(MathError::ValueOutOfRange)
}

pub(crate) fn i32_to_usize_checked(v: i32) -> Result<usize, MathError> {
    if v < 0 {
        return Err(MathError::ValueOutOfRange);
    }

    Ok(v as usize)
}

pub(crate) fn i64_to_usize_checked(v: i64) -> Result<usize, MathError> {
    if v < 0 {
        return Err(MathError::ValueOutOfRange);
    }

    if v as u64 > usize::MAX as u64 {
        return Err(MathError::ValueOutOfRange);
    }

    Ok(v as usize)
}

pub(crate) fn usize_to_i32_checked(v: usize) -> Result<i32, MathError> {
    if v > i32::MAX as usize {
        return Err(MathError::ValueOutOfRange);
    }

    Ok(v as i32)
}

pub(crate) fn usize_to_u32_checked(v: usize) -> Result<u32, MathError> {
    if v > u32::MAX as usize {
        return Err(MathError::ValueOutOfRange);
    }

    Ok(v as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_to_i32_checked_accepts_valid_value() {
        assert_eq!(f64_to_i32_checked(42.0), Ok(42));
    }

    #[test]
    fn float_to_i32_checked_rejects_non_finite_values() {
        assert_eq!(
            f64_to_i32_checked(f64::NAN),
            Err(MathError::NonFiniteCalculation)
        );
        assert_eq!(
            f64_to_i32_checked(f64::INFINITY),
            Err(MathError::NonFiniteCalculation)
        );
        assert_eq!(
            f64_to_i32_checked(f64::NEG_INFINITY),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn float_to_i32_checked_rejects_out_of_range_values() {
        assert_eq!(
            f64_to_i32_checked(f64::from(i32::MAX) + 1.0),
            Err(MathError::ValueOutOfRange)
        );

        assert_eq!(
            f64_to_i32_checked(f64::from(i32::MIN) - 1.0),
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
        assert_eq!(non_zero_f64(f64::NAN), Err(MathError::NonFiniteCalculation));
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
        assert_eq!(checked_div_i32(8, 0), Err(MathError::ValueOutOfRange));
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
        assert_eq!(checked_neg_i32(i32::MIN), Err(MathError::ValueOutOfRange));
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
        assert_eq!(checked_sub_u32(0, 1), Err(MathError::ValueOutOfRange));
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
        assert_eq!(checked_div_u32(8, 0), Err(MathError::ValueOutOfRange));
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
    fn i32_to_u32_checked_accepts_in_range_value() {
        assert_eq!(i32_to_u32_checked(42), Ok(42));
    }

    #[test]
    fn i32_to_u32_checked_rejects_out_of_range_value() {
        assert_eq!(
            i32_to_u32_checked(i32::MIN),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn i32_to_u32_checked_rejects_negative_one() {
        assert_eq!(i32_to_u32_checked(-1), Err(MathError::ValueOutOfRange));
    }

    #[test]
    fn i32_to_u32_checked_accepts_i32_max() {
        assert_eq!(i32_to_u32_checked(i32::MAX), Ok(i32::MAX as u32));
    }

    #[test]
    fn sqrt_f64_checked_accepts_valid_value() {
        assert_eq!(sqrt_f64_checked(9.0), Ok(3.0));
    }

    #[test]
    fn sqrt_f64_checked_rejects_negative_value() {
        assert_eq!(sqrt_f64_checked(-1.0), Err(MathError::NonFiniteCalculation));
    }

    #[test]
    fn sqrt_f64_checked_rejects_non_finite_value() {
        assert_eq!(
            sqrt_f64_checked(f64::NAN),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn checked_add_i64_accepts_valid_sum() {
        assert_eq!(checked_add_i64(2, 3), Ok(5));
    }

    #[test]
    fn checked_add_i64_rejects_overflow() {
        assert_eq!(
            checked_add_i64(i64::MAX, 1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_add_i64_rejects_underflow() {
        assert_eq!(
            checked_add_i64(i64::MIN, -1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_sub_i64_accepts_valid_difference() {
        assert_eq!(checked_sub_i64(5, 3), Ok(2));
    }

    #[test]
    fn checked_sub_i64_rejects_overflow() {
        assert_eq!(
            checked_sub_i64(i64::MAX, -1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_sub_i64_rejects_underflow() {
        assert_eq!(
            checked_sub_i64(i64::MIN, 1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_mul_i64_accepts_valid_product() {
        assert_eq!(checked_mul_i64(6, 7), Ok(42));
    }

    #[test]
    fn checked_mul_i64_rejects_overflow() {
        assert_eq!(
            checked_mul_i64(i64::MAX, 2),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_mul_i64_rejects_underflow() {
        assert_eq!(
            checked_mul_i64(i64::MIN, 2),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_div_i64_accepts_valid_quotient() {
        assert_eq!(checked_div_i64(8, 2), Ok(4));
    }

    #[test]
    fn checked_div_i64_rejects_division_by_zero() {
        assert_eq!(checked_div_i64(8, 0), Err(MathError::ValueOutOfRange));
    }

    #[test]
    fn checked_div_i64_rejects_min_divided_by_negative_one() {
        assert_eq!(
            checked_div_i64(i64::MIN, -1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_neg_i64_accepts_valid_negation() {
        assert_eq!(checked_neg_i64(7), Ok(-7));
    }

    #[test]
    fn checked_neg_i64_accepts_zero() {
        assert_eq!(checked_neg_i64(0), Ok(0));
    }

    #[test]
    fn checked_neg_i64_rejects_min_value() {
        assert_eq!(checked_neg_i64(i64::MIN), Err(MathError::ValueOutOfRange));
    }

    #[test]
    fn checked_neg_i64_accepts_max_value() {
        assert_eq!(checked_neg_i64(i64::MAX), Ok(-i64::MAX));
    }

    #[test]
    fn checked_div_f64_accepts_valid_quotient() {
        assert_eq!(checked_div_f64(8.0, 2.0), Ok(4.0));
    }

    #[test]
    fn checked_div_f64_accepts_fractional_quotient() {
        assert_eq!(checked_div_f64(1.0, 4.0), Ok(0.25));
    }

    #[test]
    fn checked_div_f64_accepts_negative_quotient() {
        assert_eq!(checked_div_f64(-8.0, 2.0), Ok(-4.0));
    }

    #[test]
    fn checked_div_f64_rejects_division_by_positive_zero() {
        assert_eq!(checked_div_f64(8.0, 0.0), Err(MathError::ZeroDivision));
    }

    #[test]
    fn checked_div_f64_rejects_division_by_negative_zero() {
        assert_eq!(checked_div_f64(8.0, -0.0), Err(MathError::ZeroDivision));
    }

    #[test]
    fn checked_div_f64_rejects_nan_lhs() {
        assert_eq!(
            checked_div_f64(f64::NAN, 2.0),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn checked_div_f64_rejects_nan_rhs() {
        assert_eq!(
            checked_div_f64(8.0, f64::NAN),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn checked_div_f64_rejects_infinite_lhs() {
        assert_eq!(
            checked_div_f64(f64::INFINITY, 2.0),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn checked_div_f64_rejects_infinite_rhs() {
        assert_eq!(
            checked_div_f64(8.0, f64::INFINITY),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn checked_div_f64_rejects_non_finite_output() {
        assert_eq!(
            checked_div_f64(f64::MAX, f64::MIN_POSITIVE),
            Err(MathError::NonFiniteCalculation)
        );
    }

    #[test]
    fn checked_add_usize_accepts_valid_sum() {
        assert_eq!(checked_add_usize(2, 3), Ok(5));
    }

    #[test]
    fn checked_add_usize_rejects_overflow() {
        assert_eq!(
            checked_add_usize(usize::MAX, 1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_sub_usize_accepts_valid_difference() {
        assert_eq!(checked_sub_usize(5, 3), Ok(2));
    }

    #[test]
    fn checked_sub_usize_rejects_underflow() {
        assert_eq!(checked_sub_usize(0, 1), Err(MathError::ValueOutOfRange));
    }

    #[test]
    fn checked_mul_usize_accepts_valid_product() {
        assert_eq!(checked_mul_usize(6, 7), Ok(42));
    }

    #[test]
    fn checked_mul_usize_rejects_overflow() {
        assert_eq!(
            checked_mul_usize(usize::MAX, 2),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn checked_div_usize_accepts_valid_quotient() {
        assert_eq!(checked_div_usize(8, 2), Ok(4));
    }

    #[test]
    fn checked_div_usize_rejects_division_by_zero() {
        assert_eq!(checked_div_usize(8, 0), Err(MathError::ValueOutOfRange));
    }

    #[test]
    fn i32_to_usize_checked_accepts_non_negative_value() {
        assert_eq!(i32_to_usize_checked(42), Ok(42));
    }

    #[test]
    fn i32_to_usize_checked_accepts_zero() {
        assert_eq!(i32_to_usize_checked(0), Ok(0));
    }

    #[test]
    fn i32_to_usize_checked_rejects_negative_value() {
        assert_eq!(i32_to_usize_checked(-1), Err(MathError::ValueOutOfRange));
    }

    #[test]
    fn i64_to_usize_checked_accepts_non_negative_value() {
        assert_eq!(i64_to_usize_checked(42), Ok(42));
    }

    #[test]
    fn i64_to_usize_checked_rejects_negative_value() {
        assert_eq!(i64_to_usize_checked(-1), Err(MathError::ValueOutOfRange));
    }

    #[test]
    fn i64_to_usize_checked_rejects_out_of_range_value() {
        if usize::BITS < 64 {
            assert_eq!(
                i64_to_usize_checked(i64::MAX),
                Err(MathError::ValueOutOfRange)
            );
        }
    }

    #[test]
    fn usize_to_i32_checked_accepts_in_range_value() {
        assert_eq!(usize_to_i32_checked(42), Ok(42));
    }

    #[test]
    fn usize_to_i32_checked_accepts_i32_max() {
        assert_eq!(usize_to_i32_checked(i32::MAX as usize), Ok(i32::MAX));
    }

    #[test]
    fn usize_to_i32_checked_rejects_out_of_range_value() {
        assert_eq!(
            usize_to_i32_checked(i32::MAX as usize + 1),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn usize_to_u32_checked_accepts_in_range_value() {
        assert_eq!(usize_to_u32_checked(42), Ok(42));
    }

    #[test]
    fn usize_to_u32_checked_accepts_u32_max() {
        assert_eq!(usize_to_u32_checked(u32::MAX as usize), Ok(u32::MAX));
    }

    #[test]
    fn usize_to_u32_checked_rejects_out_of_range_value() {
        if usize::BITS > 32 {
            assert_eq!(
                usize_to_u32_checked(u32::MAX as usize + 1),
                Err(MathError::ValueOutOfRange)
            );
        }
    }
    #[test]
    fn u32_to_usize_checked_accepts_zero() {
        assert_eq!(u32_to_usize_checked(0), Ok(0));
    }

    #[test]
    fn u32_to_usize_checked_accepts_in_range_value() {
        assert_eq!(u32_to_usize_checked(42), Ok(42));
    }

    #[test]
    fn u32_to_usize_checked_accepts_u32_max_on_supported_platforms() {
        if usize::BITS >= 32 {
            assert_eq!(u32_to_usize_checked(u32::MAX), Ok(u32::MAX as usize));
        }
    }
}
