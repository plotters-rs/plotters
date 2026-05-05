use super::{BackendColor, BackendCoord};
use crate::{math_guard::checked_neg_i32, MathError};
use std::error::Error;

/// Describes font family.
/// This can be either a specific font family name, such as "arial",
/// or a general font family class, such as "serif" and "sans-serif"
#[derive(Clone, Copy)]
pub enum FontFamily<'a> {
    /// The system default serif font family
    Serif,
    /// The system default sans-serif font family
    SansSerif,
    /// The system default monospace font
    Monospace,
    /// A specific font family name
    Name(&'a str),
}

impl<'a> FontFamily<'a> {
    /// Make a CSS compatible string for the font family name.
    /// This can be used as the value of `font-family` attribute in SVG.
    pub fn as_str(&self) -> &str {
        match self {
            FontFamily::Serif => "serif",
            FontFamily::SansSerif => "sans-serif",
            FontFamily::Monospace => "monospace",
            FontFamily::Name(face) => face,
        }
    }
}

impl<'a> From<&'a str> for FontFamily<'a> {
    fn from(from: &'a str) -> FontFamily<'a> {
        match from.to_lowercase().as_str() {
            "serif" => FontFamily::Serif,
            "sans-serif" => FontFamily::SansSerif,
            "monospace" => FontFamily::Monospace,
            _ => FontFamily::Name(from),
        }
    }
}

/// Text anchor attributes are used to properly position the text.
///
/// # Examples
///
/// In the example below, the text anchor (X) position is `Pos::new(HPos::Right, VPos::Center)`.
/// ```text
///    ***** X
/// ```
/// The position is always relative to the text regardless of its rotation.
/// In the example below, the text has style
/// `style.transform(FontTransform::Rotate90).pos(Pos::new(HPos::Center, VPos::Top))`.
/// ```text
///        *
///        *
///        * X
///        *
///        *
/// ```
pub mod text_anchor {
    /// The horizontal position of the anchor point relative to the text.
    #[derive(Clone, Copy, Default)]
    pub enum HPos {
        /// Anchor point is on the left side of the text
        #[default]
        Left,
        /// Anchor point is on the right side of the text
        Right,
        /// Anchor point is in the horizontal center of the text
        Center,
    }

    /// The vertical position of the anchor point relative to the text.
    #[derive(Clone, Copy, Default)]
    pub enum VPos {
        /// Anchor point is on the top of the text
        #[default]
        Top,
        /// Anchor point is in the vertical center of the text
        Center,
        /// Anchor point is on the bottom of the text
        Bottom,
    }

    /// The text anchor position.
    #[derive(Clone, Copy, Default)]
    pub struct Pos {
        /// The horizontal position of the anchor point
        pub h_pos: HPos,
        /// The vertical position of the anchor point
        pub v_pos: VPos,
    }

    impl Pos {
        /// Create a new text anchor position.
        ///
        /// - `h_pos`: The horizontal position of the anchor point
        /// - `v_pos`: The vertical position of the anchor point
        /// - **returns** The newly created text anchor position
        ///
        /// ```rust
        /// use plotters_backend::text_anchor::{Pos, HPos, VPos};
        ///
        /// let pos = Pos::new(HPos::Left, VPos::Top);
        /// ```
        pub fn new(h_pos: HPos, v_pos: VPos) -> Self {
            Pos { h_pos, v_pos }
        }
    }
}

/// Specifying text transformations
#[derive(Clone)]
pub enum FontTransform {
    /// Nothing to transform
    None,
    /// Rotating the text 90 degree clockwise
    Rotate90,
    /// Rotating the text 180 degree clockwise
    Rotate180,
    /// Rotating the text 270 degree clockwise
    Rotate270,
}

impl FontTransform {
    /// Transform the coordinate to perform the rotation
    ///
    /// - `x`: The x coordinate in pixels before transform
    /// - `y`: The y coordinate in pixels before transform
    /// - **returns**: The coordinate after transform
    pub fn transform(&self, x: i32, y: i32) -> Result<(i32, i32), MathError> {
        Ok(match self {
            FontTransform::None => (x, y),
            FontTransform::Rotate90 => (checked_neg_i32(y)?, x),
            FontTransform::Rotate180 => (checked_neg_i32(x)?, checked_neg_i32(y)?),
            FontTransform::Rotate270 => (y, checked_neg_i32(x)?),
        })
    }
}
/// Describes the font style. Such as Italic, Oblique, etc.
#[derive(Clone, Copy)]
pub enum FontStyle {
    /// The normal style
    Normal,
    /// The oblique style
    Oblique,
    /// The italic style
    Italic,
    /// The bold style
    Bold,
}

impl FontStyle {
    /// Convert the font style into a CSS compatible string which can be used in `font-style` attribute.
    pub fn as_str(&self) -> &str {
        match self {
            FontStyle::Normal => "normal",
            FontStyle::Italic => "italic",
            FontStyle::Oblique => "oblique",
            FontStyle::Bold => "bold",
        }
    }
}

