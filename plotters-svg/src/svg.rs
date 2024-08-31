/*!
The SVG image drawing backend
*/

use plotters_backend::{
    text_anchor::{HPos, VPos},
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
    FontStyle, FontTransform,
};

use std::fmt::Write as _;
use std::fs::File;
#[allow(unused_imports)]
use std::io::Cursor;
use std::io::{BufWriter, Error, Write};
use std::path::Path;

struct Rgb(u8, u8, u8);
fn make_svg_color(color: BackendColor) -> Rgb {
    Rgb(color.rgb.0, color.rgb.1, color.rgb.2)
}

enum Target<'a> {
    File(String, &'a Path),
    Buffer(&'a mut String),
    // TODO: At this point we won't make the breaking change
    // so the u8 buffer is still supported. But in 0.3, we definitely
    // should get rid of this.
    #[cfg(feature = "deprecated_items")]
    U8Buffer(String, &'a mut Vec<u8>),
}

impl Target<'_> {
    fn get_mut(&mut self) -> &mut String {
        match self {
            Target::File(ref mut buf, _) => buf,
            Target::Buffer(buf) => buf,
            #[cfg(feature = "deprecated_items")]
            Target::U8Buffer(ref mut buf, _) => buf,
        }
    }
}

#[derive(Clone)]
enum SVGTag {
    Svg,
    Circle,
    Line,
    Polygon,
    Polyline,
    Rectangle,
    Text,
    #[allow(dead_code)]
    Image,
}

impl SVGTag {
    fn to_tag_name(&self) -> &'static str {
        match self {
            SVGTag::Svg => "svg",
            SVGTag::Circle => "circle",
            SVGTag::Line => "line",
            SVGTag::Polyline => "polyline",
            SVGTag::Rectangle => "rect",
            SVGTag::Text => "text",
            SVGTag::Image => "image",
            SVGTag::Polygon => "polygon",
        }
    }
}

/// The SVG image drawing backend
pub struct SVGBackend<'a> {
    target: Target<'a>,
    size: (u32, u32),
    tag_stack: Vec<SVGTag>,
    saved: bool,
}

trait FormatEscaped {
    fn format_escaped(buf: &mut String, s: Self);
}
macro_rules! impl_format_escaped_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<$($t,)+> FormatEscaped for ($($t,)+)
        where
            $($t: FormatEscaped,)+
        {
            fn format_escaped(buf: &mut String, tup: Self) {
                $(
                    let _ = FormatEscaped::format_escaped(buf, tup.$idx);
                )+
            }
        }
    };
}
impl_format_escaped_tuple!(0 A);
impl_format_escaped_tuple!(0 A, 1 B);
impl_format_escaped_tuple!(0 A, 1 B, 2 C);
impl_format_escaped_tuple!(0 A, 1 B, 2 C, 3 D);
impl_format_escaped_tuple!(0 A, 1 B, 2 C, 3 D, 4 E);
impl_format_escaped_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
impl_format_escaped_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);

macro_rules! impl_format_escaped_plain {
    ($($t:ty),*) => {
        $(
        impl FormatEscaped for $t {
            fn format_escaped(buf: &mut String, s: Self) {
                let _ = write!(buf, "{}", s);
            }
        }
        )*
    };
}

impl_format_escaped_plain!(u32, i32, f32, f64);
impl FormatEscaped for &str {
    fn format_escaped(buf: &mut String, s: &str) {
        for c in s.chars() {
            FormatEscaped::format_escaped(buf, c);
        }
    }
}

impl FormatEscaped for Rgb {
    fn format_escaped(buf: &mut String, Rgb(r, g, b): Rgb) {
        let _ = write!(buf, "#{:02X}{:02X}{:02X}", r, g, b);
    }
}

impl<T: FormatEscaped> FormatEscaped for Option<T> {
    fn format_escaped(buf: &mut String, opt: Option<T>) {
        match opt {
            None => {
                let _ = FormatEscaped::format_escaped(buf, "none");
            }
            Some(x) => {
                let _ = FormatEscaped::format_escaped(buf, x);
            }
        }
    }
}

impl FormatEscaped for char {
    fn format_escaped(buf: &mut String, c: char) {
        match c {
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            '&' => buf.push_str("&amp;"),
            '"' => buf.push_str("&quot;"),
            '\'' => buf.push_str("&apos;"),
            other => buf.push(other),
        };
    }
}

