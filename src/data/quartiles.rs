/// The quartiles
#[derive(Clone, Debug)]
pub struct Quartiles {
    lower_fence: f64,
    lower: f64,
    median: f64,
    upper: f64,
    upper_fence: f64,
}

impl Quartiles {
    fn calc_median<T: Into<f64> + Copy + PartialOrd>(s: &[T]) -> f64 {
        let mut s = s.to_owned();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        match s.len() % 2 {
            0 => (s[(s.len() / 2) - 1].into() / 2.0) + (s[(s.len() / 2)].into() / 2.0),
            _ => s[s.len() / 2].into(),
        }
    }

    /// Create a new quartiles struct with the values calculated from the argument.
    ///
    /// - `s`: The array of the original values
    /// - **returns** The newly created quartiles
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let quartiles = Quartiles::new(&[7, 15, 36, 39, 40, 41]);
    /// assert_eq!(quartiles.median(), 37.5);
    /// ```
    pub fn new<T: Into<f64> + Copy + PartialOrd>(s: &[T]) -> Self {
        if s.len() == 1 {
            let value = s[0].into();
            return Self {
                lower_fence: value,
                lower: value,
                median: value,
                upper: value,
                upper_fence: value,
            };
        }
        let mut s = s.to_owned();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (a, b) = if s.len() % 2 == 0 {
            s.split_at(s.len() / 2)
        } else {
            (&s[..(s.len() / 2)], &s[((s.len() / 2) + 1)..])
        };
        let lower = Quartiles::calc_median(a);
        let median = Quartiles::calc_median(&s);
        let upper = Quartiles::calc_median(b);
        let iqr = upper - lower;
        let lower_fence = lower - 1.5 * iqr;
        let upper_fence = upper + 1.5 * iqr;
        Self {
            lower_fence,
            lower,
            median,
            upper,
            upper_fence,
        }
    }

    /// Get the quartiles values.
    ///
    /// - **returns** The array [lower fence, lower quartile, median, upper quartile, upper fence]
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let quartiles = Quartiles::new(&[7, 15, 36, 39, 40, 41]);
    /// let values = quartiles.values();
    /// assert_eq!(values, [-22.5, 15.0, 37.5, 40.0, 77.5]);
    /// ```
    pub fn values(&self) -> [f32; 5] {
        [
            self.lower_fence as f32,
            self.lower as f32,
            self.median as f32,
            self.upper as f32,
            self.upper_fence as f32,
        ]
    }

    /// Get the quartiles median.
    ///
    /// - **returns** The median
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let quartiles = Quartiles::new(&[7, 15, 36, 39, 40, 41]);
    /// assert_eq!(quartiles.median(), 37.5);
    /// ```
    pub fn median(&self) -> f64 {
        self.median
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn test_empty_input() {
        let empty_array: [i32; 0] = [];
        Quartiles::new(&empty_array);
    }

    #[test]
    fn test_low_inputs() {
        assert_eq!(
            Quartiles::new(&[15.0]).values(),
            [15.0, 15.0, 15.0, 15.0, 15.0]
        );
        assert_eq!(
            Quartiles::new(&[10, 20]).values(),
            [-5.0, 10.0, 15.0, 20.0, 35.0]
        );
        assert_eq!(
            Quartiles::new(&[10, 20, 30]).values(),
            [-20.0, 10.0, 20.0, 30.0, 60.0]
        );
    }
}