impl<'a> From<&'a str> for FontStyle {
    fn from(from: &'a str) -> FontStyle {
        match from.to_lowercase().as_str() {
            "normal" => FontStyle::Normal,
            "italic" => FontStyle::Italic,
            "oblique" => FontStyle::Oblique,
            "bold" => FontStyle::Bold,
            _ => FontStyle::Normal,
        }
    }
}

/// The trait that abstracts a style of a text.
///
/// This is used because the the backend crate have no knowledge about how
/// the text handling is implemented in plotters.
///
/// But the backend still wants to know some information about the font, for
/// the backend doesn't handles text drawing, may want to call the `draw` method which
/// is implemented by the plotters main crate. While for the backend that handles the
/// text drawing, those font information provides instructions about how the text should be
/// rendered: color, size, slant, anchor, font, etc.
///
/// This trait decouples the detailed implementation about the font and the backend code which
/// wants to perform some operation on the font.
///
pub trait BackendTextStyle {
    /// The error type of this text style implementation
    type FontError: Error + Sync + Send + 'static;

    fn color(&self) -> BackendColor {
        BackendColor {
            alpha: 1.0,
            rgb: (0, 0, 0),
        }
    }

    fn size(&self) -> f64 {
        1.0
    }

    fn transform(&self) -> FontTransform {
        FontTransform::None
    }

    fn style(&self) -> FontStyle {
        FontStyle::Normal
    }

    fn anchor(&self) -> text_anchor::Pos {
        text_anchor::Pos::default()
    }

    fn family(&self) -> FontFamily<'_>;

    #[allow(clippy::type_complexity)]
    fn layout_box(&self, text: &str) -> Result<((i32, i32), (i32, i32)), Self::FontError>;

    fn draw<E, DrawFunc: FnMut(i32, i32, BackendColor) -> Result<(), E>>(
        &self,
        text: &str,
        pos: BackendCoord,
        draw: DrawFunc,
    ) -> Result<Result<(), E>, Self::FontError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[derive(Debug)]
    struct TestFontError;

