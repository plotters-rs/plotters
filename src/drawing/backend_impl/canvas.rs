use js_sys::JSON;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
use crate::style::text_anchor::{HPos, VPos};
use crate::style::{Color, FontTransform, RGBAColor, TextStyle};

/// The backend that is drawing on the HTML canvas
/// TODO: Support double buffering
pub struct CanvasBackend {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

pub struct CanvasError(String);

impl std::fmt::Display for CanvasError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        return write!(fmt, "Canvas Error: {}", self.0);
    }
}

impl std::fmt::Debug for CanvasError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        return write!(fmt, "CanvasError({})", self.0);
    }
}

impl From<JsValue> for DrawingErrorKind<CanvasError> {
    fn from(e: JsValue) -> DrawingErrorKind<CanvasError> {
        DrawingErrorKind::DrawingError(CanvasError(
            JSON::stringify(&e)
                .map(|s| Into::<String>::into(&s))
                .unwrap_or_else(|_| "Unknown".to_string()),
        ))
    }
}

impl std::error::Error for CanvasError {}

impl CanvasBackend {
    fn init_backend(canvas: HtmlCanvasElement) -> Option<Self> {
        let context: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
        Some(CanvasBackend { canvas, context })
    }

    /// Create a new drawing backend backed with an HTML5 canvas object with given Id
    /// - `elem_id` The element id for the canvas
    /// - Return either some drawing backend has been created, or none in error case
    pub fn new(elem_id: &str) -> Option<Self> {
        let document = window()?.document()?;
        let canvas = document.get_element_by_id(elem_id)?;
        let canvas: HtmlCanvasElement = canvas.dyn_into().ok()?;
        Self::init_backend(canvas)
    }

    /// Create a new drawing backend backend with a HTML5 canvas object passed in
    /// - `canvas` The object we want to use as backend
    /// - Return either the drawing backend or None for error
    pub fn with_canvas_object(canvas: HtmlCanvasElement) -> Option<Self> {
        Self::init_backend(canvas)
    }
}

fn make_canvas_color(color: RGBAColor) -> JsValue {
    let (r, g, b) = color.rgb();
    let a = color.alpha();
    format!("rgba({},{},{},{})", r, g, b, a).into()
}

impl DrawingBackend for CanvasBackend {
    type ErrorType = CanvasError;

