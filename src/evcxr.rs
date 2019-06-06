use crate::coord::Shift;
use crate::drawing::{DrawingArea, IntoDrawingArea, SVGBackend};

/// The wrapper for the generated SVG
pub struct SVGWrapper(Vec<u8>);

impl SVGWrapper {
    pub fn evcxr_display(&self) {
        let svg = String::from_utf8_lossy(self.0.as_slice());
        println!(
            "EVCXR_BEGIN_CONTENT text/html\n<svg>{}</svg>\nEVCXR_END_CONTENT",
            svg
        );
    }
}

/// Start drawing an evcxr figure
pub fn evcxr_figure<
    Draw: FnOnce(DrawingArea<SVGBackend, Shift>) -> Result<(), Box<dyn std::error::Error>>,
>(
    size: (u32, u32),
    draw: Draw,
) -> SVGWrapper {
    let mut buffer = vec![];
    let root = SVGBackend::with_buffer(&mut buffer, size).into_drawing_area();
    draw(root).expect("Drawing failure");
    SVGWrapper(buffer)
}
