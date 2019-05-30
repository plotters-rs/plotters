/*!
  Defines the drawing elements, the high-level drawing unit in Plotters drawing system

  ## Introduction
  An element is the drawing unit for Plotter's high-level drawing API.
  Different from low-level drawing API, an element is a logic unit of component in the image.
  There are few built-in elements, including `Circle`, `Pixel`, `Rectangle`, `Path`, `Text`, etc.

  All element can be drawn onto the drawing area using API `DrawingArea::draw(...)`.
  Plotters use "iterator of elements" as the abstraction of any type of plot.

  ## Implementing your
  You can also define your own element, `CandleStick` is a good sample of implementing complex
  element. There are two trait required for an element:

  - `PointCollection` - the struct should be able to return an iterator of key-points under guest coordinate
  - `Drawable` - the struct should be able to use performe drawing on a drawing backend with pixel-based coordinate

  An example of element that draws a red "X" in a red rectangle onto the backend:

  ```rust
    use std::iter::{Once, once};
    use plotters::element::{PointCollection, Drawable};
    use plotters::drawing::backend::{BackendCoord, DrawingErrorKind};
    use plotters::prelude::*;

    // Any example drawing a red X
    struct RedBoxedX((i32, i32));

    // For any reference to RedX, we can convert it into an iterator of points
    impl <'a> PointCollection<'a, (i32, i32)> for &'a RedBoxedX {
        type Borrow = &'a (i32, i32);
        type IntoIter = Once<&'a (i32, i32)>;
        fn point_iter(self) -> Self::IntoIter {
            once(&self.0)
        }
    }

    // How to actually draw this element
    impl <DB:DrawingBackend> Drawable<DB> for RedBoxedX {
        fn draw<I:Iterator<Item = BackendCoord>>(
            &self,
            mut pos: I,
            backend: &mut DB
        ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
            let pos = pos.next().unwrap();
            backend.draw_rect(pos, (pos.0 + 10, pos.1 + 12), &Red, false)?;
            backend.draw_text("X", &("Arial", 20).into(), pos, &Red)
        }
    }

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new("examples/outputs/element-0.png", (640, 480)).into_drawing_area();
        root.draw(&RedBoxedX((200, 200)))?;
        Ok(())
    }
  ```
  ![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/element-0.png)

  ## Composable Elements
  You also have an convenient way to build an element that isn't built into the Plotters library by
  combining existing elements into a logic group. To build an composable elemnet, you need to use an
  logic empty element that draws nothing to the backend but denotes the relative zero point of the logical
  group. Any element defined with pixel based offset coordinate can be added into the group later using
  the `+` operator.

  For example, the red boxed X element can be implemented with Composable element in the following way:
  ```rust
    use plotters::prelude::*;
    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new("examples/outputs/element-1.png", (640, 480)).into_drawing_area();
        let font:FontDesc = ("Arial", 20).into();
        let style = TextStyle{ font: &font, color: &Red };
        root.draw(&(EmptyElement::at((200, 200)) + OwnedText::new("X", (0, 0), &style) + Rectangle::new([(0,0), (10, 12)], &Red)))?;
        Ok(())
    }
  ```
  ![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/element-1.png)
*/
use std::borrow::Borrow;
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

mod basic_shapes;
pub use basic_shapes::*;

mod points;
pub use points::*;

mod composable;
pub use composable::{ComposedElement, EmptyElement};

mod candlestick;
pub use candlestick::CandleStick;

/// A type which is logically a collection of points, under any given coordinate system
pub trait PointCollection<'a, Coord> {
    /// The item in point iterator
    type Borrow: Borrow<Coord>;

    /// The point iterator
    type IntoIter: IntoIterator<Item = Self::Borrow>;

    /// framework to do the coordinate mapping
    fn point_iter(self) -> Self::IntoIter;
}

/// The trait indicates we are able to draw it on a drawing area
pub trait Drawable<DB:DrawingBackend> {
    /// Actually draws the element. The key points is already translated into the
    /// image cooridnate and can be used by DC directly
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        pos: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>>;
}

