// TODO: ? operator is very slow. See issue #58 for details
macro_rules! check_result {
    ($e:expr) => {
        let result = $e;
        if result.is_err() {
            return result;
        }
    };
}

mod line;
pub use line::draw_line;

mod rect;
pub use rect::draw_rect;

mod circle;
pub use circle::draw_circle;

mod polygon;
pub use polygon::fill_polygon;

mod path;
pub use path::polygonize;
