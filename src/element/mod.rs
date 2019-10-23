/*!
    Defines the drawing elements, the high-level drawing unit in Plotters drawing system

    ## Introduction
    An element is the drawing unit for Plotter's high-level drawing API.
    Different from low-level drawing API, an element is a logic unit of component in the image.
    There are few built-in elements, including `Circle`, `Pixel`, `Rectangle`, `Path`, `Text`, etc.

    All element can be drawn onto the drawing area using API `DrawingArea::draw(...)`.
    Plotters use "iterator of elements" as the abstraction of any type of plot.

    ## Implementing your own element
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
            backend: &mut DB,
            _: (u32, u32),
        ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
            let pos = pos.next().unwrap();
            backend.draw_rect(pos, (pos.0 + 10, pos.1 + 12), &RED.to_rgba(), false)?;
            backend.draw_text("X", &("serif", 20).into(), pos, &RED.to_rgba())
        }
    }

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(
            "plotters-doc-data/element-0.png",
            (640, 480)
        ).into_drawing_area();
        root.draw(&RedBoxedX((200, 200)))?;
        Ok(())
    }
    ```
      ![](https://plotters-rs.github.io/plotters-doc-data/element-0.png)

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
        let root = BitMapBackend::new(
            "plotters-doc-data/element-1.png",
            (640, 480)
        ).into_drawing_area();
        let font:FontDesc = ("serif", 20).into();
        root.draw(&(EmptyElement::at((200, 200))
                + Text::new("X", (0, 0), &"serif".into_font().resize(20.0).color(&RED))
                + Rectangle::new([(0,0), (10, 12)], &RED)
        ))?;
        Ok(())
    }
    ```
    ![](https://plotters-rs.github.io/plotters-doc-data/element-1.png)

    ## Dynamic Elements
    By default, Plotters uses static dispatch for all the elements and series. For example,
    the `ChartContext::draw_series` method accepts an iterator of `T` where type `T` implements
    all the traits a element should implement. Although, we can use the series of composable element
    for complex series drawing. But sometimes, we still want to make the series heterogyous, which means
    the iterator should be able to holds elements in different type.
    For example, a point series with corss and circle. This requires the dynamically dispatched elements.
    In plotters, all the elements can be converted into `DynElement`, the dynamic dispatch container for
    all elements (include exernal implemented ones).
    Plotters automatically implements `IntoDynElement` for all elements, by doing so, any dynamic element should have
    `into_dyn` function which would wrap the element into a dynmanic element wrapper.

    For example, the following code counts the number of factors of integer and mark all prime numbers in cross.
    ```rust
    use plotters::prelude::*;
    fn num_of_factor(n: i32) -> i32 {
        let mut ret = 2;
        for i in 2..n {
            if i * i > n {
                break;
            }

            if n % i == 0 {
                if i * i != n {
                    ret += 2;
                } else {
                    ret += 1;
                }
            }
        }
        return ret;
    }
    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let root =
            BitMapBackend::new("plotters-doc-data/element-3.png", (640, 480))
            .into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .margin(5)
            .build_ranged(0..50, 0..10)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        chart.draw_series((0..50).map(|x| {
            let center = (x, num_of_factor(x));
            // Although the arms of if statement has different types,
            // but they can be placed into a dynamic element wrapper,
            // by doing so, the type is unified.
            if center.1 == 2 {
                Cross::new(center, 4, Into::<ShapeStyle>::into(&RED).filled()).into_dyn()
            } else {
                Circle::new(center, 4, Into::<ShapeStyle>::into(&GREEN).filled()).into_dyn()
            }
        }))?;

        Ok(())
    }
    ```
    ![](https://plotters-rs.github.io/plotters-doc-data/element-3.png)
*/
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use std::borrow::Borrow;

mod basic_shapes;
pub use basic_shapes::*;

mod text;
pub use text::*;

mod points;
pub use points::*;

mod composable;
pub use composable::{ComposedElement, EmptyElement};

mod candlestick;
pub use candlestick::CandleStick;

mod errorbar;
pub use errorbar::{ErrorBar, ErrorBarOrientH, ErrorBarOrientV};

#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
mod image;
#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
pub use self::image::BitMapElement;

mod dynelem;
pub use dynelem::{DynElement, IntoDynElement};

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
pub trait Drawable<DB: DrawingBackend> {
    /// Actually draws the element. The key points is already translated into the
    /// image cooridnate and can be used by DC directly
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        pos: I,
        backend: &mut DB,
        parent_dim: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>>;
}