    fn get_size(&self) -> (u32, u32) {
        // Getting just canvas.width gives poor results on HighDPI screens.
        let rect = self.canvas.get_bounding_client_rect();
        (rect.width() as u32, rect.height() as u32)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<CanvasError>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<CanvasError>> {
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        style: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<CanvasError>> {
        if style.alpha() == 0.0 {
            return Ok(());
        }

        self.context
            .set_fill_style(&make_canvas_color(style.as_color()));
        self.context
            .fill_rect(f64::from(point.0), f64::from(point.1), 1.0, 1.0);
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.as_color().alpha() == 0.0 {
            return Ok(());
        }

        self.context
            .set_stroke_style(&make_canvas_color(style.as_color()));
        self.context.begin_path();
        self.context.move_to(f64::from(from.0), f64::from(from.1));
        self.context.line_to(f64::from(to.0), f64::from(to.1));
        self.context.stroke();
        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.as_color().alpha() == 0.0 {
            return Ok(());
        }
        if fill {
            self.context
                .set_fill_style(&make_canvas_color(style.as_color()));
            self.context.fill_rect(
                f64::from(upper_left.0),
                f64::from(upper_left.1),
                f64::from(bottom_right.0 - upper_left.0),
                f64::from(bottom_right.1 - upper_left.1),
            );
        } else {
            self.context
                .set_stroke_style(&make_canvas_color(style.as_color()));
            self.context.stroke_rect(
                f64::from(upper_left.0),
                f64::from(upper_left.1),
                f64::from(bottom_right.0 - upper_left.0),
                f64::from(bottom_right.1 - upper_left.1),
            );
        }
        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.as_color().alpha() == 0.0 {
            return Ok(());
        }
        let mut path = path.into_iter();
        self.context.begin_path();
        if let Some(start) = path.next() {
            self.context
                .set_stroke_style(&make_canvas_color(style.as_color()));
            self.context.move_to(f64::from(start.0), f64::from(start.1));
            for next in path {
                self.context.line_to(f64::from(next.0), f64::from(next.1));
            }
        }
        self.context.stroke();
        Ok(())
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.as_color().alpha() == 0.0 {
            return Ok(());
        }
        let mut path = path.into_iter();
        self.context.begin_path();
        if let Some(start) = path.next() {
            self.context
                .set_fill_style(&make_canvas_color(style.as_color()));
            self.context.move_to(f64::from(start.0), f64::from(start.1));
            for next in path {
                self.context.line_to(f64::from(next.0), f64::from(next.1));
            }
            self.context.close_path();
        }
        self.context.fill();
        Ok(())
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.as_color().alpha() == 0.0 {
            return Ok(());
        }
        if fill {
            self.context
                .set_fill_style(&make_canvas_color(style.as_color()));
        } else {
            self.context
                .set_stroke_style(&make_canvas_color(style.as_color()));
        }
        self.context.begin_path();
        self.context.arc(
            f64::from(center.0),
            f64::from(center.1),
            f64::from(radius),
            0.0,
            std::f64::consts::PI * 2.0,
        )?;
        if fill {
            self.context.fill();
        } else {
            self.context.stroke();
        }
        Ok(())
    }

    fn draw_text(
        &mut self,
        text: &str,
        style: &TextStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let font = &style.font;
        let color = &style.color;
        if color.alpha() == 0.0 {
            return Ok(());
        }

        let (mut x, mut y) = (pos.0, pos.1);

        let degree = match font.get_transform() {
            FontTransform::None => 0.0,
            FontTransform::Rotate90 => 90.0,
            FontTransform::Rotate180 => 180.0,
            FontTransform::Rotate270 => 270.0,
        } / 180.0
            * std::f64::consts::PI;

        if degree != 0.0 {
            self.context.save();
            self.context.translate(f64::from(x), f64::from(y))?;
            self.context.rotate(degree)?;
            x = 0;
            y = 0;
        }

        let text_baseline = match style.pos.v_pos {
            VPos::Top => "top",
            VPos::Center => "middle",
            VPos::Bottom => "bottom",
        };
        self.context.set_text_baseline(text_baseline);

        let text_align = match style.pos.h_pos {
            HPos::Left => "start",
            HPos::Right => "end",
            HPos::Center => "center",
        };
        self.context.set_text_align(text_align);

        self.context
            .set_fill_style(&make_canvas_color(color.clone()));
        self.context.set_font(&format!(
            "{} {}px {}",
            font.get_style().as_str(),
            font.get_size(),
            font.get_name()
        ));
        self.context.fill_text(text, f64::from(x), f64::from(y))?;

        if degree != 0.0 {
            self.context.restore();
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::element::Circle;
    use crate::prelude::*;
    use crate::style::text_anchor::Pos;
    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use wasm_bindgen_test::*;
    use web_sys::Document;

    wasm_bindgen_test_configure!(run_in_browser);

    fn create_canvas(document: &Document, id: &str, width: u32, height: u32) -> HtmlCanvasElement {
        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        let div = document.create_element("div").unwrap();
        div.append_child(&canvas).unwrap();
        document.body().unwrap().append_child(&div).unwrap();
        canvas.set_attribute("id", id).unwrap();
        canvas.set_width(width);
        canvas.set_height(height);
        canvas
    }

    fn check_content(document: &Document, id: &str) {
        let canvas = document
            .get_element_by_id(id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        let data_uri = canvas.to_data_url().unwrap();
        let prefix = "data:image/png;base64,";
        assert!(&data_uri.starts_with(prefix));
    }

    fn draw_mesh_with_custom_ticks(tick_size: i32, test_name: &str) {
        let document = window().unwrap().document().unwrap();
        let canvas = create_canvas(&document, test_name, 500, 500);
        let backend = CanvasBackend::with_canvas_object(canvas).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        let mut chart = ChartBuilder::on(&root)
            .caption("This is a test", ("sans-serif", 20))
            .set_all_label_area_size(40)
            .build_ranged(0..10, 0..10)
            .unwrap();

        chart
            .configure_mesh()
            .set_all_tick_mark_size(tick_size)
            .draw()
            .unwrap();

        check_content(&document, test_name);
    }

    #[wasm_bindgen_test]
    fn test_draw_mesh_no_ticks() {
        draw_mesh_with_custom_ticks(0, "test_draw_mesh_no_ticks");
    }

    #[wasm_bindgen_test]
    fn test_draw_mesh_negative_ticks() {
        draw_mesh_with_custom_ticks(-10, "test_draw_mesh_negative_ticks");
    }

    #[wasm_bindgen_test]
    fn test_text_draw() {
        let document = window().unwrap().document().unwrap();
        let canvas = create_canvas(&document, "test_text_draw", 1500, 800);
        let backend = CanvasBackend::with_canvas_object(canvas).expect("cannot find canvas");
        let root = backend.into_drawing_area();
        let root = root
            .titled("Image Title", ("sans-serif", 60).into_font())
            .unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption("All anchor point positions", ("sans-serif", 20))
            .set_all_label_area_size(40)
            .build_ranged(0..100, 0..50)
            .unwrap();

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .x_desc("X Axis")
            .y_desc("Y Axis")
            .draw()
            .unwrap();

        let ((x1, y1), (x2, y2), (x3, y3)) = ((-30, 30), (0, -30), (30, 30));

        for (dy, trans) in [
            FontTransform::None,
            FontTransform::Rotate90,
            FontTransform::Rotate180,
            FontTransform::Rotate270,
        ]
        .iter()
        .enumerate()
        {
            for (dx1, h_pos) in [HPos::Left, HPos::Right, HPos::Center].iter().enumerate() {
                for (dx2, v_pos) in [VPos::Top, VPos::Center, VPos::Bottom].iter().enumerate() {
                    let x = 150_i32 + (dx1 as i32 * 3 + dx2 as i32) * 150;
                    let y = 120 + dy as i32 * 150;
                    let draw = |x, y, text| {
                        root.draw(&Circle::new((x, y), 3, &BLACK.mix(0.5))).unwrap();
                        let style = TextStyle::from(("sans-serif", 20).into_font())
                            .pos(Pos::new(*h_pos, *v_pos))
                            .transform(trans.clone());
                        root.draw_text(text, &style, (x, y)).unwrap();
                    };
                    draw(x + x1, y + y1, "dood");
                    draw(x + x2, y + y2, "dog");
                    draw(x + x3, y + y3, "goog");
                }
            }
        }
        check_content(&document, "test_text_draw");
    }

    #[wasm_bindgen_test]
    fn test_text_clipping() {
        let (width, height) = (500_i32, 500_i32);
        let document = window().unwrap().document().unwrap();
        let canvas = create_canvas(&document, "test_text_clipping", width as u32, height as u32);
        let backend = CanvasBackend::with_canvas_object(canvas).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        let style = TextStyle::from(("sans-serif", 20).into_font())
            .pos(Pos::new(HPos::Center, VPos::Center));
        root.draw_text("TOP LEFT", &style, (0, 0)).unwrap();
        root.draw_text("TOP CENTER", &style, (width / 2, 0))
            .unwrap();
        root.draw_text("TOP RIGHT", &style, (width, 0)).unwrap();

        root.draw_text("MIDDLE LEFT", &style, (0, height / 2))
            .unwrap();
        root.draw_text("MIDDLE RIGHT", &style, (width, height / 2))
            .unwrap();

        root.draw_text("BOTTOM LEFT", &style, (0, height)).unwrap();
        root.draw_text("BOTTOM CENTER", &style, (width / 2, height))
            .unwrap();
        root.draw_text("BOTTOM RIGHT", &style, (width, height))
            .unwrap();

        check_content(&document, "test_text_clipping");
    }

    #[wasm_bindgen_test]
    fn test_series_labels() {
        let (width, height) = (500, 500);
        let document = window().unwrap().document().unwrap();
        let canvas = create_canvas(&document, "test_series_labels", width, height);
        let backend = CanvasBackend::with_canvas_object(canvas).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        let mut chart = ChartBuilder::on(&root)
            .caption("All series label positions", ("sans-serif", 20))
            .set_all_label_area_size(40)
            .build_ranged(0..50, 0..50)
            .unwrap();

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()
            .unwrap();

        chart
            .draw_series(std::iter::once(Circle::new((5, 15), 5, &RED)))
            .expect("Drawing error")
            .label("Series 1")
            .legend(|(x, y)| Circle::new((x, y), 3, RED.filled()));

        chart
            .draw_series(std::iter::once(Circle::new((5, 15), 10, &BLUE)))
            .expect("Drawing error")
            .label("Series 2")
            .legend(|(x, y)| Circle::new((x, y), 3, BLUE.filled()));

        for pos in vec![
            SeriesLabelPosition::UpperLeft,
            SeriesLabelPosition::MiddleLeft,
            SeriesLabelPosition::LowerLeft,
            SeriesLabelPosition::UpperMiddle,
            SeriesLabelPosition::MiddleMiddle,
            SeriesLabelPosition::LowerMiddle,
            SeriesLabelPosition::UpperRight,
            SeriesLabelPosition::MiddleRight,
            SeriesLabelPosition::LowerRight,
            SeriesLabelPosition::Coordinate(70, 70),
        ]
        .into_iter()
        {
            chart
                .configure_series_labels()
                .border_style(&BLACK.mix(0.5))
                .position(pos)
                .draw()
                .expect("Drawing error");
        }

        check_content(&document, "test_series_labels");
    }

    #[wasm_bindgen_test]
    fn test_draw_pixel_alphas() {
        let (width, height) = (100_i32, 100_i32);
        let document = window().unwrap().document().unwrap();
        let canvas = create_canvas(
            &document,
            "test_draw_pixel_alphas",
            width as u32,
            height as u32,
        );
        let backend = CanvasBackend::with_canvas_object(canvas).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        for i in -20..20 {
            let alpha = i as f64 * 0.1;
            root.draw_pixel((50 + i, 50 + i), &BLACK.mix(alpha))
                .unwrap();
        }

        check_content(&document, "test_draw_pixel_alphas");
    }
}
