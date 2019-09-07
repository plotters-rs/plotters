# Changelog

## Plotters latest (?)

### Bug Fix

- Key points algorithm for numeric coordinate might not respect the constraint

## Plotters 0.2.4 (2019-09-05)

### Improvement

- Add `i128` and `u128` as coordinate type (Credit to @Geemili)

### Bug Fix

- Date coordinate is working for a long time span now

## Plotters 0.2.3 (2019-08-19)

### Improvement

- Color system now is based on `palette` crate (Credit to @Veykril)

## Plotters 0.2.2 (2019-06-25)

### Added

- More documentation: a Jupyter interactive notebook of Plotters tutorial 
- Add more quadrants to the `SeriesLabelPosition` (Credit to @wolfjagger).

### Improvement

- Histogram imporvements, horizental bar is supported, new creation API which compiler can infer the type
- Supporting split the drawing area with a list of breakpoints using `DrawingArea::split_by_breakpoints`
- Enable SVG support for WASM
- Make the `BitMapBackend` takes an in memory mutable buffer

### Fix

- Rectangle drawing bug when the axis is reversed

## Plotters 0.2.1 (2019-06-10)

### Improvement

- Move the sample images and other documentaiton data out of this repository.

### Fix
- Make drawing errors shareable across threads. Otherwise, it causes compile error in some cases. (Thanks to @rkarp)

## Plotters 0.2.0 (2019-06-08)

### Added
- Add relative sizing by added function `DrawingArea::relative_to_height` and `DrawingArea::relative_to_width`.
- Added piston backend, now we can render plot on a window and dynamically render the plot

### Improved
- Creating drawing area with `&Rc<RefCell<DrawingBackend>>`. Previously, the drawing area creation requires take over the drawing backend's ownership. But sometimes the drawing backend may have additonal options. With new API, this can be done by putting the backend drawing area into smart pointers, thus, the drawing backend is accessible after creates the root drawing area.

## Plotters 0.1.14 (2019-06-06)

### Added
- Font is now support rotation transformation. Use `FontDesc::transform` to apply an rotation to transformation. For example, `font.transform(FontTransform::Rotate90)`.
- ChartContext now support drawing axis description. Use `MeshStyle::x_desc` and `MeshStyle::y_desc` to specify the axis description text.
- Add series label support. `ChartContext::draw_series` now returns a struct `SeriesAnno` that collects the additional information for series labeling. `ChartContext::draw_series_labels` are used to actually draw the series label. (See `examples/chart.rs` for detailed examples)
- Mocking drawing backend.
- evcxr Support

### Improvement
- Unify `OwnedText` and `Text` into `Text`. Previously, `OwnedText` and `Text` are two seperate types, one holds a `String` another holds a `&str`. Now `OwnedText` is removed.
use `Text::new("text".to_string(),...)` for owned text element and `Text::new("text", ...)` for borrowed text.
- Refactor the color representation code, since previously it's heavily relies on the trait object and hard to use
- More test cases

## Plotters 0.1.13 (2019-05-31)

### Added
- New abstraction of backend style with `BackendStyle` trait which should be able to extend easier in the future
- Backend support features, now feature options can be used to control which backend should be supported
- Add new trait `IntoDrawingArea`, now we can use `backend.into_drawing_area()` to convert the backend into a raw drawing area
- Now elements support dynamic dispatch, use `element.into_dyn()` to convert the element into a runtime dispatching element

### Improvement
- Improved the overall code quality
- Documentation polish
- Stabilized APIs
- New conversion traits impls
- Now transparent color is ignored by SVG, bitmap and HTML Canvas backend

### Fix
- Changed the oepn-close pattern to a `present` function which indicates the end of drawing one frame
- Fix the but that `ChartBuilder::title` and `ChartBuilder::margin` cannot be called at the same time && `build_ranged` now returning a result.
