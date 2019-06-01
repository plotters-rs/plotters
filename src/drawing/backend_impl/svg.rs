/*!
The SVG image drawing backend
*/

use svg::node::element::{Circle, Line, Polyline, Rectangle, Text};
use svg::Document;

use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
use crate::style::{Color, FontDesc, FontTransform};

use std::io::Error;
use std::path::Path;

fn make_svg_color<C: Color>(color: &C) -> String {
    let (r, g, b) = color.rgb();
    return format!("#{:02X}{:02X}{:02X}", r, g, b);
}

fn make_svg_opacity<C: Color>(color: &C) -> String {
    return format!("{}", color.alpha());
}

/// The SVG image drawing backend
pub struct SVGBackend<'a> {
    path: &'a Path,
    size: (u32, u32),
    document: Option<Document>,
    saved: bool,
}

impl<'a> SVGBackend<'a> {
    fn update_document<F: FnOnce(Document) -> Document>(&mut self, op: F) {
        let mut temp = None;
        std::mem::swap(&mut temp, &mut self.document);
        self.document = Some(op(temp.unwrap()));
    }

    /// Create a new SVG drawing backend
    pub fn new<T: AsRef<Path> + ?Sized>(path: &'a T, size: (u32, u32)) -> Self {
        Self {
            path: path.as_ref(),
            size,
            document: Some(Document::new().set("viewBox", (0, 0, size.0, size.1))),
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
        svg::save(self.path, self.document.as_ref().unwrap())
            .map_err(DrawingErrorKind::DrawingError)?;
        self.saved = true;
        Ok(())
    }

    fn draw_pixel<C: Color>(
        &mut self,
        point: BackendCoord,
        color: &C,
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
            .set("opacity", make_svg_opacity(style.as_color()))
            .set("stroke", make_svg_color(style.as_color()));
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
                .set("opacity", make_svg_opacity(style.as_color()))
                .set("stroke", make_svg_color(style.as_color()))
                .set("fill", "none");
        } else {
            node = node
                .set("opacity", make_svg_opacity(style.as_color()))
                .set("fill", make_svg_color(style.as_color()))
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
            .set("opacity", make_svg_opacity(style.as_color()))
            .set("stroke", make_svg_color(style.as_color()))
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
                .set("opacity", make_svg_opacity(style.as_color()))
                .set("stroke", make_svg_color(style.as_color()))
                .set("fill", "none");
        } else {
            node = node
                .set("opacity", make_svg_opacity(style.as_color()))
                .set("fill", make_svg_color(style.as_color()))
                .set("stroke", "none");
        }

        self.update_document(|d| d.add(node));
        Ok(())
    }
    fn draw_text<'b, C: Color>(
        &mut self,
        text: &str,
        font: &FontDesc<'b>,
        pos: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if color.alpha() == 0.0 {
            return Ok(());
        }
        let context = svg::node::Text::new(text);
        let layout = font.layout_box(text).map_err(DrawingErrorKind::FontError)?;

        let trans = font.get_transform();
        let offset = trans.offset(layout);
        let x0 = pos.0 + offset.0;
        let y0 = pos.1 + offset.1;

        let node = Text::new()
            .set("x", x0)
            .set("y", y0 - (layout.0).1)
            .set("font-famliy", font.get_name())
            .set("font-size", font.get_size())
            .set("opacity", make_svg_opacity(color))
            .set("fill", make_svg_color(color));

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
}

impl Drop for SVGBackend<'_> {
    fn drop(&mut self) {
        if !self.saved {
            self.present().expect("Unable to save the SVG image");
        }
    }
}
