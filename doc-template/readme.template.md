# Plotters - A drawing library for Plotting

<a href="https://crates.io/crates/plotters">
    <img style="display: inline!important" src="https://img.shields.io/crates/v/plotters.svg"></img>
</a>

<a href="https://docs.rs/plotters">
    <img style="display: inline!important" src="https://docs.rs/plotters/badge.svg"></img>
</a>

*Please note: This library is in a very early stage. I am trying my best to stabilize the APIs, but APIs may change during the time.*

Plotters is drawing library designed for rendering figures, plots and charts, in pure rust. 

### Reasons for Plotting in Rust

* **Rust is fast.** If you need rendering a figure with trillions of data points, 
Rust is a good choice. Rust's performance allows you combine data processing step 
and rendering step into a single application. When plotting in high-level programming languages,
e.g. Javascript or Python, data points must be downsampled before feeding into the plotting 
program because of the performance considerations. Rust is fast enough to do the data processing and visualization 
within a signle program. You can also integrate the 
figure rendering code into your application handling huge amount of data and visualize it in real-time.

* **Iterators** Rust has a very good iterator system built into the standard library. With the help of iterators,
Plotting in Rust can be as easy as most of the high-level programming languages. The Rust based plotting library
can be very easy to use.

* **WebAssembly Support** Rust is one of few the language with the best WASM support. Plotting in Rust could be 
very useful for visualization on a web page and would have a huge performance improvement comparing to Javascript.

### What type of figure is supported?

Currently, we support line series, point series, candlestick series and histogram.
And the library is designed to be able to render multiple figure into a single image.
But Plotter is aimed to be a platform that is fully extendable to supporting any other types of figure.

### Plotting on HTML5 canvas

Plotters currently supports backend that uses the HTML5 canvas. To use the WASM support, you can simply create
`CanvasBackend` instead of other backend and all other API remains the same!

There's a small demo for Plotters + WASM under `examples/wasm-demo` directory of this repo. 
And you should be able to try the deployed version with the following [link](https://plumberserver.com/plotters-wasm-demo/index.html).

## Gallery

$$gallery$$

## Quick Start

```rust
use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = BitMapBackend::new("examples/outputs/0.png", (640, 480));
    let root: DrawingArea<_, _> = backend.into();
    let font = Into::<FontDesc>::into("Arial").resize(50.0);
    root.fill(&White)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("y=x^2", &font)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(-1f32..1f32, -0.1f32..1f32);

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
        &RGBColor(255, 0, 0),
    ))?;

    root.present()?;
    Ok(())
}
```

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/0.png)

## Concepts by examples

### Drawing Backends
Plotters can use different drawing backends, such as SVG, BitMap, etc, even real-time rendering. For example a bitmap drawing backend.

```rust
use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a 800*600 bitmap and start drawing
    let mut backend = BitMapBackend::new("examples/outputs/1.png", (300,200));
    // And if we want SVG backend
    // let backend = SVGBackend::new("output.svg", (800, 600));
    backend.draw_rect((50,50), (200, 150), &Red, true)?;
    backend.present()?;
    Ok(())
}
```

And this will produce

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/1.png)

### Drawing Area
Plotters use a concept called drawing area for layout purpose.
Plotters support multiple plot integrate in a single image.
This is done by creating sub drawing areas.

Besides that, drawing area also allows customized coordinate system, by doing so, the coordinate mapping is done by the drawing area automatically.

```rust
use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = BitMapBackend::new("examples/outputs/2.png", (300, 200));
    // A backend object can be converted into a drawing area
    let root_drawing_area:DrawingArea<_,_> = backend.into();
    // And we can split the drawing area into 3x3 grid
    let child_drawing_areas = root_drawing_area.split_evenly((3,3));
    // Then we fill the drawing area with different color
    for (area,color) in child_drawing_areas.into_iter().zip(0..) {
        area.fill(&Palette99::pick(color))?;
    }
    root_drawing_area.present()?;
    Ok(())
}
```

And this will produce

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/2.png)

### Elements

In Plotters, elements are build blocks of a image. All elements are able to draw on a drawing area.
There are different types of elements, such as, lines, texts, circles, etc.

You may also combining existing elements to build a complex element.

```rust
use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = BitMapBackend::new("examples/outputs/3.png", (300, 200));
    // A backend object can be converted into a drawing area
    let root:DrawingArea<_,_> = backend.into();
    root.fill(&White);
    // Draw an circle on the drawing area
    root.draw(&Circle::new((100,100), 50, Into::<ShapeStyle>::into(&Green).filled()))?;
    root.present()?;
    Ok(())
}
```

And this will produce

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/3.png)

### Composable Elements

Besides the basic elements, elements can be composed into a logic group we called composed elements.
When composing new elements, the upper-left conner is given in the target coordinate, and a new pixel
based coordinate which has the upper-left conner defined as `(0,0)` is used for further element composition purpose.

For example, we can have an element which includes a dot and its coordinate.

```rust
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = BitMapBackend::new("examples/outputs/4.png", (640, 480));
    // A backend object can be converted into a drawing area
    let root:DrawingArea<_,_> = backend.into();
    root.fill(&RGBColor(240,200,200))?;

    let root = root.apply_coord_spec(RangedCoord::<RangedCoordf32, RangedCoordf32>::new(0f32..1f32, 0f32..1f32, (0..640, 0..480)));
    let font = Into::<FontDesc>::into("Arial").resize(15.0);

    let dot_and_label = |x:f32,y:f32| {
        return EmptyElement::at((x,y))
               + Circle::new((0,0), 3, ShapeStyle::from(&Black).filled())
               + OwnedText::new(format!("({:.2},{:.2})", x, y), (10, 0), &font);
    };

    root.draw(&dot_and_label(0.5, 0.6))?;
    root.draw(&dot_and_label(0.25, 0.33))?;
    root.draw(&dot_and_label(0.8, 0.8))?;
    root.present()?;
    Ok(())
}
```

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/4.png)

### Chart Context

In order to draw a chart, Plotters need an data object build on top of drawing area called `ChartContext`.
The chart context defines even higher level constructs compare to the drawing area.
For example, you can define the label areas, meshs, and put a data series onto the drawing area with the help
of the chart context object.

```rust
use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = BitMapBackend::new("examples/outputs/5.png", (640, 480));
    let root:DrawingArea<_,_> = backend.into();
    root.fill(&White);
    let root = root.margin(10,10,10,10);
    // After this point, we should be able to draw construct a chart context
    let font:FontDesc = Into::<FontDesc>::into("Arial").resize(40.0);
    // Create the chart object
    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption("This is our first plot", &font)
        // Set the size of the label region
        .x_label_area_size(20)
        .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_ranged(0f32..10f32, 0f32..10f32);

    // Then we can draw a mesh
    chart.configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(5)
        .y_labels(5)
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw()?;

    // And we can draw something in the drawing area
    let smaller_font = font.resize(10.0);
    chart.draw_series(LineSeries::new(vec![(0.0,0.0), (5.0, 5.0), (8.0, 7.0)], &Red))?;
    // Similarly, we can draw point series
    chart.draw_series(PointSeries::of_element(vec![(0.0,0.0), (5.0, 5.0), (8.0, 7.0)], 5, &Red, &|c,s,st| {
        return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
            + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
            + OwnedText::new(format!("{:?}", c), (10, 0), &smaller_font);
    }))?;
    root.present()?;
    Ok(())
}
```

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/5.png)

$$style$$
