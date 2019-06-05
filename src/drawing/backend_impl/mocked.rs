use crate::coord::Shift;
use crate::drawing::area::IntoDrawingArea;
use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
use crate::drawing::DrawingArea;
use crate::style::{Color, FontDesc};

pub struct RGBA(pub u8, pub u8, pub u8, pub f64);

pub struct MockedBackend {
    height: u32,
    width: u32,
    init_count: u32,
    draw_count: u32,
    check_draw_pixel: Option<Box<dyn FnMut(RGBA, BackendCoord)>>,
    check_draw_line: Option<Box<dyn FnMut(RGBA, BackendCoord, BackendCoord)>>,
    check_draw_rect: Option<Box<dyn FnMut(RGBA, bool, BackendCoord, BackendCoord)>>,
    check_draw_path: Option<Box<dyn FnMut(RGBA, Vec<BackendCoord>)>>,
    check_draw_circle: Option<Box<dyn FnMut(RGBA, bool, BackendCoord, u32)>>,
    check_draw_text: Option<Box<dyn FnMut(RGBA, &str, f64, BackendCoord, &str)>>,
}

macro_rules! def_set_checker_func {
    ($name:ident, $($param:ty),*) => {
        pub fn $name<T: FnMut($($param,)*) + 'static>(&mut self, check:T) -> &mut Self {
            self.$name = Some(Box::new(check));
            self
        }
    }
}

impl MockedBackend {
    pub fn new(width: u32, height: u32) -> Self {
        MockedBackend {
            height,
            width,
            init_count: 0,
            draw_count: 0,
            check_draw_pixel: None,
            check_draw_line: None,
            check_draw_rect: None,
            check_draw_path: None,
            check_draw_circle: None,
            check_draw_text: None,
        }
    }

    def_set_checker_func!(check_draw_pixel, RGBA, BackendCoord);
    def_set_checker_func!(check_draw_line, RGBA, BackendCoord, BackendCoord);
    def_set_checker_func!(check_draw_rect, RGBA, bool, BackendCoord, BackendCoord);
    def_set_checker_func!(check_draw_path, RGBA, Vec<BackendCoord>);
    def_set_checker_func!(check_draw_circle, RGBA, bool, BackendCoord, u32);
    def_set_checker_func!(check_draw_text, RGBA, &str, f64, BackendCoord, &str);

    fn check_before_draw(&mut self) {
        self.draw_count += 1;
        assert_eq!(self.init_count, self.draw_count);
    }
}

#[derive(Debug)]
pub struct MockedError;

impl std::fmt::Display for MockedError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "MockedError")
    }
}

impl std::error::Error for MockedError {}

fn to_rgba<T: Color>(color: &T) -> RGBA {
    let (r, g, b) = color.rgb();
    let a = color.alpha();
    RGBA(r, g, b, a)
}

impl DrawingBackend for MockedBackend {
    type ErrorType = MockedError;

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<MockedError>> {
        self.init_count += 1;
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<MockedError>> {
        self.init_count = 0;
        self.draw_count = 0;
        Ok(())
    }

    fn draw_pixel<S: Color>(
        &mut self,
        point: BackendCoord,
        color: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        let color = to_rgba(color);
        if let Some(ref mut checker) = self.check_draw_pixel {
            checker(color, point);
        }
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        let color = to_rgba(style.as_color());
        if let Some(ref mut checker) = self.check_draw_line {
            checker(color, from, to);
        }
        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        let color = to_rgba(style.as_color());
        if let Some(ref mut checker) = self.check_draw_rect {
            checker(color, fill, upper_left, bottom_right);
        }
        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        let color = to_rgba(style.as_color());
        if let Some(ref mut checker) = self.check_draw_path {
            checker(color, path.into_iter().collect());
        }
        Ok(())
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        let color = to_rgba(style.as_color());
        if let Some(ref mut checker) = self.check_draw_circle {
            checker(color, fill, center, radius);
        }
        Ok(())
    }

    fn draw_text<'a, C: Color>(
        &mut self,
        text: &str,
        font: &FontDesc<'a>,
        pos: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.check_before_draw();
        let color = to_rgba(color);
        if let Some(ref mut checker) = self.check_draw_text {
            checker(color, font.get_name(), font.get_size(), pos, text);
        }
        Ok(())
    }
}

pub fn create_mocked_drawing_area<F: FnOnce(&mut MockedBackend)>(
    width: u32,
    height: u32,
    setup: F,
) -> DrawingArea<MockedBackend, Shift> {
    let mut backend = MockedBackend::new(width, height);
    setup(&mut backend);
    backend.into_drawing_area()
}
