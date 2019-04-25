pub mod chart;
pub mod data;
pub mod drawing;
pub mod element;
pub mod series;
pub mod style;

pub mod prelude {
    pub use crate::chart::{ChartBuilder, ChartContext};
    pub use crate::drawing::coord::{RangedCoordf32, RangedCoordf64, RangedCoordu32, RangedCoordu64, RangedCoordi32, RangedCoordi64, Ranged, RangedCoord, CoordTranslate};
    pub use crate::drawing::{backend::DrawingBackend, DrawingArea};
    pub use crate::style::{Color, RGBColor, Plattle99, Plattle9999, Plattle100, Plattle, FontDesc, TextStyle, ShapeStyle, Mixable};
    pub use crate::series::{LineSeries, PointSeries};

    pub use crate::drawing::BitMapBackend;

    pub use crate::element::{Path, Cross, Circle, Text, Rectangle, EmptyElement, OwnedText};
}

