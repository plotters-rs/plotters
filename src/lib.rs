/*!
# Plotters - Another Plotting Library in Rust

Plotters is a flexible drawing library for data visualization written in pure Rust. 
The library isn't aimed supporting different types of plotting, but a generic platform 
that can be extended to support different types of visualization methods.

## Concepts by examples

### Drawing Backends
Plotters can use different drawing backends, such as SVG, BitMap, etc. And even real-time rendering,
such as library. For example a bitmap drawing backend.

```rust
use plotters::prelude::*;
fn main() -> Result<(), Box<dyn Error>> {
    // Create a 800*600 bitmap and start drawing
    let backend = BitMapBackend::new("output.png", (800,600));
    // And if we want SVG backend
    // let backend = SVGBackend::new("output.svg", (800, 600));
    backend.open()?;
    backend.draw_rect((100,100), (500, 500), &RGBColor(255,0,0), true)?;
    backend.close()?;
}
```

### Drawing Area
Plotters use a concept called drawing area for layout purpose.
Plotters support multiple plot integrate in a single image. 
This is done by craeting sub drawing areas.

Besides that, drawing area also allows customized cooridnate system, by doing so, the coordinate mapping is done by the drawing area automatically.

```rust
use plotters::prelude::*;
fn main() -> Result<(), Box<dyn Error>> {
    let backend = BitMapBackend::new("output.png", (800, 600));
    // A backend object can be converted into a drawing area
    let root_drawing_area:DrawingArea<_,_> = backend.into();
    // And we can split the drawing area into 3x3 grid
    let child_drawing_areas = root_drawing_area.split_evenly((3,3));
    // Then we fill the drawing area with different color
    for (area,color) in child_drawing_areas.zip(0..) {
        area.fill(&Plattle99.pick(color))?;
    }
    backend.close();
}
```

*/
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

    pub use crate::drawing::{BitMapBackend, SVGBackend};

    pub use crate::element::{Circle, Cross, EmptyElement, OwnedText, Path, Rectangle, Text};
}
