use cairo::{Context as CairoContext, FontSlant, FontWeight, Status as CairoStatus};

#[allow(unused_imports)]
use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
#[allow(unused_imports)]
use crate::style::{Color, FontDesc, FontTransform, RGBAColor};

/// The drawing backend that is backed with a Cairo context
pub struct CairoBackend<'a> {
    context: &'a CairoContext,
    width: u32,
    height: u32,
    init_flag: bool,
}

#[derive(Debug)]
pub struct CairoError(CairoStatus);

impl std::fmt::Display for CairoError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl std::error::Error for CairoError {}

impl<'a> CairoBackend<'a> {
    fn call_cairo<F: Fn(&CairoContext)>(&self, f: F) -> Result<(), DrawingErrorKind<CairoError>> {
        f(self.context);
        if self.context.status() == CairoStatus::Success {
            return Ok(());
        }
        Err(DrawingErrorKind::DrawingError(CairoError(
            self.context.status(),
        )))
    }

    fn set_color(&self, color: &RGBAColor) -> Result<(), DrawingErrorKind<CairoError>> {
        self.call_cairo(|c| {
            c.set_source_rgba(
                f64::from(color.rgb().0) / 255.0,
                f64::from(color.rgb().1) / 255.0,
                f64::from(color.rgb().2) / 255.0,
                f64::from(color.alpha()),
            )
        })?;
        Ok(())
    }

    fn set_stroke_width(&self, width: u32) -> Result<(), DrawingErrorKind<CairoError>> {
        self.call_cairo(|c| c.set_line_width(f64::from(width)))?;
        Ok(())
    }

    pub fn new(context: &'a CairoContext, (w, h): (u32, u32)) -> Result<Self, CairoError> {
        let ret = Self {
            context,
            width: w,
            height: h,
            init_flag: false,
        };
        Ok(ret)
    }
}

impl<'a> DrawingBackend for CairoBackend<'a> {
    type ErrorType = CairoError;

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if !self.init_flag {
            let (x0, y0, x1, y1) = self.context.clip_extents();
            self.call_cairo(|c| {
                c.scale(
                    (x1 - x0) / f64::from(self.width),
                    (y1 - y0) / f64::from(self.height),
                )
            })?;
            self.init_flag = true;
        }
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.call_cairo(|c| c.rectangle(f64::from(point.0), f64::from(point.1), 1.0, 1.0))?;
        self.call_cairo(|c| {
            c.set_source_rgba(
                f64::from(color.rgb().0) / 255.0,
                f64::from(color.rgb().1) / 255.0,
                f64::from(color.rgb().2) / 255.0,
                f64::from(color.alpha()),
            )
        })?;
        self.call_cairo(|c| c.fill())?;
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.call_cairo(|c| c.move_to(f64::from(from.0), f64::from(from.1)))?;

        self.set_color(&style.as_color())?;
        self.set_stroke_width(style.stroke_width())?;

        self.call_cairo(|c| c.line_to(f64::from(to.0), f64::from(to.1)))?;
        self.call_cairo(|c| c.stroke())?;
        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.set_color(&style.as_color())?;
        self.set_stroke_width(style.stroke_width())?;

        self.call_cairo(|c| {
            c.rectangle(
                f64::from(upper_left.0),
                f64::from(upper_left.1),
                f64::from(bottom_right.0 - upper_left.0),
                f64::from(bottom_right.1 - upper_left.1),
            )
        })?;

        if fill {
            self.call_cairo(|c| c.fill())?;
        } else {
            self.call_cairo(|c| c.stroke())?;
        }

        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.set_color(&style.as_color())?;
        self.set_stroke_width(style.stroke_width())?;

        let mut path = path.into_iter();

        if let Some((x, y)) = path.next() {
            self.call_cairo(|c| c.move_to(f64::from(x), f64::from(y)))?;
        }

        for (x, y) in path {
            self.call_cairo(|c| c.line_to(f64::from(x), f64::from(y)))?;
        }

        self.call_cairo(|c| c.stroke())?;

        Ok(())
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.set_color(&style.as_color())?;
        self.set_stroke_width(style.stroke_width())?;

        let mut path = path.into_iter();

        if let Some((x, y)) = path.next() {
            self.call_cairo(|c| c.move_to(f64::from(x), f64::from(y)))?;
        } else {
            return Ok(());
        }

        for (x, y) in path {
            self.call_cairo(|c| c.line_to(f64::from(x), f64::from(y)))?;
        }

        self.call_cairo(|c| c.close_path())?;
        self.call_cairo(|c| c.fill())?;

        Ok(())
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.set_color(&style.as_color())?;
        self.set_stroke_width(style.stroke_width())?;

        self.call_cairo(|c| {
            c.arc(
                f64::from(center.0),
                f64::from(center.1),
                f64::from(radius),
                0.0,
                std::f64::consts::PI * 2.0,
            )
        })?;

        if fill {
            self.call_cairo(|c| c.fill())?;
        } else {
            self.call_cairo(|c| c.stroke())?;
        }
        Ok(())
    }

    fn draw_text<'b>(
        &mut self,
        text: &str,
        font: &FontDesc<'b>,
        pos: BackendCoord,
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (mut x, mut y) = (pos.0, pos.1);

        let degree = match font.get_transform() {
            FontTransform::None => 0.0,
            FontTransform::Rotate90 => 90.0,
            FontTransform::Rotate180 => 180.0,
            FontTransform::Rotate270 => 270.0,
        } / 180.0
            * std::f64::consts::PI;

        let layout = font.layout_box(text).map_err(DrawingErrorKind::FontError)?;

        if degree != 0.0 {
            self.call_cairo(|c| c.save())?;
            let offset = font.get_transform().offset(layout);
            self.call_cairo(|c| c.translate(f64::from(x + offset.0), f64::from(y + offset.1)))?;
            self.call_cairo(|c| c.rotate(degree))?;
            x = 0;
            y = 0;
        }

        self.call_cairo(|c| {
            c.select_font_face(font.get_name(), FontSlant::Normal, FontWeight::Normal)
        })?;
        let actual_size = font.get_size();
        self.call_cairo(|c| c.set_font_size(actual_size))?;
        self.set_color(&color)?;
        self.call_cairo(|c| c.move_to(f64::from(x), f64::from(y - (layout.0).1)))?;
        self.call_cairo(|c| c.show_text(text))?;

        if degree != 0.0 {
            self.call_cairo(|c| c.restore())?;
        }
        Ok(())
    }
}
