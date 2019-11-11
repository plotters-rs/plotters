use super::numeric::RangedCoordusize;
use super::{DiscreteRanged, Ranged};
use num_traits::float::Float;
use std::ops::Range;

pub enum LinspaceRounding {
    Floor,
    Ceil,
    Round,
}

impl LinspaceRounding {
    fn get_index(&self, value: f64) -> usize {
        match self {
            Self::Floor => value.floor() as usize,
            Self::Ceil => value.ceil() as usize,
            Self::Round => value.round() as usize,
        }
    }
}

/// A linspace coordinate system, which maps a float number on a descerete coordinate
pub struct Linspace {
    start: f64,
    end: f64,
    step: f64,
    rounding: LinspaceRounding,
    coord_mapping: RangedCoordusize,
}

impl Linspace {
    pub fn new<T: Float>(range: Range<T>) -> Self {
        let start = range.start.to_f64().unwrap();
        let end = range.end.to_f64().unwrap();
        let step = if start == end {
            1.0
        } else {
            (end - start) / 50.0
        };
        let coord_mapping = if start == end {
            (0..0).into()
        } else {
            (0..50).into()
        };
        Self {
            start,
            end,
            step,
            rounding: LinspaceRounding::Round,
            coord_mapping,
        }
    }

    pub fn num_values(mut self, n: usize) -> Self {
        let idrange = self.coord_mapping.range();
        if idrange.end == 0 {
            return self;
        } else if n == 0 {
            self.step = 1.0;
            self.coord_mapping = (0..0).into();
            return self;
        }
        let new_step = (self.end - self.start) / n as f64;
        self.step = new_step;
        self.coord_mapping = (0..n).into();
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        let idrange = self.coord_mapping.range();
        if idrange.end == 0 {
            return self;
        }
        let new_count = ((self.end - self.start) / step).floor() as usize;
        self.step = step;
        self.coord_mapping = (0..new_count).into();
        self
    }
}

impl Ranged for Linspace {
    type ValueType = f64;
    fn map(&self, &value: &f64, limit: (i32, i32)) -> i32 {
        let index = self.rounding.get_index((value - self.start) / self.step);
        self.coord_mapping.map(&index, limit)
    }
    fn range(&self) -> Range<f64> {
        self.start..self.end
    }
    fn key_points(&self, max_points: usize) -> Vec<f64> {
        self.coord_mapping
            .key_points(max_points)
            .into_iter()
            .map(|value| value as f64 * self.step + self.start)
            .collect()
    }
}

impl DiscreteRanged for Linspace {
    fn size(&self) -> usize {
        self.coord_mapping.size()
    }

    fn index_of(&self, value: &Self::ValueType) -> Option<usize> {
        let index = self.rounding.get_index((*value - self.start) / self.step);
        Some(index)
    }

    fn from_index(&self, idx: usize) -> Option<Self::ValueType> {
        if idx >= self.size() {
            return None;
        }
        Some(self.step as f64 * idx as f64 + self.start)
    }
}

