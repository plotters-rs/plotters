/*!
The SVG image drawing backend
*/
pub use svg as svg_types;

use svg::node::element::{Circle, Line, Polygon, Polyline, Rectangle, Text};
use svg::Document;

use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
use crate::style::text_anchor::{HPos, VPos};
use crate::style::{Color, FontStyle, FontTransform, RGBAColor, TextStyle};

use std::io::{Cursor, Error};
use std::path::Path;

fn make_svg_color<C: Color>(color: &C) -> String {
    let (r, g, b) = color.rgb();
    return format!("#{:02X}{:02X}{:02X}", r, g, b);
}

fn make_svg_opacity<C: Color>(color: &C) -> String {
    return format!("{}", color.alpha());
}

enum Target<'a> {
    File(&'a Path),
    Buffer(Cursor<&'a mut Vec<u8>>),
}

/// The SVG image drawing backend
pub struct SVGBackend<'a> {
    target: Target<'a>,
    size: (u32, u32),
    document: Option<Document>,
    saved: bool,
}

impl<'a> SVGBackend<'a> {
    pub fn update_document<F: FnOnce(Document) -> Document>(&mut self, op: F) {
        let mut temp = None;
        std::mem::swap(&mut temp, &mut self.document);
        self.document = Some(op(temp.unwrap()));
    }

    /// Create a new SVG drawing backend
    pub fn new<T: AsRef<Path> + ?Sized>(path: &'a T, size: (u32, u32)) -> Self {
        Self {
            target: Target::File(path.as_ref()),
            size,
            document: Some(Document::new().set("width", size.0).set("height", size.1)),
            saved: false,
        }
    }

    /// Create a new SVG drawing backend and store the document into a u8 buffer
    pub fn with_buffer(buf: &'a mut Vec<u8>, size: (u32, u32)) -> Self {
        Self {
            target: Target::Buffer(Cursor::new(buf)),
            size,
            document: Some(Document::new().set("width", size.0).set("height", size.1)),
            saved: false,
        }
    }
}

impl<'a> DrawingBackend for SVGBackend<'a> {
    type ErrorType = Error;