struct FormatEscapedIter<I>(I);
impl<I: IntoIterator<Item: FormatEscaped>> FormatEscaped for FormatEscapedIter<I> {
    fn format_escaped(buf: &mut String, iter: FormatEscapedIter<I>) {
        let iter = iter.0.into_iter();
        for item in iter {
            FormatEscaped::format_escaped(buf, item);
        }
    }
}

enum Value {}
enum Init {}
struct AttrWriter<'a, State> {
    buf: &'a mut String,
    tag: SVGTag,
    tag_stack: &'a mut Vec<SVGTag>,
    state: std::marker::PhantomData<State>,
}

/// Used for opening a tag and then optionally writing some attributes. The expected workflow is
/// to call [open_tag](AttrWriter::open_tag), then zero or more times calling
/// [write_key](AttrWriter::write_key) followed optionally by `[write_value](AttrWriter::write_value)`
/// and finally calling one of [close](AttrWriter::close) (to close a self-closing tag)
/// or [finish_without_closing](AttrWriter::finish_without_closing) (to schedule writing
//  writing the closing tag for later).
impl<'a> AttrWriter<'a, Init> {
    fn open_tag(buf: &'a mut String, tag: SVGTag, tag_stack: &'a mut Vec<SVGTag>) -> Self {
        buf.push('<');
        buf.push_str(tag.to_tag_name());
        AttrWriter {
            buf,
            tag,
            tag_stack,
            state: Default::default(),
        }
    }

    fn write_key<'s>(&'s mut self, key: &str) -> AttrWriter<'s, Value> {
        self.buf.push(' ');
        self.buf.push_str(key);
        AttrWriter {
            buf: self.buf,
            tag: self.tag.clone(),
            tag_stack: &mut self.tag_stack,
            state: Default::default(),
        }
    }

    fn close(self) {
        self.buf.push_str("/>\n");
    }

    fn finish_without_closing(self) {
        self.tag_stack.push(self.tag);
        self.buf.push_str(">\n");
    }
}

impl<'a> AttrWriter<'a, Value> {
    fn write_value(self, value: impl FormatEscaped) {
        self.buf.push_str("=\"");
        FormatEscaped::format_escaped(self.buf, value);
        self.buf.push('"');
    }
}

impl<'a> SVGBackend<'a> {
    fn escape_and_push(buf: &mut String, value: &str) {
        value.chars().for_each(|c| match c {
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            '&' => buf.push_str("&amp;"),
            '"' => buf.push_str("&quot;"),
            '\'' => buf.push_str("&apos;"),
            other => buf.push(other),
        });
    }

    fn close_tag(&mut self) -> bool {
        if let Some(tag) = self.tag_stack.pop() {
            let buf = self.target.get_mut();
            buf.push_str("</");
            buf.push_str(tag.to_tag_name());
            buf.push_str(">\n");
            return true;
        }
        false
    }

