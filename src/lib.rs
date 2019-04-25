pub mod chart;
pub mod data;
pub mod drawing;
pub mod element;
pub mod series;
pub mod style;

pub mod prelude {
    pub use crate::chart::{ChartBuilder, ChartContext};
    pub use crate::drawing::coord::{
        CoordTranslate, Ranged, RangedCoord, RangedCoordf32, RangedCoordf64, RangedCoordi32,
        RangedCoordi64, RangedCoordu32, RangedCoordu64,
    };
    pub use crate::drawing::{backend::DrawingBackend, DrawingArea};
    pub use crate::series::{LineSeries, PointSeries};
    pub use crate::style::{
        Color, FontDesc, Mixable, Plattle, Plattle100, Plattle99, Plattle9999, RGBColor,
        ShapeStyle, TextStyle,
    };

    pub use crate::drawing::BitMapBackend;

    pub use crate::element::{Circle, Cross, EmptyElement, OwnedText, Path, Rectangle, Text};
}
