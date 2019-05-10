use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};
use super::FontData;

#[derive(Debug, Clone)]
pub enum FontError {
    UnknownError
}

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        return match self {
            _ => write!(fmt, "Unknown error")
        };
    }
}

impl std::error::Error for FontError {}

#[derive(Clone)]
pub struct FontDataInternal(String);


impl FontData for FontDataInternal {
    type ErrorType = FontError;
    fn new(face:&str) -> Result<Self, FontError> {
        return Ok(FontDataInternal(face.to_string()));
    }
    fn estimate_layout(&self, size:f64, text:&str) -> Result<((i32, i32), (i32, i32)), Self::ErrorType> {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();
        let span = document.create_element("span").unwrap();
        span.set_text_content(Some(text));
        span.set_attribute("style", &format!("display: inline-block; font-family:{}; font-size: {}px; position: fixed; top: 100%", self.0, size)).unwrap();
        let span = span.into();
        body.append_with_node_1(&span).unwrap();
        let elem = JsCast::dyn_into::<HtmlElement>(span).unwrap();
        let height = elem.offset_height() as i32;
        let width  = elem.offset_width() as i32;
        elem.remove();
        return Ok(((0,0), (width, height)));
    }
}
