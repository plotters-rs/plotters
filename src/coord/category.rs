use std::fmt;
use std::ops::Range;
use std::rc::Rc;

use super::{AsRangedCoord, Ranged};

/// The category coordinate
pub struct Category<T: PartialEq> {
    name: String,
    elements: Rc<Vec<T>>,
}

/// The category element reference (tick).
pub struct CategoryElementRef<T: PartialEq> {
    inner: Rc<Vec<T>>,
    // i32 type is required for the empty ref (having -1 value)
    idx: i32,
}

/// The category elements range.
pub struct CategoryElementsRange<T: PartialEq>(CategoryElementRef<T>, CategoryElementRef<T>);

impl<T: PartialEq> Clone for CategoryElementRef<T> {
    fn clone(&self) -> Self {
        CategoryElementRef {
            inner: Rc::clone(&self.inner),
            idx: self.idx,
        }
    }
}

impl<T: PartialEq + fmt::Display> fmt::Debug for CategoryElementRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let element = &self.inner[self.idx as usize];
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
    pub fn get(&self, val: &T) -> Option<CategoryElementRef<T>> {
        match self.elements.iter().position(|x| x == val) {
            Some(pos) => {
                let element_ref = CategoryElementRef {
                    inner: Rc::clone(&self.elements),
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
    pub fn range(&self) -> CategoryElementsRange<T> {
        let start = 0;
        let end = self.elements.len() as i32 - 1;
        CategoryElementsRange(
            CategoryElementRef {
                inner: Rc::clone(&self.elements),
                idx: start,
            },
            CategoryElementRef {
                inner: Rc::clone(&self.elements),
                idx: end,
            },
        )
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

impl<T: PartialEq> From<Range<CategoryElementRef<T>>> for CategoryElementsRange<T> {
    fn from(range: Range<CategoryElementRef<T>>) -> Self {
        Self(range.start, range.end)
    }
}

impl<T: PartialEq> Ranged for CategoryElementsRange<T> {
    type ValueType = CategoryElementRef<T>;

    fn range(&self) -> Range<CategoryElementRef<T>> {
        self.0.clone()..self.1.clone()
    }

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        // Add margins to spans as edge values are not applicable to category
        let total_span = (self.1.idx - self.0.idx + 2) as f64;
        let value_span = (value.idx - self.0.idx + 1) as f64;
        (f64::from(limit.1 - limit.0) * value_span / total_span) as i32 + limit.0
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        let mut ret = vec![];
        let intervals = (self.1.idx - self.0.idx) as f64;
        let inner = &self.0.inner;
        let step = (intervals / max_points as f64 + 1.0) as usize;
        for idx in (self.0.idx..=self.1.idx).step_by(step) {
            ret.push(CategoryElementRef {
                inner: Rc::clone(&inner),
                idx,
            });
        }
        ret
    }
}

impl<T: PartialEq> AsRangedCoord for Range<CategoryElementRef<T>> {
    type CoordDescType = CategoryElementsRange<T>;
    type Value = CategoryElementRef<T>;
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
    fn test_from_range_trait() {
        let category = Category::new("color", vec!["red", "green", "blue"]);
        let range = category.get(&"red").unwrap()..category.get(&"blue").unwrap();
        let elements_range = CategoryElementsRange::from(range);
        assert_eq!(elements_range.0.idx, 0);
        assert_eq!(elements_range.1.idx, 2);
    }

    #[test]
    fn test_ranged_trait() {
        let category = Category::new("color", vec!["red", "green", "blue"]);
        let elements_range = category.range();
        let range = elements_range.range();
        let elements_range = CategoryElementsRange::from(range);
        assert_eq!(elements_range.0.idx, 0);
        assert_eq!(elements_range.1.idx, 2);
        assert_eq!(elements_range.map(&elements_range.0, (10, 20)), 12);
        assert_eq!(elements_range.key_points(5).len(), 3);
    }
}
