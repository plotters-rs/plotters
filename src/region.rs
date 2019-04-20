use std::rc::Rc;
use crate::drawing::DrawingBackend;
use std::marker::PhantomData;
use std::cell::RefCell;
use crate::element::Element;

pub struct Region{
    x0:u32,
    y0:u32,
    x1:u32,
    y1:u32,
}

pub trait CoordTrans {
    type CoordType;
    fn translate(&self, from:&Self::CoordType) -> (u32,u32);
}

pub struct Shift(u32,u32);

impl CoordTrans for Shift {
    type CoordType = (u32, u32);
    fn translate(&self, (x,y):&(u32,u32)) -> (u32,u32) {
        return (x + self.0, y + self.1);
    }
}

pub struct TranslateFunction<T, F:Fn(&T)->(u32,u32)>(F, PhantomData<(T)>);

impl <T, F:Fn(&T)->(u32,u32)> From<F> for TranslateFunction<T,F> {
    fn from(f:F) -> Self {
        return TranslateFunction(f, PhantomData);
    }
}

impl <T, F:Fn(&T)->(u32,u32)> CoordTrans for TranslateFunction<T,F> {
    type CoordType = T;
    fn translate(&self, coord:&T) -> (u32,u32) {
        return (self.0)(coord);
    }
}

pub struct DrawingRegion<DC:DrawingBackend, CT:CoordTrans> {
    region: Region,
    translate: CT,
    ctx: Rc<RefCell<DC>>,
}

impl <DC:DrawingBackend> From<DC> for DrawingRegion<DC, Shift> {
    fn from(dc:DC) -> Self {
        let (x1,y1) = dc.get_size();
        return Self {
            region: Region {x0:0,y0:0,x1,y1},
            translate: Shift(0,0),
            ctx: Rc::new(RefCell::new(dc)),
        };
    }
}

impl <DC:DrawingBackend, CT:CoordTrans> DrawingRegion<DC, CT> {
    pub fn draw<'a, E:Element<'a, CT::CoordType>>(&self, element:&'a E) -> Result<(), DC::ErrorType> 
        where CT::CoordType : 'a
    {

        if let Ok(mut dc) = self.ctx.try_borrow_mut() {
            let translated = element.points().into_iter().map(|p| self.translate.translate(p));
            return element.draw(translated, &mut *dc);
        }

        //TODO: Handle error
        Ok(())
    }

    pub fn close(&self) -> Result<(), DC::ErrorType> {
        if let Ok(mut dc) = self.ctx.try_borrow_mut() {
            return dc.close();
        }
        //TODO: Handle error
        Ok(())
    }
}

fn compute_splits(x0:u32, x1:u32, n:u32) -> Vec<u32> {
    let mut xs = vec![];
    let mut d = (x1 - x0) % n;
    let mut x = x0;

    while x <= x1 {
        xs.push(x);
        
        x += (x1 - x0) / n;
        if d > 0 {
            d -=1;
            x += 1;
        }

    }

    return xs;
}

pub trait Splitable<DC:DrawingBackend> {
    fn get_region(&self) -> &Region;
    fn get_dc(&self) -> Rc<RefCell<DC>>;
    fn split(&self, (row, col):(u32,u32)) -> Vec<DrawingRegion<DC,Shift>> {
        let mut ret = vec![];

        let dim = self.get_region();
        let xs = compute_splits(dim.x0, dim.x1, col);
        let ys = compute_splits(dim.y0, dim.y1, row);

        for (y0, y1) in ys.iter().zip(ys.iter().skip(1)) {
            for (x0, x1) in xs.iter().zip(xs.iter().skip(1)) {
                ret.push(DrawingRegion {
                    region: Region{x0: *x0, x1: *x1, y0: *y0, y1: *y1},
                    translate: Shift(*x0,*y0),
                    ctx: self.get_dc(),
                });
            }
        }

        return ret;
    }
}

impl <DC:DrawingBackend> Splitable<DC> for DrawingRegion<DC, Shift> {

    fn get_region(&self) -> &Region {
        return &self.region;
    }

    fn get_dc(&self) -> Rc<RefCell<DC>> {
        return self.ctx.clone();
    }
}
