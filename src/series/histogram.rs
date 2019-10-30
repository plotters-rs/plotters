use std::collections::{hash_map::IntoIter as HashMapIter, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::AddAssign;

use crate::chart::ChartContext;
use crate::coord::{DiscreteRanged, Ranged, RangedCoord};
use crate::drawing::DrawingBackend;
use crate::element::Rectangle;
use crate::style::{Color, ShapeStyle, GREEN};

pub trait HistogramType {}
pub struct Vertical;
pub struct Horizontal;

impl HistogramType for Vertical {}
impl HistogramType for Horizontal {}

/// The series that aggregate data into a histogram
pub struct Histogram<'a, BR, A, Tag = Vertical>
where
    BR: DiscreteRanged,
    BR::ValueType: Eq + Hash,
    A: AddAssign<A> + Default,
    Tag: HistogramType,
{
    style: Box<dyn Fn(&BR::ValueType, &A) -> ShapeStyle + 'a>,
    margin: u32,
    iter: HashMapIter<BR::ValueType, A>,
    baseline: Box<dyn Fn(BR::ValueType) -> A + 'a>,
    _p: PhantomData<(BR, Tag)>,
}

impl<'a, BR, A, Tag> Histogram<'a, BR, A, Tag>
where
    BR: DiscreteRanged,
    BR::ValueType: Eq + Hash,
    A: AddAssign<A> + Default + 'a,
    Tag: HistogramType,
{
    fn empty() -> Self {
        Self {
            style: Box::new(|_, _| GREEN.filled()),
            margin: 5,
            iter: HashMap::new().into_iter(),
            baseline: Box::new(|_| A::default()),
            _p: PhantomData,
        }
    }
    /// Set the style of the histogram
    pub fn style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        let style = style.into();
        self.style = Box::new(move |_, _| style.clone());
        self
    }

    /// Set the style of histogram using a lambda function
    pub fn style_func(
        mut self,
        style_func: impl Fn(&BR::ValueType, &A) -> ShapeStyle + 'a,
    ) -> Self {
        self.style = Box::new(style_func);
        self
    }

    /// Set the baseline of the histogram
    pub fn baseline(mut self, baseline: A) -> Self
    where
        A: Clone,
    {
        self.baseline = Box::new(move |_| baseline.clone());
        self
    }

    /// Set a function that defines variant baseline
    pub fn baseline_func(mut self, func: impl Fn(BR::ValueType) -> A + 'a) -> Self {
        self.baseline = Box::new(func);
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

impl<'a, BR, A> Histogram<'a, BR, A, Vertical>
where
    BR: DiscreteRanged,
    BR::ValueType: Eq + Hash,
    A: AddAssign<A> + Default + 'a,
{
    /// Create a new histogram series.
    ///
    /// - `iter`: The data iterator
    /// - `margin`: The margin between bars
    /// - `style`: The style of bars
    ///
    /// Returns the newly created histogram series
    #[allow(clippy::redundant_closure)]
    pub fn new<S: Into<ShapeStyle>, I: IntoIterator<Item = (BR::ValueType, A)>>(
        iter: I,
        margin: u32,
        style: S,
    ) -> Self {
        let mut buffer = HashMap::<BR::ValueType, A>::new();
        for (x, y) in iter.into_iter() {
            *buffer.entry(x).or_insert_with(Default::default) += y;
        }
        let style = style.into();
        Self {
            style: Box::new(move |_, _| style.clone()),
            margin,
            iter: buffer.into_iter(),
            baseline: Box::new(|_| A::default()),
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

impl<'a, BR, A> Histogram<'a, BR, A, Horizontal>
where
    BR: DiscreteRanged,
    BR::ValueType: Eq + Hash,
    A: AddAssign<A> + Default + 'a,
{
    pub fn horizontal<ACoord, DB: DrawingBackend>(
        _: &ChartContext<DB, RangedCoord<ACoord, BR>>,
    ) -> Self
    where
        ACoord: Ranged<ValueType = A>,
    {
        Self::empty()
    }
}

impl<'a, BR, A> Iterator for Histogram<'a, BR, A, Vertical>
where
    BR: DiscreteRanged,
    BR::ValueType: Eq + Hash,
    A: AddAssign<A> + Default,
{
    type Item = Rectangle<(BR::ValueType, A)>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((x, y)) = self.iter.next() {
            let nx = BR::next_value(&x);
            let base = (self.baseline)(BR::previous_value(&nx));
            let style = (self.style)(&x, &y);
            let mut rect = Rectangle::new([(x, y), (nx, base)], style);
            rect.set_margin(0, 0, self.margin, self.margin);
            return Some(rect);
        }
        None
    }
}

impl<'a, BR, A> Iterator for Histogram<'a, BR, A, Horizontal>
where
    BR: DiscreteRanged,
    BR::ValueType: Eq + Hash,
    A: AddAssign<A> + Default,
{
    type Item = Rectangle<(A, BR::ValueType)>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((y, x)) = self.iter.next() {
            let ny = BR::next_value(&y);
            // With this trick we can avoid the clone trait bound
            let base = (self.baseline)(BR::previous_value(&ny));
            let style = (self.style)(&y, &x);
            let mut rect = Rectangle::new([(x, y), (base, ny)], style);
            rect.set_margin(self.margin, self.margin, 0, 0);
            return Some(rect);
        }
        None
    }
}
