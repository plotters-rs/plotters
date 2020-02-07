use crate::element::{DynElement, IntoDynElement, PathElement, Polygon};
use crate::style::colors::TRANSPARENT;
use crate::style::ShapeStyle;
use plotters_backend::DrawingBackend;

/// An area series is similar to a line series but use a filled polygon
pub struct AreaSeries<DB: DrawingBackend, X: Clone, Y: Clone> {
    area_style: ShapeStyle,
    border_style: ShapeStyle,
    baseline: Y,
    data: Vec<(X, Y)>,
    state: u32,
    _p: std::marker::PhantomData<DB>,
}

impl<DB: DrawingBackend, X: Clone, Y: Clone> AreaSeries<DB, X, Y> {
    pub fn new<S: Into<ShapeStyle>, I: IntoIterator<Item = (X, Y)>>(
        iter: I,
        baseline: Y,
        area_style: S,
    ) -> Self {
        Self {
            area_style: area_style.into(),
            baseline,
            data: iter.into_iter().collect(),
            state: 0,
            border_style: (&TRANSPARENT).into(),
            _p: std::marker::PhantomData,
        }
    }

    pub fn border_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.border_style = style.into();
        self
    }
}

impl<DB: DrawingBackend, X: Clone + 'static, Y: Clone + 'static> Iterator for AreaSeries<DB, X, Y> {
    type Item = DynElement<'static, DB, (X, Y)>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.state == 0 {
            let mut data: Vec<_> = self.data.clone();

            if !data.is_empty() {
                data.push((data[data.len() - 1].0.clone(), self.baseline.clone()));
                data.push((data[0].0.clone(), self.baseline.clone()));
            }

            self.state = 1;

            Some(Polygon::new(data, self.area_style.clone()).into_dyn())
        } else if self.state == 1 {
            let data: Vec<_> = self.data.clone();

            self.state = 2;

            Some(PathElement::new(data, self.border_style.clone()).into_dyn())
        } else {
            None
        }
    }
}
