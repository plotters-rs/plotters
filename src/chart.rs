/// The abstraction of a chart
use std::marker::PhantomData;
use std::fmt::Debug;
use std::borrow::Borrow;

use crate::drawing::{DrawingArea, DrawingAreaErrorKind};
use crate::drawing::backend::DrawingBackend;
use crate::drawing::coord::{Shift, CoordTranslate, RangedCoord, Ranged, MeshLine};
use crate::style::{TextStyle, ShapeStyle, RGBColor, FontDesc, Mixable};
use crate::element::{PointCollection, Drawable};

/// The helper object to create a chart
pub struct ChartBuilder<'a, DB:DrawingBackend> {
    x_label_size: u32,
    y_label_size: u32,
    root_area: &'a DrawingArea<DB, Shift>,
    titled_area: Option<DrawingArea<DB, Shift>>,
}

impl <'a, DB:DrawingBackend> ChartBuilder<'a, DB> {
    /// Create a chart builder on the given drawing area
    pub fn on(root:&'a DrawingArea<DB, Shift>) -> Self {
        return Self {
            x_label_size: 0,
            y_label_size: 0,
            root_area: root,
            titled_area: None,
        };
    }

    /// Set the size of X label
    pub fn set_x_label_size(&mut self, size: u32) -> &mut Self {
        self.x_label_size = size;
        return self;
    }

    /// Set the size of the Y label
    pub fn set_y_label_size(&mut self, size: u32) -> &mut Self {
        self.y_label_size = size;
        return self;
    }

    /// Set the caption of the chart
    pub fn caption<S:AsRef<str>>(&mut self, caption: S, style: TextStyle) -> &mut Self {
        if self.titled_area.is_some() {
            return self;
        }

        self.titled_area = Some(self.root_area.titled(caption.as_ref(), style).expect("Unable to create caption for chart"));
        return self; 
    }

    /// Builder the chart
    pub fn build_ranged<XR:Ranged, YR:Ranged, X:Into<XR>, Y:Into<YR>>(&mut self, x_spec:X, y_spec:Y) -> ChartContext<DB, RangedCoord<XR,YR>> {
        let mut x_label_area = None;
        let mut y_label_area = None;
        
        let mut temp = None;
        std::mem::swap(&mut self.titled_area, &mut temp);

        let mut drawing_area = temp.unwrap_or_else(|| DrawingArea::clone(self.root_area));

        if self.x_label_size > 0 {
            let (_,h) = drawing_area.dim_in_pixel();
            let (upper,bottom) = drawing_area.split_vertically(h as i32 - self.x_label_size as i32);
            drawing_area = upper;
            x_label_area = Some(bottom);
        }

        if self.y_label_size > 0 {
            let (left, right) = drawing_area.split_horizentally(self.y_label_size as i32);
            drawing_area = right;
            y_label_area = Some(left);
            
            if let Some(xl) = x_label_area {
                let (_, right) = xl.split_horizentally(self.y_label_size as i32);
                x_label_area = Some(right);
            }
        }

        let mut pixel_range = drawing_area.get_pixel_range();
        pixel_range.1 = pixel_range.1.end..pixel_range.1.start;

        return ChartContext {
            x_label_area,
            y_label_area,
            drawing_area: drawing_area.apply_coord_spec(RangedCoord::new(x_spec.into(), y_spec.into(), pixel_range))
        };
    }
}

/// The context of the chart
pub struct ChartContext<DB:DrawingBackend, CT:CoordTranslate> {
    pub x_label_area: Option<DrawingArea<DB, Shift>>,
    pub y_label_area: Option<DrawingArea<DB, Shift>>,
    pub drawing_area: DrawingArea<DB, CT>,
}

pub struct MeshStyle<'a, X:Ranged, Y:Ranged, DB> 
    where DB:DrawingBackend
{
    n_x_labels: usize,
    n_y_labels: usize,
    line_style_1: Option<&'a ShapeStyle<'a>>,
    line_style_2: Option<&'a ShapeStyle<'a>>,
    label_style: Option<&'a TextStyle<'a>>,
    format_x: Box<dyn Fn(&X::ValueType)->String>,
    format_y: Box<dyn Fn(&Y::ValueType)->String>,
    target: Option<&'a mut ChartContext<DB, RangedCoord<X,Y>>>,
    _pahtom_data: PhantomData<(X,Y)>
}

