use super::{FontData, FontFamily, FontStyle, LayoutBox};
use wasm_bindgen::JsCast;
use web_sys::{
    window, CanvasRenderingContext2d, HtmlCanvasElement, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d, TextMetrics,
};

#[derive(Debug, Clone)]
pub enum FontError {
    UnknownError,
}

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            _ => write!(fmt, "Unknown error"),
        }
    }
}

impl std::error::Error for FontError {}

#[derive(Clone)]
pub struct FontDataInternal(String, String);

impl FontData for FontDataInternal {
    type ErrorType = FontError;
    fn new(family: FontFamily, style: FontStyle) -> Result<Self, FontError> {
        Ok(FontDataInternal(
            family.as_str().into(),
            style.as_str().into(),
        ))
    }
    fn estimate_layout(&self, size: f64, text: &str) -> Result<LayoutBox, Self::ErrorType> {
        let metrics = measure(&format!("{} {}px {}", self.1, size, self.0), text)
            .ok_or(FontError::UnknownError)?;
        let height = metrics.font_bounding_box_ascent() + metrics.font_bounding_box_descent();
        Ok((
            (0, 0),
            (metrics.width().round() as i32, height.round() as i32),
        ))
    }
}

/// Measure `text` in the CSS `font` with an `OffscreenCanvas`, or with a canvas element where
/// `OffscreenCanvas` doesn't exist.
fn measure(font: &str, text: &str) -> Option<TextMetrics> {
    let offscreen = || -> Option<TextMetrics> {
        let context: OffscreenCanvasRenderingContext2d = OffscreenCanvas::new(0, 0)
            .ok()?
            .get_context("2d")
            .ok()??
            .dyn_into()
            .ok()?;
        context.set_font(font);
        context.measure_text(text).ok()
    };
    let element = || -> Option<TextMetrics> {
        let canvas: HtmlCanvasElement = window()?
            .document()?
            .create_element("canvas")
            .ok()?
            .dyn_into()
            .ok()?;
        let context: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
        context.set_font(font);
        context.measure_text(text).ok()
    };
    offscreen().or_else(element)
}