    /// Opens a tag and provides facilities for writing attrs and closing the tag
    fn open_tag<'s>(&'s mut self, tag: SVGTag) -> AttrWriter<'s, Init> {
        AttrWriter::open_tag(self.target.get_mut(), tag, &mut self.tag_stack)
    }

    fn init_svg_file(&mut self, size: (u32, u32)) {
        let mut attrwriter = self.open_tag(SVGTag::Svg);
        attrwriter.write_key("width").write_value(size.0);
        attrwriter.write_key("height").write_value(size.1);
        attrwriter
            .write_key("viewBox")
            .write_value(("0 0 ", size.0, ' ', size.1));
        attrwriter
            .write_key("xmlns")
            .write_value("http://www.w3.org/2000/svg");
        attrwriter.finish_without_closing();
    }

    /// Create a new SVG drawing backend
    pub fn new<T: AsRef<Path> + ?Sized>(path: &'a T, size: (u32, u32)) -> Self {
        let mut ret = Self {
            target: Target::File(String::default(), path.as_ref()),
            size,
            tag_stack: vec![],
            saved: false,
        };

        ret.init_svg_file(size);
        ret
    }

    /// Create a new SVG drawing backend and store the document into a u8 vector
    #[cfg(feature = "deprecated_items")]
    #[deprecated(
        note = "This will be replaced by `with_string`, consider use `with_string` to avoid breaking change in the future"
    )]
    pub fn with_buffer(buf: &'a mut Vec<u8>, size: (u32, u32)) -> Self {
        let mut ret = Self {
            target: Target::U8Buffer(String::default(), buf),
            size,
            tag_stack: vec![],
            saved: false,
        };

        ret.init_svg_file(size);

        ret
    }

    /// Create a new SVG drawing backend and store the document into a String buffer
    pub fn with_string(buf: &'a mut String, size: (u32, u32)) -> Self {
        let mut ret = Self {
            target: Target::Buffer(buf),
            size,
            tag_stack: vec![],
            saved: false,
        };

        ret.init_svg_file(size);

        ret
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
            while self.close_tag() {}
            match self.target {
                Target::File(ref buf, path) => {
                    let outfile = File::create(path).map_err(DrawingErrorKind::DrawingError)?;
                    let mut outfile = BufWriter::new(outfile);
                    outfile
                        .write_all(buf.as_ref())
                        .map_err(DrawingErrorKind::DrawingError)?;
                }
                Target::Buffer(_) => {}
                #[cfg(feature = "deprecated_items")]
                Target::U8Buffer(ref actual, ref mut target) => {
                    target.clear();
                    target.extend_from_slice(actual.as_bytes());
                }
            }
            self.saved = true;
        }
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Error>> {
        if color.alpha == 0.0 {
            return Ok(());
        }
        let mut attrwriter = self.open_tag(SVGTag::Rectangle);
        attrwriter.write_key("x").write_value(point.0);
        attrwriter.write_key("y").write_value(point.1);
        attrwriter.write_key("width").write_value("1");
        attrwriter.write_key("height").write_value("1");
        attrwriter.write_key("stroke").write_value("none");
        attrwriter
            .write_key("opacity")
            .write_value(make_svg_color(color));
        attrwriter.write_key("fill").write_value(color.alpha);
        attrwriter.close();
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        let mut attrwriter = self.open_tag(SVGTag::Line);
        attrwriter
            .write_key("opacity")
            .write_value(style.color().alpha);
        attrwriter
            .write_key("stroke")
            .write_value(make_svg_color(style.color()));
        attrwriter
            .write_key("stroke-width")
            .write_value(style.stroke_width());
        attrwriter.write_key("x1").write_value(from.0);
        attrwriter.write_key("y1").write_value(from.1);
        attrwriter.write_key("x2").write_value(to.0);
        attrwriter.write_key("y2").write_value(to.1);
        attrwriter.close();
        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }

        let color = make_svg_color(style.color());
        let (fill, stroke) = if !fill {
            (None, Some(color))
        } else {
            (Some(color), None)
        };

        let mut attrwriter = self.open_tag(SVGTag::Rectangle);
        attrwriter.write_key("x").write_value(upper_left.0);
        attrwriter.write_key("y").write_value(upper_left.1);
        attrwriter
            .write_key("width")
            .write_value(bottom_right.0 - upper_left.0);
        attrwriter
            .write_key("height")
            .write_value(bottom_right.1 - upper_left.1);
        attrwriter
            .write_key("opacity")
            .write_value(style.color().alpha);
        attrwriter.write_key("fill").write_value(fill);
        attrwriter.write_key("stroke").write_value(stroke);
        attrwriter.close();
        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        let mut attrwriter = self.open_tag(SVGTag::Polyline);
        attrwriter.write_key("fill").write_value("none");
        attrwriter
            .write_key("opacity")
            .write_value(style.color().alpha);
        attrwriter
            .write_key("stroke")
            .write_value(make_svg_color(style.color()));
        attrwriter
            .write_key("stroke-width")
            .write_value(style.stroke_width());
        attrwriter
            .write_key("points")
            .write_value(FormatEscapedIter(path.into_iter().map(|c| (c.0, ',', c.1))));
        attrwriter.close();
        Ok(())
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        let mut attrwriter = self.open_tag(SVGTag::Polygon);
        attrwriter
            .write_key("opacity")
            .write_value(style.color().alpha);
        attrwriter
            .write_key("fill")
            .write_value(make_svg_color(style.color()));
        attrwriter
            .write_key("points")
            .write_value(FormatEscapedIter(path.into_iter()));
        attrwriter.close();

        Ok(())
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        let color = make_svg_color(style.color());
        let (stroke, fill) = if !fill {
            (Some(color), None)
        } else {
            (None, Some(color))
        };
        let mut attrwriter = self.open_tag(SVGTag::Circle);
        attrwriter.write_key("cx").write_value(center.0);
        attrwriter.write_key("cy").write_value(center.1);
        attrwriter.write_key("r").write_value(radius);
        attrwriter
            .write_key("opacity")
            .write_value(style.color().alpha);
        attrwriter.write_key("fill").write_value(fill);
        attrwriter.write_key("stroke").write_value(stroke);
        attrwriter
            .write_key("stroke-width")
            .write_value(style.stroke_width());
        attrwriter.close();
        Ok(())
    }

    fn draw_text<S: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &S,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let color = style.color();
        if color.alpha == 0.0 {
            return Ok(());
        }

        let (x0, y0) = pos;
        let text_anchor = match style.anchor().h_pos {
            HPos::Left => "start",
            HPos::Right => "end",
            HPos::Center => "middle",
        };

        let dy = match style.anchor().v_pos {
            VPos::Top => "0.76em",
            VPos::Center => "0.5ex",
            VPos::Bottom => "-0.5ex",
        };

        #[cfg(feature = "debug")]
        {
            let ((fx0, fy0), (fx1, fy1)) =
                font.layout_box(text).map_err(DrawingErrorKind::FontError)?;
            let x0 = match style.anchor().h_pos {
                HPos::Left => x0,
                HPos::Center => x0 - fx1 / 2 + fx0 / 2,
                HPos::Right => x0 - fx1 + fx0,
            };
            let y0 = match style.anchor().v_pos {
                VPos::Top => y0,
                VPos::Center => y0 - fy1 / 2 + fy0 / 2,
                VPos::Bottom => y0 - fy1 + fy0,
            };
            self.draw_rect(
                (x0, y0),
                (x0 + fx1 - fx0, y0 + fy1 - fy0),
                &crate::prelude::RED,
                false,
            )
            .unwrap();
            self.draw_circle((x0, y0), 2, &crate::prelude::RED, false)
                .unwrap();
        }

        let mut attrwriter = self.open_tag(SVGTag::Text);
        attrwriter.write_key("x").write_value(x0);
        attrwriter.write_key("y").write_value(y0);
        attrwriter.write_key("dy").write_value(dy);
        attrwriter.write_key("text-anchor").write_value(text_anchor);
        attrwriter
            .write_key("font-family")
            .write_value(style.family().as_str());
        attrwriter
            .write_key("font-size")
            .write_value(style.size() / 1.24);
        attrwriter.write_key("opacity").write_value(color.alpha);
        attrwriter
            .write_key("fill")
            .write_value(make_svg_color(color));

        match style.style() {
            FontStyle::Normal => {}
            FontStyle::Bold => {
                attrwriter.write_key("font-weight").write_value("bold");
            }
            other_style => {
                attrwriter
                    .write_key("font-style")
                    .write_value(other_style.as_str());
            }
        };

        let trans = style.transform();
        match trans {
            FontTransform::Rotate90 => {
                attrwriter
                    .write_key("transform")
                    .write_value(("rotate(90,", x0, ',', y0, ')'));
            }
            FontTransform::Rotate180 => {
                attrwriter
                    .write_key("transform")
                    .write_value(("rotate(180,", x0, ',', y0, ')'));
            }
            FontTransform::Rotate270 => {
                attrwriter
                    .write_key("transform")
                    .write_value(("rotate(270,", x0, ',', y0, ')'));
            }
            _ => {}
        }
        attrwriter.finish_without_closing();

        Self::escape_and_push(self.target.get_mut(), text);
        self.target.get_mut().push('\n');

        self.close_tag();

        Ok(())
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    fn blit_bitmap<'b>(
        &mut self,
        pos: BackendCoord,
        (w, h): (u32, u32),
        src: &'b [u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        use image::codecs::png::PngEncoder;
        use image::ImageEncoder;

        let mut data = vec![0; 0];

        {
            let cursor = Cursor::new(&mut data);

            let encoder = PngEncoder::new(cursor);

            let color = image::ColorType::Rgb8;

            encoder.write_image(src, w, h, color).map_err(|e| {
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

        let mut attrwriter = self.open_tag(SVGTag::Image);
        attrwriter.write_key("x").write_value(pos.0);
        attrwriter.write_key("y").write_value(pos.1);
        attrwriter.write_key("width").write_value(w);
        attrwriter.write_key("height").write_value(h);
        attrwriter.write_key("href").write_value(buf.as_str());
        attrwriter.close();

        Ok(())
    }
}

impl Drop for SVGBackend<'_> {
    fn drop(&mut self) {
        if !self.saved {
            // drop should not panic, so we ignore a failed present
            let _ = self.present();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use plotters::element::Circle;
    use plotters::prelude::{
        ChartBuilder, Color, IntoDrawingArea, IntoFont, SeriesLabelPosition, TextStyle, BLACK,
        BLUE, RED, WHITE,
    };
    use plotters::style::text_anchor::{HPos, Pos, VPos};
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
        let mut content: String = Default::default();
        {
            let root = SVGBackend::with_string(&mut content, (500, 500)).into_drawing_area();

            let mut chart = ChartBuilder::on(&root)
                .caption("This is a test", ("sans-serif", 20u32))
                .set_all_label_area_size(40u32)
                .build_cartesian_2d(0..10, 0..10)
                .unwrap();

            chart
                .configure_mesh()
                .set_all_tick_mark_size(tick_size)
                .draw()
                .unwrap();
        }

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
        let mut content: String = Default::default();
        {
            let mut root = SVGBackend::with_string(&mut content, (500, 500));

            let style = TextStyle::from(("sans-serif", 20).into_font())
                .pos(Pos::new(HPos::Right, VPos::Top));
            root.draw_text("right-align", &style, (150, 50)).unwrap();

            let style = style.pos(Pos::new(HPos::Center, VPos::Top));
            root.draw_text("center-align", &style, (150, 150)).unwrap();

            let style = style.pos(Pos::new(HPos::Left, VPos::Top));
            root.draw_text("left-align", &style, (150, 200)).unwrap();
        }

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
        let mut content: String = Default::default();
        {
            let root = SVGBackend::with_string(&mut content, (1500, 800)).into_drawing_area();
            let root = root
                .titled("Image Title", ("sans-serif", 60).into_font())
                .unwrap();

            let mut chart = ChartBuilder::on(&root)
                .caption("All anchor point positions", ("sans-serif", 20u32))
                .set_all_label_area_size(40u32)
                .build_cartesian_2d(0..100i32, 0..50i32)
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
        }

        checked_save_file("test_text_draw", &content);

        assert_eq!(content.matches("dog").count(), 36);
        assert_eq!(content.matches("dood").count(), 36);
        assert_eq!(content.matches("goog").count(), 36);
    }

    #[test]
    fn test_text_clipping() {
        let mut content: String = Default::default();
        {
            let (width, height) = (500_i32, 500_i32);
            let root = SVGBackend::with_string(&mut content, (width as u32, height as u32))
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

        checked_save_file("test_text_clipping", &content);
    }

    #[test]
    fn test_series_labels() {
        let mut content = String::default();
        {
            let (width, height) = (500, 500);
            let root = SVGBackend::with_string(&mut content, (width, height)).into_drawing_area();

            let mut chart = ChartBuilder::on(&root)
                .caption("All series label positions", ("sans-serif", 20u32))
                .set_all_label_area_size(40u32)
                .build_cartesian_2d(0..50i32, 0..50i32)
                .unwrap();

            chart
                .configure_mesh()
                .disable_x_mesh()
                .disable_y_mesh()
                .draw()
                .unwrap();

            chart
                .draw_series(std::iter::once(Circle::new((5, 15), 5u32, &RED)))
                .expect("Drawing error")
                .label("Series 1")
                .legend(|(x, y)| Circle::new((x, y), 3u32, RED.filled()));

            chart
                .draw_series(std::iter::once(Circle::new((5, 15), 10u32, &BLUE)))
                .expect("Drawing error")
                .label("Series 2")
                .legend(|(x, y)| Circle::new((x, y), 3u32, BLUE.filled()));

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

        checked_save_file("test_series_labels", &content);
    }

    #[test]
    fn test_draw_pixel_alphas() {
        let mut content = String::default();
        {
            let (width, height) = (100_i32, 100_i32);
            let root = SVGBackend::with_string(&mut content, (width as u32, height as u32))
                .into_drawing_area();
            root.fill(&WHITE).unwrap();

            for i in -20..20 {
                let alpha = i as f64 * 0.1;
                root.draw_pixel((50 + i, 50 + i), &BLACK.mix(alpha))
                    .unwrap();
            }
        }

        checked_save_file("test_draw_pixel_alphas", &content);
    }
}
