use super::{FontData, FontFamily, FontStyle, LayoutBox};
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d};

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
        let canvas = OffscreenCanvas::new(0, 0).expect("offscreen canvas");
        let context = canvas
            .get_context("2d")
            .expect("getContext")
            .expect("context for 2d not null")
            .dyn_into::<OffscreenCanvasRenderingContext2d>()
            .expect("cast");
        context.set_font(&format!(
            "{} {}px {}",
            self.1.as_str(),
            size,
            self.0.as_str(),
        ));
        let metrics = context
            .measure_text(text)
            .expect("measure_text to return metrics");
        let width = metrics.width();
        let height = metrics.font_bounding_box_ascent() + metrics.font_bounding_box_descent();
        Ok(((0, 0), (width as i32, height as i32)))
    }
}
