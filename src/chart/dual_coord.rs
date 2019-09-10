use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use super::context::{ChartContext, SeriesAnno};
use super::mesh::SecondaryMeshStyle;

use crate::coord::{CoordTranslate, Ranged, RangedCoord};
use crate::drawing::backend::DrawingBackend;
use crate::drawing::DrawingAreaErrorKind;
use crate::element::{Drawable, PointCollection};

/// The chart context that has two coordinate system attached
pub struct DualCoordChartContext<'a, DB: DrawingBackend, CT1: CoordTranslate, CT2: CoordTranslate> {
    pub(super) primiary: ChartContext<'a, DB, CT1>,
    pub(super) secondary: ChartContext<'a, DB, CT2>,
}

impl<'a, DB: DrawingBackend, CT1: CoordTranslate, CT2: CoordTranslate>
    DualCoordChartContext<'a, DB, CT1, CT2>
{
    pub(super) fn new(mut primiary: ChartContext<'a, DB, CT1>, secondary_coord: CT2) -> Self {
        let secondary_drawing_area = primiary
            .drawing_area
            .strip_coord_spec()
            .apply_coord_spec(secondary_coord);
        let mut secondary_x_label_area = [None, None];
        let mut secondary_y_label_area = [None, None];

        std::mem::swap(
            &mut primiary.x_label_area[0],
            &mut secondary_x_label_area[0],
        );
        std::mem::swap(
            &mut primiary.y_label_area[1],
            &mut secondary_y_label_area[1],
        );

        Self {
            primiary,
            secondary: ChartContext {
                x_label_area: secondary_x_label_area,
                y_label_area: secondary_y_label_area,
                drawing_area: secondary_drawing_area,
                series_anno: vec![],
            },
        }
    }
}

impl<'a, DB: DrawingBackend, CT1: CoordTranslate, SX: Ranged, SY: Ranged>
    DualCoordChartContext<'a, DB, CT1, RangedCoord<SX, SY>>
where
    SX::ValueType: Debug,
    SY::ValueType: Debug,
{
    pub fn configure_secondary_axes<'b>(&'b mut self) -> SecondaryMeshStyle<'a, 'b, SX, SY, DB> {
        SecondaryMeshStyle::new(&mut self.secondary)
    }
}

impl<'a, DB: DrawingBackend, X: Ranged, Y: Ranged, SX: Ranged, SY: Ranged>
    DualCoordChartContext<'a, DB, RangedCoord<X, Y>, RangedCoord<SX, SY>>
where
    X::ValueType: Debug,
    Y::ValueType: Debug,
    SX::ValueType: Debug,
    SY::ValueType: Debug,
{
    pub fn draw_secondary_series<E, R, S>(
        &mut self,
        series: S,
    ) -> Result<&mut SeriesAnno<'a, DB>, DrawingAreaErrorKind<DB::ErrorType>>
    where
        for<'b> &'b E: PointCollection<'b, (SX::ValueType, SY::ValueType)>,
        E: Drawable<DB>,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        self.secondary.draw_series_impl(series)?;
        Ok(self.primiary.alloc_series_anno())
    }
}

impl<'a, DB: DrawingBackend, CT1: CoordTranslate, CT2: CoordTranslate>
    Borrow<ChartContext<'a, DB, CT1>> for DualCoordChartContext<'a, DB, CT1, CT2>
{
    fn borrow(&self) -> &ChartContext<'a, DB, CT1> {
        &self.primiary
    }
}

impl<'a, DB: DrawingBackend, CT1: CoordTranslate, CT2: CoordTranslate>
    BorrowMut<ChartContext<'a, DB, CT1>> for DualCoordChartContext<'a, DB, CT1, CT2>
{
    fn borrow_mut(&mut self) -> &mut ChartContext<'a, DB, CT1> {
        &mut self.primiary
    }
}

impl<'a, DB: DrawingBackend, CT1: CoordTranslate, CT2: CoordTranslate> Deref
    for DualCoordChartContext<'a, DB, CT1, CT2>
{
    type Target = ChartContext<'a, DB, CT1>;
    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl<'a, DB: DrawingBackend, CT1: CoordTranslate, CT2: CoordTranslate> DerefMut
    for DualCoordChartContext<'a, DB, CT1, CT2>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow_mut()
    }
}
