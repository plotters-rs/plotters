// The code that is related to float number handling

fn find_minimal_repr(n: f64, eps: f64) -> (f64, usize) {
    if eps >= 1.0 {
        return (n, 0);
    }
    if n - n.floor() < eps {
        (n.floor(), 0)
    } else if n.ceil() - n < eps {
        (n.ceil(), 0)
    } else {
        let (rem, pre) = find_minimal_repr((n - n.floor()) * 10.0, eps * 10.0);
        (n.floor() + rem / 10.0, pre + 1)
    }
}

fn float_to_string(n: f64, max_precision: usize) -> String {
    let (sign, n) = if n < 0.0 { ("-", -n) } else { ("", n) };
    let int_part = n.floor();

    let dec_part =
        ((n.abs() - int_part.abs()) * (10.0f64).powf(max_precision as f64)).round() as u64;

    if dec_part == 0 || max_precision == 0 {
        return format!("{}{:.0}", sign, int_part);
    }

    let mut leading = "".to_string();
    let mut dec_result = format!("{}", dec_part);

    for _ in 0..(max_precision - dec_result.len()) {
        leading.push('0');
    }

    while let Some(c) = dec_result.pop() {
        if c != '0' {
            dec_result.push(c);
            break;
        }
    }

    format!("{}{:.0}.{}{}", sign, int_part, leading, dec_result)
}

/// The function that pretty prints the floating number
/// Since rust doesn't have anything that can format a float with out appearance, so we just
/// implemnet a float pretty printing function, which finds the shortest representation of a
/// floating point number within the allowed error range.
///
/// - `n`: The float number to pretty-print
/// - `allow_sn`: Should we use scientific notation when possible
/// - **returns**: The pretty printed string
pub fn pretty_print_float(n: f64, allow_sn: bool) -> String {
    let (n, p) = find_minimal_repr(n, 1e-10);
    let d_repr = float_to_string(n, p);
    if !allow_sn {
        d_repr
    } else {
        if n == 0.0 {
            return "0".to_string();
        }

        let mut idx = n.abs().log10().floor();
        let mut exp = (10.0f64).powf(idx);

        if n.abs() / exp + 1e-5 >= 10.0 {
            idx += 1.0;
            exp *= 10.0;
        }

        if idx.abs() < 3.0 {
            return d_repr;
        }

        let (sn, sp) = find_minimal_repr(n / exp, 1e-5);
        let s_repr = format!("{}e{}", float_to_string(sn, sp), float_to_string(idx, 0));
        if s_repr.len() + 1 < d_repr.len() {
            s_repr
        } else {
            d_repr
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_pretty_printing() {
        assert_eq!(pretty_print_float(0.99999999999999999999, false), "1");
        assert_eq!(pretty_print_float(0.9999, false), "0.9999");
        assert_eq!(
            pretty_print_float(-1e-5 - 0.00000000000000001, true),
            "-1e-5"
        );
        assert_eq!(
            pretty_print_float(-1e-5 - 0.00000000000000001, false),
            "-0.00001"
        );
        assert_eq!(pretty_print_float(1e100, true), "1e100");
        assert_eq!(pretty_print_float(1234567890f64, true), "1234567890");
        assert_eq!(pretty_print_float(1000000001f64, true), "1e9");
    }
}