    fn get_size(&self) -> (u32, u32) {
        self.size
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Error>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Error>> {
        if !self.saved {
            match self.target {
                Target::File(path) => svg::save(path, self.document.as_ref().unwrap())
                    .map_err(DrawingErrorKind::DrawingError)?,
                Target::Buffer(ref mut w) => svg::write(w, self.document.as_ref().unwrap())
                    .map_err(DrawingErrorKind::DrawingError)?,
            }
            self.saved = true;
        }
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<Error>> {
        if color.alpha() == 0.0 {
            return Ok(());
        }
        let node = Rectangle::new()
            .set("x", point.0)
            .set("y", point.1)
            .set("width", 1)
            .set("height", 1)
            .set("stroke", "none")
            .set("opacity", make_svg_opacity(color))
            .set("fill", make_svg_color(color));
        self.update_document(|d| d.add(node));
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
        let node = Line::new()
            .set("x1", from.0)
            .set("y1", from.1)
            .set("x2", to.0)
            .set("y2", to.1)
            .set("opacity", make_svg_opacity(&style.as_color()))
            .set("stroke", make_svg_color(&style.as_color()))
            .set("stroke-width", style.stroke_width());
        self.update_document(|d| d.add(node));
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
        let mut node = Rectangle::new()
            .set("x", upper_left.0)
            .set("y", upper_left.1)
            .set("width", bottom_right.0 - upper_left.0)
            .set("height", bottom_right.1 - upper_left.1);

        if !fill {
            node = node
                .set("opacity", make_svg_opacity(&style.as_color()))
                .set("stroke", make_svg_color(&style.as_color()))
                .set("fill", "none");
        } else {
            node = node
                .set("opacity", make_svg_opacity(&style.as_color()))
                .set("fill", make_svg_color(&style.as_color()))
                .set("stroke", "none");
        }

        self.update_document(|d| d.add(node));
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
        let node = Polyline::new()
            .set("fill", "none")
            .set("opacity", make_svg_opacity(&style.as_color()))
            .set("stroke", make_svg_color(&style.as_color()))
            .set("stroke-width", style.stroke_width())
            .set(
                "points",
                path.into_iter().fold(String::new(), |mut s, (x, y)| {
                    s.push_str(&format!("{},{} ", x, y));
                    s
                }),
            );
        self.update_document(|d| d.add(node));
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
        let node = Polygon::new()
            .set("opacity", make_svg_opacity(&style.as_color()))
            .set("fill", make_svg_color(&style.as_color()))
            .set(
                "points",
                path.into_iter().fold(String::new(), |mut s, (x, y)| {
                    s.push_str(&format!("{},{} ", x, y));
                    s
                }),
            );
        self.update_document(|d| d.add(node));
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
        let mut node = Circle::new()
            .set("cx", center.0)
            .set("cy", center.1)
            .set("r", radius);

        if !fill {
            node = node
                .set("opacity", make_svg_opacity(&style.as_color()))
                .set("stroke", make_svg_color(&style.as_color()))
                .set("fill", "none");
        } else {
            node = node
                .set("opacity", make_svg_opacity(&style.as_color()))
                .set("fill", make_svg_color(&style.as_color()))
                .set("stroke", "none");
        }

        self.update_document(|d| d.add(node));
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

        let (x0, y0) = pos;
        let text_anchor = match style.pos.h_pos {
            HPos::Left => "start",
            HPos::Right => "end",
            HPos::Center => "middle",
        };
        let dominant_baseline = match style.pos.v_pos {
            VPos::Top => "hanging",
            VPos::Center => "middle",
            VPos::Bottom => "baseline",
        };

        let node = Text::new()
            .set("x", x0)
            .set("y", y0)
            .set("dominant-baseline", dominant_baseline)
            .set("text-anchor", text_anchor)
            .set("font-family", font.get_name())
            .set("font-size", font.get_size())
            .set("opacity", make_svg_opacity(color))
            .set("fill", make_svg_color(color));

        let node = match font.get_style() {
            FontStyle::Normal => node,
            FontStyle::Bold => node.set("font-weight", "bold"),
            other_style => node.set("font-style", other_style.as_str()),
        };

        let context = svg::node::Text::new(text);
        let trans = font.get_transform();
        let node = match trans {
            FontTransform::Rotate90 => node.set("transform", format!("rotate(90, {}, {})", x0, y0)),
            FontTransform::Rotate180 => {
                node.set("transform", format!("rotate(180, {}, {})", x0, y0))
            }
            FontTransform::Rotate270 => {
                node.set("transform", format!("rotate(270, {}, {})", x0, y0))
            }
            _ => node,
        }
        .add(context);

        self.update_document(|d| d.add(node));

        Ok(())
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    fn blit_bitmap<'b>(
        &mut self,
        pos: BackendCoord,
        (w, h): (u32, u32),
        src: &'b [u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        use image::png::PNGEncoder;
        use svg::node::element::Image;

        let mut data = vec![0; 0];

        {
            let cursor = Cursor::new(&mut data);

            let encoder = PNGEncoder::new(cursor);

            let color = image::ColorType::RGB(8);

            encoder.encode(src, w, h, color).map_err(|e| {
                DrawingErrorKind::DrawingError(Error::new(
                    std::io::ErrorKind::Other,
                    format!("Image error: {}", e),
                ))
            })?;
        }

        let padding = (3 - data.len() % 3) % 3;
        for _ in 0..padding {
            data.push(0);
        }

        let mut rem_bits = 0;
        let mut rem_num = 0;

        fn cvt_base64(from: u8) -> char {
            (if from < 26 {
                b'A' + from
            } else if from < 52 {
                b'a' + from - 26
            } else if from < 62 {
                b'0' + from - 52
            } else if from == 62 {
                b'+'
            } else {
                b'/'
            })
            .into()
        }

        let mut buf = String::new();
        buf.push_str("data:png;base64,");

        for byte in data {
            let value = (rem_bits << (6 - rem_num)) | (byte >> (rem_num + 2));
            rem_bits = byte & ((1 << (2 + rem_num)) - 1);
            rem_num += 2;

            buf.push(cvt_base64(value));
            if rem_num == 6 {
                buf.push(cvt_base64(rem_bits));
                rem_bits = 0;
                rem_num = 0;
            }
        }

        for _ in 0..padding {
            buf.pop();
            buf.push('=');
        }

        let node = Image::new()
            .set("x", pos.0)
            .set("y", pos.1)
            .set("width", w)
            .set("height", h)
            .set("href", buf.as_str());

        self.update_document(|d| d.add(node));

        Ok(())
    }
}

impl Drop for SVGBackend<'_> {
    fn drop(&mut self) {
        if !self.saved {
            self.present().expect("Unable to save the SVG image");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::element::Circle;
    use crate::prelude::*;
    use crate::style::text_anchor::{HPos, Pos, VPos};
    use std::fs;
    use std::path::Path;

    static DST_DIR: &str = "target/test/svg";

    fn checked_save_file(name: &str, content: &str) {
        /*
          Please use the SVG file to manually verify the results.
        */
        assert!(!content.is_empty());
        fs::create_dir_all(DST_DIR).unwrap();
        let file_name = format!("{}.svg", name);
        let file_path = Path::new(DST_DIR).join(file_name);
        println!("{:?} created", file_path);
        fs::write(file_path, &content).unwrap();
    }

    fn draw_mesh_with_custom_ticks(tick_size: i32, test_name: &str) {
        let mut buffer: Vec<u8> = vec![];
        {
            let root = SVGBackend::with_buffer(&mut buffer, (500, 500)).into_drawing_area();

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
        }

        let content = String::from_utf8(buffer).unwrap();
        checked_save_file(test_name, &content);

        assert!(content.contains("This is a test"));
    }

    #[test]
    fn test_draw_mesh_no_ticks() {
        draw_mesh_with_custom_ticks(0, "test_draw_mesh_no_ticks");
    }

    #[test]
    fn test_draw_mesh_negative_ticks() {
        draw_mesh_with_custom_ticks(-10, "test_draw_mesh_negative_ticks");
    }

    #[test]
    fn test_text_alignments() {
        let mut buffer: Vec<u8> = vec![];
        {
            let mut root = SVGBackend::with_buffer(&mut buffer, (500, 500));

            let style = TextStyle::from(("sans-serif", 20).into_font())
                .pos(Pos::new(HPos::Right, VPos::Top));
            root.draw_text("right-align", &style, (150, 50)).unwrap();

            let style = style.pos(Pos::new(HPos::Center, VPos::Top));
            root.draw_text("center-align", &style, (150, 150)).unwrap();

            let style = style.pos(Pos::new(HPos::Left, VPos::Top));
            root.draw_text("left-align", &style, (150, 200)).unwrap();
        }

        let content = String::from_utf8(buffer).unwrap();
        checked_save_file("test_text_alignments", &content);

        for svg_line in content.split("</text>") {
            if let Some(anchor_and_rest) = svg_line.split("text-anchor=\"").nth(1) {
                if anchor_and_rest.starts_with("end") {
                    assert!(anchor_and_rest.contains("right-align"))
                }
                if anchor_and_rest.starts_with("middle") {
                    assert!(anchor_and_rest.contains("center-align"))
                }
                if anchor_and_rest.starts_with("start") {
                    assert!(anchor_and_rest.contains("left-align"))
                }
            }
        }
    }

    #[test]
    fn test_text_draw() {
        let mut buffer: Vec<u8> = vec![];
        {
            let root = SVGBackend::with_buffer(&mut buffer, (1000, 500)).into_drawing_area();

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
                        let x = 100_i32 + (dx1 as i32 * 3 + dx2 as i32) * 100;
                        let y = 100 + dy as i32 * 100;
                        root.draw(&Circle::new((x, y), 3, &BLACK.mix(0.5))).unwrap();
                        let style = TextStyle::from(("sans-serif", 20).into_font())
                            .pos(Pos::new(*h_pos, *v_pos))
                            .transform(trans.clone());
                        root.draw_text("test", &style, (x, y)).unwrap();
                    }
                }
            }
        }

        let content = String::from_utf8(buffer).unwrap();
        checked_save_file("test_text_draw", &content);

        assert_eq!(content.matches("test").count(), 36);
    }

    #[test]
    fn test_text_clipping() {
        let mut buffer: Vec<u8> = vec![];
        {
            let (width, height) = (500_i32, 500_i32);
            let root = SVGBackend::with_buffer(&mut buffer, (width as u32, height as u32))
                .into_drawing_area();

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
        }

        let content = String::from_utf8(buffer).unwrap();
        checked_save_file("test_text_clipping", &content);
    }

    #[test]
    fn test_series_labels() {
        let mut buffer: Vec<u8> = vec![];
        {
            let (width, height) = (500, 500);
            let root = SVGBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();

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
        }

        let content = String::from_utf8(buffer).unwrap();
        checked_save_file("test_series_labels", &content);
    }
}
