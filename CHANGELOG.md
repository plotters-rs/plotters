# Changelog

## Plotters 0.3.0 (2020-09-03)
This is the next major release of Plotters, see [release notes](./RELEASE-NOTES.md) for more detials.

### Added

- The plotters backend API crate is introduced which allows third-party backend.
- Allow slice used as a coordinate if the item type implements `PartialOrd`.
- Linspace coordinate type, which allows define a discrete coordinate on continous value types (such as f32, DateTime, etc) with a step specification
- Nested coordinate type, now we support define pair of category and nested values as nested coordinate.
- Introduce backend crates: plotters-bitmap, plotters-svg, plotters-cairo, plotters-canvas, plotters-console
- 3D Plotting features

### Fixed

- Adjust Canvas backend size with DPR (Thanks to Marius-Mueller)

### Improvement

- Enhanced key point algorithms: New key point hint trait are used for key point algorithms && many improvment on key point algorithms for different types
- Renamed `MeshStyle::line\_style\_1\ and `MeshStyle::line\_style\_2` to `bold\_line\_style` and `light\_line\_style`
- Reorganize the "plotters::coord" namespace 
- Improved discrete coordinate trait
- Split backend code into isolated crates and can be maintained indepdendenly
- Category coordinate is now replaced by slice coordinate
- Removing the requirement for `Debug` trait for chart coordinate, allows ranged coordinate define its own formatting function. 

### Removed

- The `Path` name alias for `PathElement`
- Most code `plotters::drawing::backend\_impl::\* ` (expect `MockedBackend` for testing purpose) is removed due to crate split. 
- Piston backend due to the Piston project seems not actively developing

## Plotters 0.2.15 (2020-05-26)
### Fixed

- Division by zero with logarithmic coord (issue #143)
- Update dependencies

## Plotters 0.2.14 (2020-05-05)
### Fixed

- Compile error with older rustc which causing breaks

## Plotters 0.2.13 (2020-05-04)
### Added

- Line width is supported for SVG 

### Fixed

- Updated dependencies
- Default rasterizer causing bitmap backend draw out-of-range pixels
- Depdendicy fix

## Plotters 0.2.12 (2019-12-06)
### Added

- BitMapBackend now is able to support different pixel format natively. Check our new minifb demo for details.
- Incremental Rendering by saving the previous chart context into a state and restore it on a different drawing area.
- BoxPlot support (See boxplot example for more details) (Thanks to @nuald)
- Category coordinate spec which allows use a list of given values as coordinate (Thanks to @nuald)
- New text positioning model which allows develvoper sepecify the anchor point. This is critical for layouting SVG correctly. See `plotters::style::text::text_anchor` for details. (Thanks to @nuald)

### Improved

- Faster bitmap blending algorithm, which is 5x faster than the original one.
- Text alignment improvement, now we can suggest the invariant point by giving `TextAlignment` to the text style (Thanks to @nauld)
- More controls on features, allows opt in and out series types
- Remove dependency to svg crate, since it doesn't provide more feature than a plain string.

## Plotters 0.2.11 (2019-10-27)

### Added

- Add font style support, now we are able to set font variations: normal, oblique, italic or bold.

### Improved 

- Font description is greatly improved, general font family is supported: `serif`, `serif-sans`, `monospace` (Thanks to @Tatrix)
- Tested the font loading on Linux, OSX and Windowns. Make font loading more reliable.
- `BitMapBackend` isn't depdends on `image` crate now. Only the image encoding part relies on the `image` crate
- Refactored WASM demo use ES6 and `wasm-pack` (Thanks to @Tatrix)
- Label style for X axis and Y axis can be set seperately now using `x\_label\_style` and `y\_label\_style`. (Thanks to @zhiburt)

## Plotters 0.2.10 (2019-10-23)

### Improved

- Refactored and simplified TTF font cache, use RwLock instead of mutex which may benefit for parallel rendering. (Thanks to @Tatrix)

### Bug Fix

- The fast bitmap filling algorithm may overflow the framebuffer and cause segfault

## Plotters 0.2.9 (2019-10-21)

### Improvement

- Avoid copying image buffer when manipulate the image. (Thanks to @ralfbiedert)
- Bitmap element which allows blit the image to another drawing area.
- Performance improvement: now the bitmap backend is 8 times faster
- Added benchmarks to monitor the performance change

### Bug Fix

- Performance fix: '?' operator is very slow
- Dynamic Element lifetime bound: Fix a bug that prevents area series draws on non-static lifetime backend

## Plotters 0.2.8 (2019-10-12)

### Added

- Cairo backend, which supports using Plotters draw a GTK surface.
- Allow secondary axis to be configured with different label style.
- Relative Sizing, now font and size can use relative scale: `(10).percent\_height()` means we want the size is 10% of parent's height. 
- Allow the axis overlapping with the plotting area with `ChartBuilder::set\_\label\_area\_overlap`.
- Allow label area is on the top of the drawing area by setting the label area size to negative (Thanks to @serzhiio).
- Allow configure the tick mark size, when the tick mark size is negative the axis becomes inward (Thanks to @serzhiio).

### Bug Fix

- `FontError` from rusttype isn't `Sync` and `Send`. We don't have trait bound to ensure this.  (Thanks to @dalance)

### Improvement

- New convenient functions: `disable_mesh` and `disable_axes`

## Plotters 0.2.7 (2019-10-1)

### Added

- GIF Support, now bitmap backend is able to render gif animation

### Bug Fix

- Fixed several polygon filling bugs.
- Completely DateTime coordinate system support

## Plotters 0.2.6 (2019-09-19)

### Added

- Allowing axis be placed on top or right by setting `right_y_label_area` and `top_x_label_area`
- Dual-coord system chart support: Now we are able to attach a secondary coord system to the chart using `ChartContext::set_secondary_coord`. And `draw_secondary_axes` to configure the style of secondary axes. Use `draw_secondary axis` to draw series under the secondary coordinate system.
- Add support for `y_label_offset`. Previously only X axis label supports offset attribute. 
- New label area size API `set_label_area_size` can be used for all 4 label area
- Added new error bar element
- New axis specification type `PartialAxis` which allows the partially rendered axis. For example, we can define the chart's axis range as `0..1`, but only `0.3..0.7` is rendered on axis. This can be done by `(0.0..1.0).partial_axis(0.3..0.7)`
- Drawing backend now support fill polygon and introduce polygon element
- Area Chart Support

### Improvement

- More examples are included
- Date coordinate now support using monthly or yearly axis. This is useful when plotting some data in monthly or yearly basis.
- Make margin on different side of a chart can be configured separately.
- Better test coverage

## Plotters 0.2.5 (2019-09-07)

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

- Histogram improvements, horizontal bar is supported, new creation API which compiler can infer the type
- Supporting split the drawing area with a list of breakpoints using `DrawingArea::split_by_breakpoints`
- Enable SVG support for WASM
- Make the `BitMapBackend` takes an in memory mutable buffer

### Fix

- Rectangle drawing bug when the axis is reversed

## Plotters 0.2.1 (2019-06-10)

### Improvement

- Move the sample images and other documentation data out of this repository.

### Fix
- Make drawing errors shareable across threads. Otherwise, it causes compile error in some cases. (Thanks to @rkarp)

## Plotters 0.2.0 (2019-06-08)

### Added
- Add relative sizing by added function `DrawingArea::relative_to_height` and `DrawingArea::relative_to_width`.
- Added piston backend, now we can render plot on a window and dynamically render the plot

### Improved
- Creating drawing area with `&Rc<RefCell<DrawingBackend>>`. Previously, the drawing area creation requires take over the drawing backend ownership. But sometimes the drawing backend may have additional options. With new API, this can be done by putting the backend drawing area into smart pointers, thus, the drawing backend is accessible after creates the root drawing area.

## Plotters 0.1.14 (2019-06-06)

### Added
- Font is now support rotation transformation. Use `FontDesc::transform` to apply an rotation to transformation. For example, `font.transform(FontTransform::Rotate90)`.
- ChartContext now support drawing axis description. Use `MeshStyle::x_desc` and `MeshStyle::y_desc` to specify the axis description text.
- Add series label support. `ChartContext::draw_series` now returns a struct `SeriesAnno` that collects the additional information for series labeling. `ChartContext::draw_series_labels` are used to actually draw the series label. (See `examples/chart.rs` for detailed examples)
- Mocking drawing backend.
- evcxr Support

### Improvement
- Unify `OwnedText` and `Text` into `Text`. Previously, `OwnedText` and `Text` are two separate types, one holds a `String` another holds a `&str`. Now `OwnedText` is removed.
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
- New conversion traits implementations
- Now transparent color is ignored by SVG, bitmap and HTML Canvas backend

### Fix
- Changed the open-close pattern to a `present` function which indicates the end of drawing one frame
- Fix the but that `ChartBuilder::title` and `ChartBuilder::margin` cannot be called at the same time && `build_ranged` now returning a result.
