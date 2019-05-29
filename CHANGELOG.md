# Changelog

## Plotters 0.1.13

- Improved the overall code quality
- Documentation polish
- Changed the oepn-close pattern to a `present` function which indicates the end of drawing one frame
- Fix the but that `ChartBuilder::title` and `ChartBuilder::margin` cannot be called at the same time && `build_ranged` now returning a result.
- New abstraction of backend style with `BackendStyle` trait which should be able to extend easier in the future
