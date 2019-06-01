# Changelog

## Plotters Development (?)

### Added
- Font is now support rotation transformation. Use `FontDesc::transform` to apply an rotation to transformation. For example, `font.transform(FontTransform::Rotate90)`.

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

## Plotters 0.1.12 (2019-05-25)

The unstable version
