use std::ops::Range;

use plotters_backend::{BackendCoord, DrawingBackend};

use crate::chart::{ChartContext, DualCoordChartContext, PolarMeshStyle};
use crate::coord::{
    cartesian::Cartesian2d,
    polar::Polar2d,
    ranged1d::{AsRangedCoord, Ranged, ValueFormatter},
    Shift,
};
use crate::drawing::DrawingArea;

mod draw_impl;

impl<'a, DB, RT, TT, R, T> ChartContext<'a, DB, Polar2d<R, T>>
where
    DB: DrawingBackend,
    R: Ranged<ValueType = RT> + ValueFormatter<RT>,
    T: Ranged<ValueType = TT> + ValueFormatter<TT>,
{
    pub(crate) fn is_overlapping_drawing_area(
        &self,
        area: Option<&DrawingArea<DB, Shift>>,
    ) -> bool {
        if let Some(area) = area {
            let (x0, y0) = area.get_base_pixel();
            let (w, h) = area.dim_in_pixel();
            let (x1, y1) = (x0 + w as i32, y0 + h as i32);
            let (dx0, dy0) = self.drawing_area.get_base_pixel();
            let (w, h) = self.drawing_area.dim_in_pixel();
            let (dx1, dy1) = (dx0 + w as i32, dy0 + h as i32);

            let (ox0, ox1) = (x0.max(dx0), x1.min(dx1));
            let (oy0, oy1) = (y0.max(dy0), y1.min(dy1));

            ox1 > ox0 && oy1 > oy0
        } else {
            false
        }
    }
    
    pub fn configure_mesh(&mut self) -> PolarMeshStyle<'a, '_, R, T, DB> {
        PolarMeshStyle::new(self)
    }
}
