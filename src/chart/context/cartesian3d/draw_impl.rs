use plotters_backend::DrawingBackend;

use crate::chart::ChartContext;
use crate::coord::{
    cartesian::Cartesian3d,
    ranged1d::{KeyPointHint, Ranged},
    CoordTranslate,
};
use crate::drawing::DrawingAreaErrorKind;
use crate::element::{EmptyElement, PathElement, Polygon, Text};
use crate::style::{
    text_anchor::{HPos, Pos, VPos},
    ShapeStyle, TextStyle,
};

use super::Coord3D;

pub(crate) struct KeyPoints3d<X: Ranged, Y: Ranged, Z: Ranged> {
    pub(crate) x_points: Vec<X::ValueType>,
    pub(crate) y_points: Vec<Y::ValueType>,
    pub(crate) z_points: Vec<Z::ValueType>,
}

impl<'a, DB, X: Ranged, Y: Ranged, Z: Ranged> ChartContext<'a, DB, Cartesian3d<X, Y, Z>>
where
    DB: DrawingBackend,
    X::ValueType: Clone,
    Y::ValueType: Clone,
    Z::ValueType: Clone,
{
    pub(crate) fn get_key_points<XH: KeyPointHint, YH: KeyPointHint, ZH: KeyPointHint>(
        &self,
        x_hint: XH,
        y_hint: YH,
        z_hint: ZH,
    ) -> KeyPoints3d<X, Y, Z> {
        let coord = self.plotting_area().as_coord_spec();
        let x_points = coord.logic_x.key_points(x_hint);
        let y_points = coord.logic_y.key_points(y_hint);
        let z_points = coord.logic_z.key_points(z_hint);
        KeyPoints3d {
            x_points,
            y_points,
            z_points,
        }
    }
    pub(crate) fn draw_axis_ticks(
        &mut self,
        axis: [[Coord3D<X::ValueType, Y::ValueType, Z::ValueType>; 3]; 2],
        labels: &[(
            [Coord3D<X::ValueType, Y::ValueType, Z::ValueType>; 3],
            String,
        )],
        tick_size: i32,
        style: ShapeStyle,
        font: TextStyle,
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
        let coord = self.plotting_area().as_coord_spec();
        let begin = coord.translate(&Coord3D::build_coord([
            &axis[0][0],
            &axis[0][1],
            &axis[0][2],
        ]));
        let end = coord.translate(&Coord3D::build_coord([
            &axis[1][0],
            &axis[1][1],
            &axis[1][2],
        ]));
        let axis_dir = (end.0 - begin.0, end.1 - begin.1);
        let (x_range, y_range) = self.plotting_area().get_pixel_range();
        let x_mid = (x_range.start + x_range.end) / 2;
        let y_mid = (y_range.start + y_range.end) / 2;

        let x_dir = if begin.0 < x_mid {
            (-tick_size, 0)
        } else {
            (tick_size, 0)
        };

        let y_dir = if begin.1 < y_mid {
            (0, -tick_size)
        } else {
            (0, tick_size)
        };

        let x_score = (x_dir.0 * axis_dir.0 + x_dir.1 * axis_dir.1).abs();
        let y_score = (y_dir.0 * axis_dir.0 + y_dir.1 * axis_dir.1).abs();

        let dir = if x_score < y_score { x_dir } else { y_dir };

        for (pos, text) in labels {
            let logic_pos = Coord3D::build_coord([&pos[0], &pos[1], &pos[2]]);
            let mut font = font.clone();
            if dir.0 < 0 {
                font.pos = Pos::new(HPos::Right, VPos::Center);
            } else if dir.0 > 0 {
                font.pos = Pos::new(HPos::Left, VPos::Center);
            };
            if dir.1 < 0 {
                font.pos = Pos::new(HPos::Center, VPos::Bottom);
            } else if dir.1 > 0 {
                font.pos = Pos::new(HPos::Center, VPos::Top);
            };
            let element = EmptyElement::at(logic_pos)
                + PathElement::new(vec![(0, 0), dir], style.clone())
                + Text::new(text.to_string(), (dir.0 * 2, dir.1 * 2), font.clone());
            self.plotting_area().draw(&element)?;
        }
        Ok(())
    }
    pub(crate) fn draw_axis(
        &mut self,
        idx: usize,
        panels: &[[[Coord3D<X::ValueType, Y::ValueType, Z::ValueType>; 3]; 2]; 3],
        style: ShapeStyle,
    ) -> Result<
        [[Coord3D<X::ValueType, Y::ValueType, Z::ValueType>; 3]; 2],
        DrawingAreaErrorKind<DB::ErrorType>,
    > {
        let coord = self.plotting_area().as_coord_spec();
        let x_range = coord.logic_x.range();
        let y_range = coord.logic_y.range();
        let z_range = coord.logic_z.range();

        let ranges: [[Coord3D<X::ValueType, Y::ValueType, Z::ValueType>; 2]; 3] = [
            [Coord3D::X(x_range.start), Coord3D::X(x_range.end)],
            [Coord3D::Y(y_range.start), Coord3D::Y(y_range.end)],
            [Coord3D::Z(z_range.start), Coord3D::Z(z_range.end)],
        ];

        let (start, end) = {
            let mut start = [&ranges[0][0], &ranges[1][0], &ranges[2][0]];
            let mut end = [&ranges[0][1], &ranges[1][1], &ranges[2][1]];

            let mut plan = vec![];

            for i in 0..3 {
                if i == idx {
                    continue;
                }
                start[i] = &panels[i][0][i];
                end[i] = &panels[i][0][i];
                for j in 0..3 {
                    if i != idx && i != j && j != idx {
                        for k in 0..2 {
                            start[j] = &panels[i][k][j];
                            end[j] = &panels[i][k][j];
                            plan.push((start, end));
                        }
                    }
                }
            }
            plan.into_iter()
                .min_by_key(|&(s, e)| {
                    let d = coord.projected_depth(s[0].get_x(), s[1].get_y(), s[2].get_z());
                    let d = d + coord.projected_depth(e[0].get_x(), e[1].get_y(), e[2].get_z());
                    let (_, y1) = coord.translate(&Coord3D::build_coord(s));
                    let (_, y2) = coord.translate(&Coord3D::build_coord(e));
                    let y = y1 + y2;
                    (d, y)
                })
                .unwrap()
        };

        self.plotting_area().draw(&PathElement::new(
            vec![Coord3D::build_coord(start), Coord3D::build_coord(end)],
            style.clone(),
        ))?;

        Ok([
            [start[0].clone(), start[1].clone(), start[2].clone()],
            [end[0].clone(), end[1].clone(), end[2].clone()],
        ])
    }
    pub(crate) fn draw_axis_panels(
        &mut self,
        bold_points: &KeyPoints3d<X, Y, Z>,
        light_points: &KeyPoints3d<X, Y, Z>,
        panel_style: ShapeStyle,
        bold_grid_style: ShapeStyle,
        light_grid_style: ShapeStyle,
    ) -> Result<
        [[[Coord3D<X::ValueType, Y::ValueType, Z::ValueType>; 3]; 2]; 3],
        DrawingAreaErrorKind<DB::ErrorType>,
    > {
        let mut r_iter = (0..3).map(|idx| {
            self.draw_axis_panel(
                idx,
                bold_points,
                light_points,
                panel_style.clone(),
                bold_grid_style.clone(),
                light_grid_style.clone(),
            )
        });
        Ok([
            r_iter.next().unwrap()?,
            r_iter.next().unwrap()?,
            r_iter.next().unwrap()?,
        ])
    }
    fn draw_axis_panel(
        &mut self,
        idx: usize,
        bold_points: &KeyPoints3d<X, Y, Z>,
        light_points: &KeyPoints3d<X, Y, Z>,
        panel_style: ShapeStyle,
        bold_grid_style: ShapeStyle,
        light_grid_style: ShapeStyle,
    ) -> Result<
        [[Coord3D<X::ValueType, Y::ValueType, Z::ValueType>; 3]; 2],
        DrawingAreaErrorKind<DB::ErrorType>,
    > {
        let coord = self.plotting_area().as_coord_spec();
        let x_range = coord.logic_x.range();
        let y_range = coord.logic_y.range();
        let z_range = coord.logic_z.range();

        let ranges: [[Coord3D<X::ValueType, Y::ValueType, Z::ValueType>; 2]; 3] = [
            [Coord3D::X(x_range.start), Coord3D::X(x_range.end)],
            [Coord3D::Y(y_range.start), Coord3D::Y(y_range.end)],
            [Coord3D::Z(z_range.start), Coord3D::Z(z_range.end)],
        ];

        let (mut panel, start, end) = {
            let a = [&ranges[0][0], &ranges[1][0], &ranges[2][0]];
            let mut b = [&ranges[0][1], &ranges[1][1], &ranges[2][1]];
            let mut c = a;
            let d = b;

            b[idx] = &ranges[idx][0];
            c[idx] = &ranges[idx][1];

            let (a, b) = if coord.projected_depth(a[0].get_x(), a[1].get_y(), a[2].get_z())
                >= coord.projected_depth(c[0].get_x(), c[1].get_y(), c[2].get_z())
            {
                (a, b)
            } else {
                (c, d)
            };

            let mut m = a.clone();
            m[(idx + 1) % 3] = b[(idx + 1) % 3];
            let mut n = a.clone();
            n[(idx + 2) % 3] = b[(idx + 2) % 3];

            (
                vec![
                    Coord3D::build_coord(a),
                    Coord3D::build_coord(m),
                    Coord3D::build_coord(b),
                    Coord3D::build_coord(n),
                ],
                a,
                b,
            )
        };
        self.plotting_area()
            .draw(&Polygon::new(panel.clone(), panel_style.clone()))?;
        panel.push(panel[0].clone());
        self.plotting_area()
            .draw(&PathElement::new(panel, bold_grid_style.clone()))?;

        for (kps, style) in vec![
            (light_points, light_grid_style),
            (bold_points, bold_grid_style),
        ]
        .into_iter()
        {
            for idx in (0..3).filter(|&i| i != idx) {
                let kps: Vec<_> = match idx {
                    0 => kps.x_points.iter().map(|x| Coord3D::X(x.clone())).collect(),
                    1 => kps.y_points.iter().map(|y| Coord3D::Y(y.clone())).collect(),
                    _ => kps.z_points.iter().map(|z| Coord3D::Z(z.clone())).collect(),
                };
                for kp in kps.iter() {
                    let mut kp_start = start;
                    let mut kp_end = end;
                    kp_start[idx] = kp;
                    kp_end[idx] = kp;
                    self.plotting_area().draw(&PathElement::new(
                        vec![Coord3D::build_coord(kp_start), Coord3D::build_coord(kp_end)],
                        style.clone(),
                    ))?;
                }
            }
        }

        Ok([
            [start[0].clone(), start[1].clone(), start[2].clone()],
            [end[0].clone(), end[1].clone(), end[2].clone()],
        ])
    }
}
