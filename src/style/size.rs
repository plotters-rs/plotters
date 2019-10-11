use crate::coord::CoordTranslate;
use crate::drawing::DrawingArea;
use crate::drawing::DrawingBackend;

/// The trait indicates that the type has a dimension info
pub trait HasDimension {
    fn dim(&self) -> (u32, u32);
}

impl<T: DrawingBackend> HasDimension for T {
    fn dim(&self) -> (u32, u32) {
        self.get_size()
    }
}

impl<D: DrawingBackend, C: CoordTranslate> HasDimension for DrawingArea<D, C> {
    fn dim(&self) -> (u32, u32) {
        self.dim_in_pixel()
    }
}

impl HasDimension for (u32, u32) {
    fn dim(&self) -> (u32, u32) {
        *self
    }
}

/// The trait that describes a size
pub trait SizeDesc {
    fn in_pixels<T: HasDimension>(&self, parent: &T) -> i32;
}

impl<T: Into<i32> + Clone> SizeDesc for T {
    fn in_pixels<D: HasDimension>(&self, _parent: &D) -> i32 {
        self.clone().into()
    }
}

pub enum RelativeSize {
    Height(f64),
    Width(f64),
}

impl RelativeSize {
    pub fn min(self, min_sz: i32) -> RelativeSizeWithBound {
        RelativeSizeWithBound {
            size: self,
            min: Some(min_sz),
            max: None,
        }
    }

    pub fn max(self, max_sz: i32) -> RelativeSizeWithBound {
        RelativeSizeWithBound {
            size: self,
            max: Some(max_sz),
            min: None,
        }
    }
}

impl SizeDesc for RelativeSize {
    fn in_pixels<D: HasDimension>(&self, parent: &D) -> i32 {
        let (w, h) = parent.dim();
        match self {
            RelativeSize::Width(p) => *p * w as f64,
            RelativeSize::Height(p) => *p * h as f64,
        }
        .round() as i32
    }
}

pub trait AsRelativeWidth: Into<f64> {
    fn percent_width(self) -> RelativeSize {
        RelativeSize::Width(self.into() / 100.0)
    }
}

pub trait AsRelativeHeight: Into<f64> {
    fn percent_height(self) -> RelativeSize {
        RelativeSize::Width(self.into() / 100.0)
    }
}

impl<T: Into<f64>> AsRelativeWidth for T {}
impl<T: Into<f64>> AsRelativeHeight for T {}

pub struct RelativeSizeWithBound {
    size: RelativeSize,
    min: Option<i32>,
    max: Option<i32>,
}

impl RelativeSizeWithBound {
    pub fn min(mut self, min_sz: i32) -> RelativeSizeWithBound {
        self.min = Some(min_sz);
        self
    }

    pub fn max(mut self, max_sz: i32) -> RelativeSizeWithBound {
        self.max = Some(max_sz);
        self
    }
}

impl SizeDesc for RelativeSizeWithBound {
    fn in_pixels<D: HasDimension>(&self, parent: &D) -> i32 {
        let size = self.size.in_pixels(parent);
        let size_lower_capped = self.min.map_or(size, |x| x.max(size));
        let size_upper_capped = self.max.map_or(size_lower_capped, |x| x.min(size));
        size_upper_capped
    }
}
