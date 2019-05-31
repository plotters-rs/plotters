# Plotters - Rust Drawing Library for Visualization  🦀 📈🚀 

<a href="https://crates.io/crates/plotters">
    <img style="display: inline!important" src="https://img.shields.io/crates/v/plotters.svg"></img>
</a>

<a href="https://docs.rs/plotters">
    <img style="display: inline!important" src="https://docs.rs/plotters/badge.svg"></img>
</a>

<a href="https://plumberserver.com/plotters-docs/plotters">
	<img style="display: inline! important" src="https://img.shields.io/badge/docs-development-lightgrey.svg"></img>
</a>

*Please note: This library is in a very early stage. I am trying my best to stabilize the APIs and improving the overall quality. APIs may change at this time.*

Plotters is drawing library designed for rendering figures, plots, and charts, in pure rust. Plotters supports various types of backends, including bitmap, vector graph and WebAssembly. 

## Gallery

$$gallery$$

## Quick Start

To use Plotters, you can simply add Plotters into your `Cargo.toml`
```toml
[depedencies]
plotters = "^0.1.12"
```

And the following code draws a quadratic function. `src/main.rs`,

```rust
$$examples/quick_start.rs$$
```

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/0.png)

To use the latest development version, put `plotters = {git = "https://github.com/38/plotters.git" }` in `Cargo.toml`'s depedencies section.


### Motivation of Plotting in Rust

Rust is a perfect language for data visualization. Although there are many mature visualization libraries in many different languages.
But Rust is one of the best languages fits the need.

* **Easy to use** Rust has a very good iterator system built into the standard library. With the help of iterators,
Plotting in Rust can be as easy as most of the high-level programming languages. The Rust based plotting library
can be very easy to use.

* **Fast** If you need rendering a figure with trillions of data points, 
Rust is a good choice. Rust's performance allows you to combine data processing step 
and rendering step into a single application. When plotting in high-level programming languages,
e.g. Javascript or Python, data points must be downsampled before feeding into the plotting 
program because of the performance considerations. Rust is fast enough to do the data processing and visualization 
within a single program. You can also integrate the 
figure rendering code into your application handling a huge amount of data and visualize it in real-time.

* **WebAssembly Support** Rust is one of few the language with the best WASM support. Plotting in Rust could be 
very useful for visualization on a web page and would have a huge performance improvement comparing to Javascript.

### What type of figure is supported?

We are not limited to any specific type of figure at all. You can create your own types of figures easily with the Plotters API.
But Plotters provides some builtin figure types for convenience.
Currently, we support line series, point series, candlestick series, and histogram.
And the library is designed to be able to render multiple figure into a single image.
But Plotter is aimed to be a platform that is fully extendable to support any other types of figure.

### Plotting on HTML5 canvas

Plotters currently supports backend that uses the HTML5 canvas. To use the WASM support, you can simply create
`CanvasBackend` instead of other backend and all other API remains the same!

There's a small demo for Plotters + WASM under `examples/wasm-demo` directory of this repo. 
And you should be able to try the deployed version with the following [link](https://plumberserver.com/plotters-wasm-demo/index.html).


## Concepts by examples

### Drawing Backends
Plotters can use different drawing backends, including SVG, BitMap, and even real-time rendering. For example, a bitmap drawing backend.

```rust
$$examples/drawing_backends.rs$$
```

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/1.png)

### Drawing Area
Plotters uses a concept called drawing area for layout purpose.
Plotters support multiple integrating into a single image.
This is done by creating sub-drawing-areas.

Besides that, the drawing area also allows the customized coordinate system, by doing so, the coordinate mapping is done by the drawing area automatically.

```rust
$$examples/drawing_area.rs$$
```

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/2.png)

### Elements

In Plotters, elements are build blocks of figures. All elements are able to draw on a drawing area.
There are different types of built-in elements, like lines, texts, circles, etc.
You can also define your own element in the application code.

You may also combine existing elements to build a complex element.

```rust
$$examples/elements.rs$$
```

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/3.png)

### Composable Elements

Besides the built-in elements, elements can be composed into a logic group we called composed elements.
When composing new elements, the upper-left corner is given in the target coordinate, and a new pixel-based 
coordinate which has the upper-left corner defined as `(0,0)` is used for further element composition purpose.

For example, we can have an element which includes a dot and its coordinate.

```rust
$$examples/composable_elements.rs$$
```

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/4.png)

### Chart Context

In order to draw a chart, Plotters need a data object built on top of the drawing area called `ChartContext`.
The chart context defines even higher level constructs compare to the drawing area.
For example, you can define the label areas, meshes, and put a data series onto the drawing area with the help
of the chart context object.

```rust
$$examples/chart.rs$$
```

![](https://raw.githubusercontent.com/38/plotters/master/examples/outputs/5.png)

$$style$$
