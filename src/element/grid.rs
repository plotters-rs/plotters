use super::Element;
use crate::color::Color;
use crate::drawing::DrawingBackend;

pub enum GridDirection {
    Horizontal,
    Vertical,
}

impl GridDirection {
    fn make_coord(&self, point:u32, (s0, s1):(u32,u32)) -> ((u32,u32), (u32,u32)) {
        return match self {
            GridDirection::Horizontal => ((s0, point), (s1, point)),
            GridDirection::Vertical   => ((point, s0), (s1, point))
        };
    }

    fn extract(&self, (x,y):(u32,u32)) -> u32 {
        return match self {
            GridDirection::Horizontal => y,
            GridDirection::Vertical   => x,
        }
    }
}

pub struct Grid<'a, ColorType:Color, Coord> {
    color: ColorType,
    key_points: Vec<(u32,u32)>,
    coords: Vec<&'a Coord>,
}

pub struct GridLineIter<'a, Coord> (std::slice::Iter<'a, (u32,u32)>, std::slice::Iter<'a, &'a Coord>);
impl <'a, Coord> Iterator for GridLineIter<'a, Coord> {
    type Item = ((u32,u32), (u32,u32), &'a Coord);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((x0,y0)) = self.0.next() {
            if let Some((x1,y1)) = self.0.next() {
                if let Some(raw) = self.1.next() {
                    return Some(((*x0,*y0), (*x1,*y1), raw));
                }
            }
        }
        return None;
    }
}

impl <'a, ColorType:Color, Coord> Grid<'a, ColorType, Coord> {
    pub fn new<CMF:Fn(&Coord)->u32>(direction: GridDirection, key_points: &'a [Coord], span:(u32, u32), cmf: CMF, color:ColorType, min_dist:u32) -> Self {
        let mut kp_buffer = vec![];
        for coord in key_points {
            let d_coord = cmf(coord);
            kp_buffer.push((direction.make_coord(d_coord, span), coord));
        }

        kp_buffer.sort_by_key(|a| direction.extract((a.0).0));

        let mut key_points = vec![];
        let mut coords = vec![];
        let mut last = None;

        for ((begin, end), raw_coord) in kp_buffer {
            if last.map_or(true, |last| last + min_dist < direction.extract(begin)) {
                key_points.push(begin);
                key_points.push(end);
                coords.push(raw_coord);
                last = Some(direction.extract(begin));
            }
        }

        return Self {
            color,
            key_points,
            coords,
        };
    }

    pub fn get_grid_lines(&'a self) -> impl Iterator<Item=((u32,u32), (u32,u32), &'a Coord)> {
        return GridLineIter(self.key_points.iter(), self.coords.iter());
    }
}

impl <'a, 'b:'a, ColorType:Color, Coord> Element<'a, (u32,u32)> for Grid<'a, ColorType, Coord> {
    type Points = &'a [(u32,u32)];

    fn points(&'a self) -> &'a [(u32,u32)] {
        return &self.key_points[..]
    }

    fn draw<DC:DrawingBackend, I:Iterator<Item=(u32,u32)>>(&self, mut pos:I, dc: &mut DC) -> Result<(), DC::ErrorType> {
        loop {
            if let Some((x0,y0)) = pos.next() {
                if let Some((x1,y1)) = pos.next() {
                    dc.draw_line((x0 as i32,y0 as i32), (x1 as i32, y1 as i32), &self.color)?;
                    continue;
                }
            }
            break;
        }

        return Ok(());
    }
}
