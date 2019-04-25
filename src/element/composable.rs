use super::*;
use std::iter::{Once,once};
use std::borrow::Borrow;
use std::ops::Add;

pub struct EmptyElement<Coord> {
    coord: Coord
}

impl <Coord> EmptyElement<Coord> {
    pub fn at(coord:Coord) -> Self {
        return Self { coord };
    }
}

impl <Coord, Other> Add<Other> for EmptyElement<Coord> 
where
    Other: Drawable,
    for <'a> &'a Other: PointCollection<'a, BackendCoord>
{
    type Output = BoxedElement<Coord, Other>;
    fn add(self, other:Other) -> Self::Output {
        return BoxedElement {
            offset: self.coord,
            inner: other,
        };
    }
}

impl <'a, Coord> PointCollection<'a, Coord> for &'a EmptyElement<Coord> {
    type Borrow = &'a Coord;
    type IntoIter = Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        return once(&self.coord);
    }
}

pub struct BoxedElement<Coord, A: Drawable> {
    inner: A,
    offset: Coord
}

impl <'b, Coord,A:Drawable> PointCollection<'b, Coord> for &'b BoxedElement<Coord, A> {
    type Borrow = &'b Coord;
    type IntoIter = Once<&'b Coord>;
    fn point_iter(self) -> Self::IntoIter {
        return once(&self.offset);
    }
}

impl <Coord,A> Drawable for BoxedElement<Coord, A>
where
    for <'a> &'a A: PointCollection<'a, BackendCoord>,
    A: Drawable,
{
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        mut pos: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x0,y0)) = pos.next() {
            self.inner.draw(self.inner.point_iter().into_iter().map(|p| {
                let p = p.borrow();
                return (p.0 + x0, p.1 + y0);
            }), backend)?;
        }
        return Ok(());
    }
}

impl <Coord, My, Yours> Add<Yours> for BoxedElement<Coord, My> 
where
    My: Drawable,
    for <'a> &'a My: PointCollection<'a, BackendCoord>,
    Yours: Drawable,
    for <'a> &'a Yours: PointCollection<'a, BackendCoord>,
{
    type Output = ComposedElement<Coord, My, Yours>;
    fn add(self, yours:Yours) -> Self::Output {
        return ComposedElement {
            offset: self.offset,
            first: self.inner,
            second: yours
        };
    }
}


pub struct ComposedElement<Coord, A, B> 
where
    A: Drawable,
    B: Drawable
{
    first: A,
    second: B,
    offset: Coord
}

impl <'b, Coord,A,B> PointCollection<'b, Coord> for &'b ComposedElement<Coord, A, B> 
where
    A: Drawable,
    B: Drawable
{
    type Borrow = &'b Coord;
    type IntoIter = Once<&'b Coord>;
    fn point_iter(self) -> Self::IntoIter {
        return once(&self.offset);
    }
}

impl <Coord,A,B> Drawable for ComposedElement<Coord, A, B>
where
    for <'a> &'a A: PointCollection<'a, BackendCoord>,
    for <'b> &'b B: PointCollection<'b, BackendCoord>,
    A: Drawable,
    B: Drawable 
{
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        mut pos: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x0,y0)) = pos.next() {
            self.first.draw(self.first.point_iter().into_iter().map(|p| {
                let p = p.borrow();
                return (p.0 + x0, p.1 + y0);
            }), backend)?;
            self.second.draw(self.second.point_iter().into_iter().map(|p| {
                let p = p.borrow();
                return (p.0 + x0, p.1 + y0);
            }), backend)?;
        }
        return Ok(());
    }
}

impl <Coord, A, B, C> Add<C> for ComposedElement<Coord, A, B> 
where
    A: Drawable,
    for <'a> &'a A: PointCollection<'a, BackendCoord>,
    B: Drawable,
    for <'a> &'a B: PointCollection<'a, BackendCoord>,
    C: Drawable,
    for <'a> &'a C: PointCollection<'a, BackendCoord>,
{
    type Output = ComposedElement<Coord, A, ComposedElement<BackendCoord, B, C>>;
    fn add(self, rhs:C) -> Self::Output {
        return ComposedElement {
            offset: self.offset,
            first: self.first,
            second: ComposedElement {
                offset: (0,0),
                first: self.second,
                second: rhs,
            },
        };
    }
}