    impl fmt::Display for TestFontError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "test font error")
        }
    }

    impl Error for TestFontError {}

    struct TestTextStyle {
        family: FontFamily<'static>,
    }

    impl BackendTextStyle for TestTextStyle {
        type FontError = TestFontError;

        fn family(&self) -> FontFamily<'_> {
            self.family
        }

        fn layout_box(&self, text: &str) -> Result<((i32, i32), (i32, i32)), Self::FontError> {
            Ok(((0, 0), (text.len() as i32, 10)))
        }

        fn draw<E, DrawFunc: FnMut(i32, i32, BackendColor) -> Result<(), E>>(
            &self,
            _text: &str,
            pos: BackendCoord,
            mut draw: DrawFunc,
        ) -> Result<Result<(), E>, Self::FontError> {
            Ok(draw(pos.0, pos.1, self.color()))
        }
    }

    #[test]
    fn font_family_as_str_returns_css_family_names() {
        assert_eq!(FontFamily::Serif.as_str(), "serif");
        assert_eq!(FontFamily::SansSerif.as_str(), "sans-serif");
        assert_eq!(FontFamily::Monospace.as_str(), "monospace");
        assert_eq!(FontFamily::Name("Fira Sans").as_str(), "Fira Sans");
    }

    #[test]
    fn font_family_from_recognizes_builtin_names_case_insensitively() {
        assert!(matches!(FontFamily::from("serif"), FontFamily::Serif));
        assert!(matches!(FontFamily::from("SERIF"), FontFamily::Serif));

        assert!(matches!(
            FontFamily::from("sans-serif"),
            FontFamily::SansSerif
        ));
        assert!(matches!(
            FontFamily::from("SANS-SERIF"),
            FontFamily::SansSerif
        ));

        assert!(matches!(
            FontFamily::from("monospace"),
            FontFamily::Monospace
        ));
        assert!(matches!(
            FontFamily::from("MONOSPACE"),
            FontFamily::Monospace
        ));
    }

    #[test]
    fn font_family_from_preserves_unknown_family_name() {
        match FontFamily::from("Fira Sans") {
            FontFamily::Name(name) => assert_eq!(name, "Fira Sans"),
            _ => panic!("expected custom font family name"),
        }
    }

    #[test]
    fn text_anchor_pos_default_is_left_top() {
        let pos = text_anchor::Pos::default();

        assert!(matches!(pos.h_pos, text_anchor::HPos::Left));
        assert!(matches!(pos.v_pos, text_anchor::VPos::Top));
    }

    #[test]
    fn text_anchor_pos_new_uses_given_positions() {
        let pos = text_anchor::Pos::new(text_anchor::HPos::Right, text_anchor::VPos::Bottom);

        assert!(matches!(pos.h_pos, text_anchor::HPos::Right));
        assert!(matches!(pos.v_pos, text_anchor::VPos::Bottom));
    }

    #[test]
    fn font_transform_none_keeps_coordinates() {
        assert_eq!(FontTransform::None.transform(2, 3), Ok((2, 3)));
    }

    #[test]
    fn font_transform_rotate90_rotates_coordinates() {
        assert_eq!(FontTransform::Rotate90.transform(2, 3), Ok((-3, 2)));
    }

    #[test]
    fn font_transform_rotate180_rotates_coordinates() {
        assert_eq!(FontTransform::Rotate180.transform(2, 3), Ok((-2, -3)));
    }

    #[test]
    fn font_transform_rotate270_rotates_coordinates() {
        assert_eq!(FontTransform::Rotate270.transform(2, 3), Ok((3, -2)));
    }

    #[test]
    fn font_transform_rotate90_rejects_y_min_value() {
        assert_eq!(
            FontTransform::Rotate90.transform(2, i32::MIN),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn font_transform_rotate180_rejects_x_min_value() {
        assert_eq!(
            FontTransform::Rotate180.transform(i32::MIN, 3),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn font_transform_rotate180_rejects_y_min_value() {
        assert_eq!(
            FontTransform::Rotate180.transform(2, i32::MIN),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn font_transform_rotate270_rejects_x_min_value() {
        assert_eq!(
            FontTransform::Rotate270.transform(i32::MIN, 3),
            Err(MathError::ValueOutOfRange)
        );
    }

    #[test]
    fn font_style_as_str_returns_css_style_names() {
        assert_eq!(FontStyle::Normal.as_str(), "normal");
        assert_eq!(FontStyle::Italic.as_str(), "italic");
        assert_eq!(FontStyle::Oblique.as_str(), "oblique");
        assert_eq!(FontStyle::Bold.as_str(), "bold");
    }

    #[test]
    fn font_style_from_recognizes_known_styles_case_insensitively() {
        assert!(matches!(FontStyle::from("normal"), FontStyle::Normal));
        assert!(matches!(FontStyle::from("NORMAL"), FontStyle::Normal));

        assert!(matches!(FontStyle::from("italic"), FontStyle::Italic));
        assert!(matches!(FontStyle::from("ITALIC"), FontStyle::Italic));

        assert!(matches!(FontStyle::from("oblique"), FontStyle::Oblique));
        assert!(matches!(FontStyle::from("OBLIQUE"), FontStyle::Oblique));

        assert!(matches!(FontStyle::from("bold"), FontStyle::Bold));
        assert!(matches!(FontStyle::from("BOLD"), FontStyle::Bold));
    }

    #[test]
    fn font_style_from_unknown_value_defaults_to_normal() {
        assert!(matches!(FontStyle::from("weird-style"), FontStyle::Normal));
    }

    #[test]
    fn backend_text_style_default_color_is_opaque_black() {
        let style = TestTextStyle {
            family: FontFamily::Serif,
        };

        let color = style.color();

        assert_eq!(color.alpha, 1.0);
        assert_eq!(color.rgb, (0, 0, 0));
    }

    #[test]
    fn backend_text_style_default_size_is_one() {
        let style = TestTextStyle {
            family: FontFamily::Serif,
        };

        assert_eq!(style.size(), 1.0);
    }

    #[test]
    fn backend_text_style_default_transform_is_none() {
        let style = TestTextStyle {
            family: FontFamily::Serif,
        };

        assert_eq!(style.transform().transform(4, 5), Ok((4, 5)));
    }

    #[test]
    fn backend_text_style_default_style_is_normal() {
        let style = TestTextStyle {
            family: FontFamily::Serif,
        };

        assert!(matches!(style.style(), FontStyle::Normal));
    }

    #[test]
    fn backend_text_style_default_anchor_is_left_top() {
        let style = TestTextStyle {
            family: FontFamily::Serif,
        };

        let anchor = style.anchor();

        assert!(matches!(anchor.h_pos, text_anchor::HPos::Left));
        assert!(matches!(anchor.v_pos, text_anchor::VPos::Top));
    }

    #[test]
    fn backend_text_style_returns_family() {
        let style = TestTextStyle {
            family: FontFamily::Monospace,
        };

        assert!(matches!(style.family(), FontFamily::Monospace));
    }

    #[test]
    fn backend_text_style_layout_box_uses_text_length() {
        let style = TestTextStyle {
            family: FontFamily::Serif,
        };

        assert_eq!(style.layout_box("hello").unwrap(), ((0, 0), (5, 10)));
    }

    #[test]
    fn backend_text_style_draw_invokes_draw_callback() {
        let style = TestTextStyle {
            family: FontFamily::Serif,
        };

        let mut drawn = Vec::new();

        let result = style
            .draw("hello", (3, 4), |x, y, color| {
                drawn.push((x, y, color.rgb, color.alpha));
                Ok::<(), TestFontError>(())
            })
            .unwrap();

        assert!(result.is_ok());
        assert_eq!(drawn, vec![(3, 4, (0, 0, 0), 1.0)]);
    }

    #[test]
    fn backend_text_style_draw_propagates_callback_error_inside_ok() {
        let style = TestTextStyle {
            family: FontFamily::Serif,
        };

        let result = style
            .draw("hello", (3, 4), |_x, _y, _color| {
                Err::<(), TestFontError>(TestFontError)
            })
            .unwrap();

        assert!(result.is_err());
    }
}
