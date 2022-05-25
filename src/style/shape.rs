use super::color::{Color, RGBAColor};
use plotters_backend::{BackendColor, BackendStyle};

/// Style for any of shape
#[derive(Copy, Clone)]
pub struct ShapeStyle {
    pub color: RGBAColor,
    pub filled: bool,
    pub stroke_width: u32,
}

impl ShapeStyle {
    /// Make a filled shape style
    pub fn filled(&self) -> Self {
        Self {
            color: self.color.to_rgba(),
            filled: true,
            stroke_width: self.stroke_width,
        }
    }

    pub fn stroke_width(&self, width: u32) -> Self {
        Self {
            color: self.color.to_rgba(),
            filled: self.filled,
            stroke_width: width,
        }
    }
}

impl<T: Color> From<T> for ShapeStyle {
    fn from(f: T) -> Self {
        ShapeStyle {
            color: f.to_rgba(),
            filled: false,
            stroke_width: 1,
        }
    }
}

impl BackendStyle for ShapeStyle {
    fn color(&self) -> BackendColor {
        self.color.to_backend_color()
    }
    fn stroke_width(&self) -> u32 {
        self.stroke_width
    }
}
