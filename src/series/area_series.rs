use crate::element::Polygon;
use crate::style::ShapeStyle;

/// An area series is similar to a line series but use a filled polygon
pub struct AreaSeries<X: Clone, Y: Clone, I: IntoIterator<Item = (X, Y)>> {
    area_style: ShapeStyle,
    baseline: Y,
    data_iter: Option<I::IntoIter>,
}

impl<X: Clone, Y: Clone, I: IntoIterator<Item = (X, Y)>> AreaSeries<X, Y, I> {
    pub fn new<S: Into<ShapeStyle>>(iter: I, baseline: Y, area_style: S) -> Self {
        Self {
            area_style: area_style.into(),
            baseline,
            data_iter: Some(iter.into_iter()),
        }
    }
}

impl<X: Clone, Y: Clone, I: IntoIterator<Item = (X, Y)>> Iterator for AreaSeries<X, Y, I> {
    type Item = Polygon<(X, Y)>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.data_iter.is_some() {
            let mut data_iter = None;
            std::mem::swap(&mut self.data_iter, &mut data_iter);
            let mut data: Vec<_> = data_iter.unwrap().collect();

            if !data.is_empty() {
                data.push((data[data.len() - 1].0.clone(), self.baseline.clone()));
                data.push((data[0].0.clone(), self.baseline.clone()));
            }

            Some(Polygon::new(data, self.area_style.clone()))
        } else {
            None
        }
    }
}