impl <'a, X, Y, DB> MeshStyle<'a, X, Y, DB> 
    where X: Ranged,
          Y: Ranged,
          DB:DrawingBackend 
{
    pub fn x_labels(&mut self, value: usize) -> &mut Self {
        self.n_x_labels = value;
        return self;
    }

    pub fn y_labels(&mut self, value: usize) -> &mut Self {
        self.n_y_labels = value;
        return self;
    }

    pub fn line_style_1(&mut self, style: &'a ShapeStyle<'a>) -> &mut Self {
        self.line_style_1 = Some(style);
        return self;
    }
    
    pub fn line_style_2(&mut self, style: &'a ShapeStyle<'a>) -> &mut Self {
        self.line_style_2 = Some(style);
        return self;
    }
    
    pub fn label_style(&mut self, style: &'a TextStyle<'a>) -> &mut Self {
        self.label_style = Some(style);
        return self;
    }

    pub fn x_label_formatter(&mut self, fmt: Box<Fn(&X::ValueType)->String>) -> &mut Self {
        self.format_x = fmt;
        return self;
    }

    pub fn y_label_formatter(&mut self, fmt: Box<dyn Fn(&Y::ValueType)->String>) -> &mut Self {
        self.format_y = fmt;
        return self;
    }

    pub fn draw(&mut self) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>{
        let mut target = None;
        std::mem::swap(&mut target, &mut self.target);
        let target = target.unwrap();
        
        let default_font = FontDesc::new("ArialMT", 10.0);
        let default_color = RGBColor(0,0,0);
        let default_label_style = TextStyle{ font:&default_font, color: &default_color};
        let label_style = unsafe{ std::mem::transmute::<_,Option<&TextStyle>>(self.label_style) }.unwrap_or(&default_label_style);

        let default_mesh_color_1 = RGBColor(0,0,0).mix(0.4);
        let default_mesh_style_1 = ShapeStyle{ color: &default_mesh_color_1 };
        let mesh_style_1 = unsafe{ std::mem::transmute::<_,Option<&ShapeStyle>>(self.line_style_1) }.unwrap_or(&default_mesh_style_1);

        let default_mesh_color_2 = RGBColor(0,0,0).mix(0.2);
        let default_mesh_style_2 = ShapeStyle{ color: &default_mesh_color_2 };
        let mesh_style_2 = unsafe{ std::mem::transmute::<_,Option<&ShapeStyle>>(self.line_style_2) }.unwrap_or(&default_mesh_style_2);


        target.draw_mesh((self.n_y_labels, self.n_x_labels), mesh_style_1, label_style, |m| {
            match m {
                MeshLine::XMesh(_,_,v) => Some((self.format_x)(v)),
                MeshLine::YMesh(_,_,v) => Some((self.format_y)(v)),
            }
        })?;

        return target.draw_mesh((self.n_y_labels * 10, self.n_x_labels * 10), mesh_style_2, label_style, |_| None);
    }
}

impl <DB:DrawingBackend, XT:Debug, YT:Debug, X:Ranged<ValueType=XT>, Y:Ranged<ValueType=YT>> ChartContext<DB, RangedCoord<X,Y>> {
    pub fn configure_mesh(&mut self) -> MeshStyle<X,Y,DB> {
        return MeshStyle {
            n_x_labels: 10,
            n_y_labels: 10,
            line_style_1: None,
            line_style_2: None,
            label_style: None,
            format_x: Box::new(|x| format!("{:?}", x)),
            format_y: Box::new(|y| format!("{:?}", y)),
            target: Some(self),
            _pahtom_data: PhantomData
        };
    }
}

impl <DB:DrawingBackend, X:Ranged, Y:Ranged> ChartContext<DB, RangedCoord<X,Y>> {
    /// Draw a series
    pub fn draw_series<E,R,S>(&self, series:S) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> 
        where for <'a> &'a E:PointCollection<'a, (X::ValueType, Y::ValueType)>,
              E: Drawable,
              R: Borrow<E>,
              S: IntoIterator<Item = R>
    {
        for element in series {
            self.drawing_area.draw(element.borrow())?;
        }
        return Ok(());
    }
    /// Draw the mesh
    fn draw_mesh<FmtLabel>(&mut self, (r,c):(usize, usize), mesh_line_style: &ShapeStyle, label_style: &TextStyle, mut fmt_label:FmtLabel) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
        where FmtLabel:FnMut(&MeshLine<X,Y>) -> Option<String>
    {
        let mut x_labels = vec![];
        let mut y_labels = vec![];
        self.drawing_area.draw_mesh(|b,l| {
            match l {
                MeshLine::XMesh((x, _) ,_,_) => {
                    if let Some(label_text) = fmt_label(&l) {
                        x_labels.push((x,label_text));
                    }
                },
                MeshLine::YMesh((_,y), _, _) => {
                    if let Some(label_text) = fmt_label(&l) {
                        y_labels.push((y,label_text));
                    } 
                }
            };
            return l.draw(b, mesh_line_style);
        }, r, c)?;

        let (x0,y0) = self.drawing_area.get_base_pixel();

        if let Some(ref xl) = self.x_label_area {
            for (p,t) in x_labels {
                let (w,_) = label_style.font.box_size(&t).unwrap_or((0,0));
                xl.draw_text(&t, label_style, (p - x0 - w as i32 / 2, 15))?;
            }
        }
        
        if let Some(ref yl) = self.y_label_area {
            let (tw, _) = yl.dim_in_pixel();
            for (p,t) in y_labels {
                let (w,h) = label_style.font.box_size(&t).unwrap_or((0,0));
                yl.draw_text(&t, label_style, (tw as i32 - w as i32 - 15, p - y0 - h as i32))?;
            }
        }
        return Ok(());
    }

}
