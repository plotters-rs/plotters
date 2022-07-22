use crate::coord::Shift;
use crate::drawing::{DrawingArea, IntoDrawingArea};
use plotters_svg::SVGBackend;
use plotters_backend::DrawingBackend;
use plotters_bitmap::BitMapBackend;

/// The wrapper for the generated SVG
pub struct SVGWrapper(String, String);

impl SVGWrapper {
    /// Displays the contents of the `SVGWrapper` struct.
    pub fn evcxr_display(&self) {
        println!("{:?}", self);
    }
    /// Sets the style of the `SVGWrapper` struct.
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

// #[cfg(all(feature = "evcxr", feature = "bitmap_backend"))]
// pub struct BitMapWrapper(String, String);

// #[cfg(all(feature = "evcxr", feature = "bitmap_backend"))]
// impl BitMapWrapper {
//     pub fn evcxr_display(&self) {
//         println!("{:?}", self);
//     }

//     pub fn style<S: Into<String>>(mut self, style: S) -> Self {
//         self.1 = style.into();
//         self
//     }
// }

// #[cfg(feature = "evcxr_bitmap_figure")]
// #[cfg(all(feature = "evcxr", feature = "bitmap_backend"))]
// impl std::fmt::Debug for BitMapWrapper {
//     fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//         let enc = self.0.as_str();
//         write!(
//             formatter,
//             "EVCXR_BEGIN_CONTENT text/html\n<img style=\"{}\" src=\"data:image/png;base64,{}\"/>\nEVCXR_END_CONTENT",
//             self.1, enc
//         )
//     }
// }


/// Start drawing an evcxr figure
#[cfg(all(feature = "evcxr", feature = "bitmap_backend"))]
pub fn evcxr_bitmap_figure<
    Draw: FnOnce(DrawingArea<BitMapBackend, Shift>) -> Result<(), Box<dyn std::error::Error>>,
>(
    size: (u32, u32),
    draw: Draw,
) -> SVGWrapper {
    let pixel_size = 3;
    let mut buf = Vec::new();
    buf.resize((size.0 as usize) * (size.1 as usize) * pixel_size, 0);
    let root = BitMapBackend::with_buffer(&mut buf, size).into_drawing_area();
    draw(root).expect("Drawing failure");
    let mut buffer = "".to_string();
    {
        let mut svg_root = SVGBackend::with_string(&mut buffer, size);
        svg_root.blit_bitmap((0, 0), size, &buf).expect("Failure converting to SVG");
    }
    SVGWrapper(buffer.clone(), "".to_string())
}

