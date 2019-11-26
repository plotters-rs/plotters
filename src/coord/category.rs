use std::fmt;
use std::ops::Range;
use std::rc::Rc;

use super::{AsRangedCoord, Ranged};

/// The category coordinate
pub struct Category<T: PartialEq> {
    name: String,
    elements: Rc<Vec<T>>,
    // i32 type is required for the empty ref (having -1 value)
    idx: i32,
}

impl<T: PartialEq> Clone for Category<T> {
    fn clone(&self) -> Self {
        Category {
            name: self.name.clone(),
            elements: Rc::clone(&self.elements),
            idx: self.idx,
        }
    }
}

impl<T: PartialEq + fmt::Display> fmt::Debug for Category<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let element = &self.elements[self.idx as usize];
        write!(f, "{}", element)
    }
}

impl<T: PartialEq> Category<T> {
    /// Create a new category coordinate.
    ///
    /// - `name`: The name of the category
    /// - `elements`: The vector of category elements
    /// - **returns** The newly created category coordinate
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let category = Category::new("color", vec!["red", "green", "blue"]);
    /// ```
    pub fn new<S: Into<String>>(name: S, elements: Vec<T>) -> Self {
        Self {
            name: name.into(),
            elements: Rc::new(elements),
            idx: -1,
        }
    }

    /// Get an element reference (tick) by its value.
    ///
    /// - `val`: The value of the element
    /// - **returns** The optional reference
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let category = Category::new("color", vec!["red", "green", "blue"]);
    /// let red = category.get(&"red");
    /// assert!(red.is_some());
    /// let unknown = category.get(&"unknown");
    /// assert!(unknown.is_none());
    /// ```
    pub fn get(&self, val: &T) -> Option<Category<T>> {
        match self.elements.iter().position(|x| x == val) {
            Some(pos) => {
                let element_ref = Category {
                    name: self.name.clone(),
                    elements: Rc::clone(&self.elements),
                    idx: pos as i32,
                };
                Some(element_ref)
            }
            _ => None,
        }
    }

    /// Create a full range over the category elements.
    ///
    /// - **returns** The range including all category elements
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let category = Category::new("color", vec!["red", "green", "blue"]);
    /// let range = category.range();
    /// ```
    pub fn range(&self) -> Self {
        self.clone()
    }

    /// Get the number of elements in the category.
    ///
    /// - **returns** The number of elements
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let category = Category::new("color", vec!["red", "green", "blue"]);
    /// assert_eq!(category.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if the category contains no elements.
    ///
    /// - **returns** `true` is no elements, otherwise - `false`
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let category = Category::new("color", vec!["red", "green", "blue"]);
    /// assert_eq!(category.is_empty(), false);
    ///
    /// let category = Category::new("empty", Vec::<&str>::new());
    /// assert_eq!(category.is_empty(), true);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get the category name.
    ///
    /// - **returns** The name of the category
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let category = Category::new("color", vec!["red", "green", "blue"]);
    /// assert_eq!(category.name(), "color");
    /// ```
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl<T: PartialEq> Ranged for Category<T> {
    type ValueType = Category<T>;

    fn range(&self) -> Range<Category<T>> {
        let mut left = self.clone();
        let mut right = self.clone();
        left.idx = 0;
        right.idx = right.len() as i32 - 1;
        left..right
    }

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        // Add margins to spans as edge values are not applicable to category
        let total_span = (self.len() + 1) as f64;
        let value_span = f64::from(value.idx + 1);
        (f64::from(limit.1 - limit.0) * value_span / total_span) as i32 + limit.0
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        let mut ret = vec![];
        let intervals = (self.len() - 1) as f64;
        let elements = &self.elements;
        let name = &self.name;
        let step = (intervals / max_points as f64 + 1.0) as usize;
        for idx in (0..self.len()).step_by(step) {
            ret.push(Category {
                name: name.clone(),
                elements: Rc::clone(&elements),
                idx: idx as i32,
            });
        }
        ret
    }
}

impl<T: PartialEq> AsRangedCoord for Category<T> {
    type CoordDescType = Self;
    type Value = Category<T>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_clone_trait() {
        let category = Category::new("color", vec!["red", "green", "blue"]);
        let red = category.get(&"red").unwrap();
        assert_eq!(red.idx, 0);
        let clone = red.clone();
        assert_eq!(clone.idx, 0);
    }

    #[test]
    fn test_debug_trait() {
        let category = Category::new("color", vec!["red", "green", "blue"]);
        let red = category.get(&"red").unwrap();
        assert_eq!(format!("{:?}", red), "red");
    }

    #[test]
    fn test_ranged_trait() {
        let category = Category::new("color", vec!["red", "green", "blue"]);
        assert_eq!(category.map(&category.get(&"red").unwrap(), (0, 8)), 2);
        assert_eq!(category.map(&category.get(&"green").unwrap(), (0, 8)), 4);
        assert_eq!(category.map(&category.get(&"blue").unwrap(), (0, 8)), 6);
        assert_eq!(category.key_points(3).len(), 3);
        assert_eq!(category.key_points(5).len(), 3);
    }
}
