use std::collections::{hash_map::IntoIter as HashMapIter, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::AddAssign;

use crate::chart::ChartContext;
use crate::coord::{DescreteRanged, Ranged, RangedCoord};
use crate::drawing::DrawingBackend;
use crate::element::Rectangle;
use crate::style::{Color, ShapeStyle, GREEN};

pub trait HistogramType {}
pub struct Vertical;
pub struct Horizental;

impl HistogramType for Vertical {}
impl HistogramType for Horizental {}

/// The series that aggregate data into a histogram
pub struct Histogram<BR, A, Tag = Vertical>
where
    BR: DescreteRanged,
    BR::ValueType: Eq + Hash + Default,
    A: AddAssign<A> + Default,
    Tag: HistogramType,
{
    style: ShapeStyle,
    margin: u32,
    iter: HashMapIter<BR::ValueType, A>,
    baseline: Box<dyn Fn() -> A>,
    _p: PhantomData<(BR, Tag)>,
}

impl<BR, A, Tag> Histogram<BR, A, Tag>
where
    BR: DescreteRanged,
    BR::ValueType: Eq + Hash + Default,
    A: AddAssign<A> + Default,
    Tag: HistogramType,
{
    fn empty() -> Self {
        Self {
            style: GREEN.filled(),
            margin: 5,
            iter: HashMap::new().into_iter(),
            baseline: Box::new(|| A::default()),
            _p: PhantomData,
        }
    }
    /// Set the style of the histogram
    pub fn style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Set the baseline of the histogram
    pub fn baseline(mut self, baseline: A) -> Self
    where
        A: Clone + 'static,
    {
        self.baseline = Box::new(move || baseline.clone());
        self
    }

    /// Set the margin for each bar
    pub fn margin(mut self, value: u32) -> Self {
        self.margin = value;
        self
    }

    /// Set the data iterator
    pub fn data<I: IntoIterator<Item = (BR::ValueType, A)>>(mut self, iter: I) -> Self {
        let mut buffer = HashMap::<BR::ValueType, A>::new();
        for (x, y) in iter.into_iter() {
            *buffer.entry(x).or_insert_with(Default::default) += y;
        }
        self.iter = buffer.into_iter();
        self
    }
}

impl<BR, A> Histogram<BR, A, Vertical>
where
    BR: DescreteRanged,
    BR::ValueType: Eq + Hash + Default,
    A: AddAssign<A> + Default,
{
    /// Create a new histogram series.
    ///
    /// - `iter`: The data iterator
    /// - `margin`: The margin between bars
    /// - `style`: The style of bars
    ///
    /// Returns the newly created histogram series
    pub fn new<S: Into<ShapeStyle>, I: IntoIterator<Item = (BR::ValueType, A)>>(
        iter: I,
        margin: u32,
        style: S,
    ) -> Self {
        let mut buffer = HashMap::<BR::ValueType, A>::new();
        for (x, y) in iter.into_iter() {
            *buffer.entry(x).or_insert_with(Default::default) += y;
        }
        Self {
            style: style.into(),
            margin,
            iter: buffer.into_iter(),
            baseline: Box::new(|| A::default()),
            _p: PhantomData,
        }
    }

    pub fn vertical<ACoord, DB: DrawingBackend>(
        _: &ChartContext<DB, RangedCoord<BR, ACoord>>,
    ) -> Self
    where
        ACoord: Ranged<ValueType = A>,
    {
        Self::empty()
    }
}

impl<BR, A> Histogram<BR, A, Horizental>
where
    BR: DescreteRanged,
    BR::ValueType: Eq + Hash + Default,
    A: AddAssign<A> + Default,
{
    pub fn horizental<ACoord, DB: DrawingBackend>(
        _: &ChartContext<DB, RangedCoord<ACoord, BR>>,
    ) -> Self
    where
        ACoord: Ranged<ValueType = A>,
    {
        Self::empty()
    }
}

impl<BR, A> Iterator for Histogram<BR, A, Vertical>
where
    BR: DescreteRanged,
    BR::ValueType: Eq + Hash + Default,
    A: AddAssign<A> + Default,
{
    type Item = Rectangle<(BR::ValueType, A)>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((x, y)) = self.iter.next() {
            let nx = BR::next_value(&x);
            let mut rect = Rectangle::new([(x, y), (nx, (self.baseline)())], self.style.clone());
            rect.set_margin(0, 0, self.margin, self.margin);
            return Some(rect);
        }
        None
    }
}

impl<BR, A> Iterator for Histogram<BR, A, Horizental>
where
    BR: DescreteRanged,
    BR::ValueType: Eq + Hash + Default,
    A: AddAssign<A> + Default,
{
    type Item = Rectangle<(A, BR::ValueType)>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((y, x)) = self.iter.next() {
            let ny = BR::next_value(&y);
            let mut rect = Rectangle::new([(x, y), ((self.baseline)(), ny)], self.style.clone());
            rect.set_margin(self.margin, self.margin, 0, 0);
            return Some(rect);
        }
        None
    }
}
