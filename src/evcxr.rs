use crate::coord::Shift;
use crate::drawing::{DrawingArea, IntoDrawingArea};
use base64;
use plotters_svg::SVGBackend;
use plotters_bitmap::BitMapBackend;

/// The wrapper for the generated SVG
pub struct SVGWrapper(String, String);

impl SVGWrapper {
    pub fn evcxr_display(&self) {
        println!("{:?}", self);
    }

    pub fn style<S: Into<String>>(mut self, style: S) -> Self {
        self.1 = style.into();
        self
    }
}

impl std::fmt::Debug for SVGWrapper {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let svg = self.0.as_str();
        write!(
            formatter,
            "EVCXR_BEGIN_CONTENT text/html\n<div style=\"{}\">{}</div>\nEVCXR_END_CONTENT",
            self.1, svg
        )
    }
}

/// Start drawing an evcxr figure
pub fn evcxr_figure<
    Draw: FnOnce(DrawingArea<SVGBackend, Shift>) -> Result<(), Box<dyn std::error::Error>>,
>(
    size: (u32, u32),
    draw: Draw,
) -> SVGWrapper {
    let mut buffer = "".to_string();
    let root = SVGBackend::with_string(&mut buffer, size).into_drawing_area();
    draw(root).expect("Drawing failure");
    SVGWrapper(buffer, "".to_string())
}

pub struct BitmapWrapper(String, String);

impl BitmapWrapper {
    pub fn evcxr_display(&self) {
        println!("{:?}", self);
    }

    pub fn style<S: Into<String>>(mut self, style: S) -> Self {
        self.1 = style.into();
        self
    }
}

impl std::fmt::Debug for BitmapWrapper {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let svg = self.0.as_str();
        write!(
            formatter,
            "EVCXR_BEGIN_CONTENT text/html\n<img style=\"{}\" src=\"data:image/png;base64,{}\"/>\nEVCXR_END_CONTENT",
            self.1, svg
        )
    }
}

/// Start drawing an evcxr figure
pub fn evcxr_bitmap_figure<
    Draw: FnOnce(DrawingArea<BitMapBackend, Shift>) -> Result<(), Box<dyn std::error::Error>>,
>(
    size: (u32, u32),
    draw: Draw,
) -> SVGWrapper {
    let pixel_size = plotters_bitmap::bitmap_pixel::RGBPixel::PIXEL_SIZE;
    let mut buf = [u8; (size.0 as usize) * (size.1 as usize) * pixel_size];
    let root = BitMapBackend::with_buffer(&buf, size).into_drawing_area();
    let buffer = base64::encode(&buf);
    draw(root).expect("Drawing failure");
    BitmapWrapper(buffer, "".to_string())
}

pub fn evcxr_animation<
    Draw: FnOnce(DrawingArea<SVGBackend, Shift>) -> Result<(), Box<dyn std::error::Error>>,
>(
    drawing_area: &DrawingArea<SVGBackend, Shift>,
    draws: Draw,
    frames: usize,
    interval: usize,
) -> SVGWrapper {
    todo!();
}