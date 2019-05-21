/*!
The SVG image drawing backend
*/

use svg::node::element::{Circle, Line, Polyline, Rectangle, Text};
use svg::Document;

use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::{Color, FontDesc};

use std::io::Error;

fn make_svg_color<C: Color>(color: &C) -> String {
    let (r, g, b) = color.rgb();
    return format!("#{:.2X}{:.2X}{:.2X}", r, g, b);
}

fn make_svg_opacity<C:Color>(color: &C) -> String {
    return format!("{}", color.alpha());
}

/// The SVG image drawing backend
pub struct SVGBackend<'a> {
    path: &'a str,
    size: (u32, u32),
    document: Option<Document>,
}

impl<'a> SVGBackend<'a> {
    fn update_document<F: FnOnce(Document) -> Document>(&mut self, op: F) {
        let mut temp = None;
        std::mem::swap(&mut temp, &mut self.document);
        self.document = Some(op(temp.unwrap()));
    }
    /// Create a new SVG drawing backend
    pub fn new(path: &'a str, size: (u32, u32)) -> Self {
        return Self {
            path,
            size,
            document: Some(Document::new().set("viewBox", (0, 0, size.0, size.1))),
        };
    }
}

impl<'a> DrawingBackend for SVGBackend<'a> {
    type ErrorType = Error;

    fn get_size(&self) -> (u32, u32) {
        return self.size;
    }

    fn open(&mut self) -> Result<(), DrawingErrorKind<Error>> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), DrawingErrorKind<Error>> {
        return svg::save(self.path, self.document.as_ref().unwrap())
            .map_err(|x| DrawingErrorKind::DrawingError(x));
    }

    fn draw_pixel<C: Color>(
        &mut self,
        point: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Error>> {
        let node = Rectangle::new()
            .set("x", point.0)
            .set("y", point.1)
            .set("width", 1)
            .set("height", 1)
            .set("stroke", "none")
            .set("opacity", make_svg_opacity(color))
            .set("fill", make_svg_color(color));
        self.update_document(|d| d.add(node));
        return Ok(());
    }

    fn draw_line<C: Color>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let node = Line::new()
            .set("x1", from.0)
            .set("y1", from.1)
            .set("x2", to.0)
            .set("y2", to.1)
            .set("opacity", make_svg_opacity(color))
            .set("stroke", make_svg_color(color));
        self.update_document(|d| d.add(node));
        return Ok(());
    }

    fn draw_rect<C: Color>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        color: &C,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let mut node = Rectangle::new()
            .set("x", upper_left.0)
            .set("y", upper_left.1)
            .set("width", bottom_right.0 - upper_left.0)
            .set("height", bottom_right.1 - upper_left.1);

        if !fill {
            node = node
                .set("opacity", make_svg_opacity(color))
                .set("stroke", make_svg_color(color))
                .set("fill", "none");
        } else {
            node = node
                .set("opacity", make_svg_opacity(color))
                .set("fill", make_svg_color(color))
                .set("stroke", "none");
        }

        self.update_document(|d| d.add(node));
        return Ok(());
    }

    fn draw_path<C: Color, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let node = Polyline::new()
            .set("fill", "none")
            .set("opacity", make_svg_opacity(color))
            .set("stroke", make_svg_color(color))
            .set(
                "points",
                path.into_iter().fold(String::new(), |mut s, (x, y)| {
                    s.push_str(&format!("{},{} ", x, y));
                    return s;
                }),
            );
        self.update_document(|d| d.add(node));
        return Ok(());
    }

    fn draw_circle<C: Color>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        color: &C,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let mut node = Circle::new()
            .set("cx", center.0)
            .set("cy", center.1)
            .set("r", radius);

        if !fill {
            node = node
                .set("opacity", make_svg_opacity(color))
                .set("stroke", make_svg_color(color))
                .set("fill", "none");
        } else {
            node = node
                .set("opacity", make_svg_opacity(color))
                .set("fill", make_svg_color(color))
                .set("stroke", "none");
        }

        self.update_document(|d| d.add(node));
        return Ok(());
    }
    fn draw_text<'b, C: Color>(
        &mut self,
        text: &str,
        font: &FontDesc<'b>,
        pos: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let context = svg::node::Text::new(text);
        let ((_, b), (_, _)) = font
            .layout_box(text)
            .map_err(|x| DrawingErrorKind::FontError(x))?;
        let node = Text::new()
            .set("x", pos.0)
            .set("y", pos.1 - b)
            .set("font-famliy", font.get_name())
            .set("font-size", font.get_size())
            .set("opacity", make_svg_opacity(color))
            .set("fill", make_svg_color(color))
            .add(context);
        self.update_document(|d| d.add(node));
        return Ok(());
    }
}
