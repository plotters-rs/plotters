use paste::paste;
use std::collections::HashMap;
use std::iter::once;

use stretch::number::OrElse;
use stretch::{
    geometry,
    geometry::Size,
    node::{MeasureFunc, Node, Stretch},
    number::Number,
    style::*,
};

macro_rules! impl_get_size {
    ($name:ident) => {
        paste! {
            #[doc = "Get the size of the `" $name "` container."]
            #[doc = "  * **Returns**: An option containing a tuple `(width, height)`."]
            pub fn [<get_ $name _size>](&self) -> Option<(i32, i32)> {
                self.extents_cache.as_ref().and_then(|extents_cache| {
                    extents_cache
                        .get(&self.$name)
                        .map(|&(x1, y1, x2, y2)| (x2 - x1, y2 - y1))
                })
            }
            #[doc = "Get the extents of the `" $name "` container."]
            #[doc = "  * **Returns**: An option containing a tuple `(x1,y1,x2,y2)`."]
            pub fn [<get_ $name _extents>](&self) -> Option<(i32, i32, i32, i32)> {
                self.extents_cache.as_ref().and_then(|extents_cache| {
                    extents_cache
                        .get(&self.$name)
                        .map(|&extent| extent)
                })
            }
        }
    };
    ($name:ident, $sub_part:ident) => {
        paste! {
            #[doc = "Get the size of the `" $name "." $sub_part "` container."]
            #[doc = "  * **Returns**: An option containing a tuple `(width, height)`."]
            pub fn [<get_ $name _size>](&self) -> Option<(i32, i32)> {
                self.extents_cache.as_ref().and_then(|extents_cache| {
                    extents_cache
                        .get(&self.$name.$sub_part)
                        .map(|&(x1, y1, x2, y2)| (x2 - x1, y2 - y1))
                })
            }
            #[doc = "Get the size of the `" $name "." $sub_part "` container."]
            #[doc = "  * **Returns**: An option containing a tuple `(x1,y1,x2,y2)`."]
            pub fn [<get_ $name _extents>](&self) -> Option<(i32, i32, i32, i32)> {
                self.extents_cache.as_ref().and_then(|extents_cache| {
                    extents_cache
                        .get(&self.$name.$sub_part)
                        .map(|&extent| extent)
                })
            }
        }
    };
}

macro_rules! impl_set_size {
    ($name:ident) => {
        paste! {
            #[doc = "Set the size of the `" $name "` container."]
            pub fn [<set_ $name _size>](
                &mut self,
                w: i32,
                h: i32,
            ) -> Result<(), Box<dyn std::error::Error>> {
                self.stretch_context.set_measure(
                    self.$name,
                    Some(new_measure_func_with_min_sizes(w as f32, h as f32)),
                )?;
                Ok(())
            }
        }
    };
    ($name:ident, $sub_part:ident) => {
        paste! {
            #[doc = "Set the size of the `" $name "." $sub_part "` container."]
            #[doc = "  * **Returns**: An option containing a tuple `(width, height)`."]
            pub fn [<set_ $name _size>](
                &mut self,
                w: i32,
                h: i32,
            ) -> Result<(), Box<dyn std::error::Error>> {
                self.stretch_context.set_measure(
                    self.$name.$sub_part,
                    Some(new_measure_func_with_min_sizes(w as f32, h as f32)),
                )?;
                Ok(())
            }
        }
    };
}

/// A structure containing two nodes, `inner` and `outer`.
/// `inner` is contained within `outer` and will be centered within
/// `outer`. `inner` will be centered horizontally for a `row_layout`
/// and vertically for a `col_layout`.
#[derive(Debug, Clone)]
pub(crate) struct CenteredLabelLayout {
    outer: Node,
    inner: Node,
}
impl CenteredLabelLayout {
    /// Create an inner node that is `justify-content: center` with respect
    /// to its outer node.
    fn new(stretch_context: &mut Stretch) -> Result<Self, Box<dyn std::error::Error>> {
        let inner = stretch_context.new_leaf(
            Default::default(),
            Box::new(|constraint| {
                Ok(stretch::geometry::Size {
                    width: constraint.width.or_else(0.0),
                    height: constraint.height.or_else(0.0),
                })
            }),
        )?;
        let outer = stretch_context.new_node(
            Style {
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            vec![inner],
        )?;

        Ok(Self { inner, outer })
    }
    /// Create an inner node that is horizontally centered in its 100% width parent.
    fn new_row_layout(stretch_context: &mut Stretch) -> Result<Self, Box<dyn std::error::Error>> {
        let layout = Self::new(stretch_context)?;
        // If the layout is placed in a row, the outer should have 100% width.
        let outer_style = *stretch_context.style(layout.outer)?;
        stretch_context.set_style(
            layout.outer,
            Style {
                flex_direction: FlexDirection::Row,
                ..outer_style
            },
        )?;

        Ok(layout)
    }
    /// Create an inner node that is vertically centered in its 100% height parent.
    fn new_col_layout(stretch_context: &mut Stretch) -> Result<Self, Box<dyn std::error::Error>> {
        let layout = Self::new(stretch_context)?;
        // If the layout is placed in a row, the outer should have 100% width.
        let outer_style = *stretch_context.style(layout.outer)?;
        stretch_context.set_style(
            layout.outer,
            Style {
                flex_direction: FlexDirection::Column,
                ..outer_style
            },
        )?;

        Ok(layout)
    }
}

/// A struct to store the layout structure of a chart using the `stretch`
/// library. The `stretch` library uses a flexbox-compatible algorithm to lay
/// out nodes. The layout hierarchy is equivalent to the following HTML.
/// ```html
/// <outer_container>
///		<chart_title.outer>
///        <chart_title.inner>Title</chart_title.inner>
///    </chart_title.outer>
///    <chart_container>
///        <left_area>
///            <left_label.outer>
///                <left_label.inner>left_label</left_label.inner>
///            </left_label.outer>
///            <left_tick_label />
///        </left_area>
///        <center_container>
///            <top_area>
///                <top_label.outer>
///                    <top_label.inner>top_label</top_label.inner>
///                </top_label.outer>
///                <top_tick_label />
///            </top_area>
///            <chart_area>CHART</chart_area>
///            <bottom_area>
///                <bottom_label.outer>
///                    <bottom_label.inner>bottom_label</bottom_label.inner>
///                </bottom_label.outer>
///                <bottom_tick_label />
///            </bottom_area>
///        </center_container>
///        <right_area>
///            <right_label.outer>
///                <right_label.inner>right_label</right_label.inner>
///            </right_label.outer>
///            <right_tick_label />
///        </right_area>
///    </chart_container>
///</outer_container>
/// ```
pub(crate) struct ChartLayoutNodes {
    /// A map from nodes to extents of the form `(x1,y1,x2,y2)` where
    /// `(x1,y1)` is the upper left corner of the node and
    /// `(x2,y2)` is the lower right corner of the node.
    extents_cache: Option<HashMap<Node, (i32, i32, i32, i32)>>,
    /// The `stretch` context that is used to compute the layout.
    stretch_context: Stretch,
    /// The outer-most node which contains all others.
    outer_container: Node,
    /// The title of the whole chart
    chart_title: CenteredLabelLayout,
    top_area: Node,
    /// x-axis label above chart
    top_label: CenteredLabelLayout,
    top_tick_label: Node,
    left_area: Node,
    /// y-axis label left of chart
    left_label: CenteredLabelLayout,
    left_tick_label: Node,
    right_area: Node,
    /// y-axis label right of chart
    right_label: CenteredLabelLayout,
    right_tick_label: Node,
    bottom_area: Node,
    /// x-axis label above chart
    bottom_label: CenteredLabelLayout,
    bottom_tick_label: Node,
    center_container: Node,
    chart_area: Node,
    chart_container: Node,
}

impl ChartLayoutNodes {
    /// Create a new `ChartLayoutNodes`. All margins/padding/sizes are set to 0
    /// and should be overridden as needed.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Set up the layout engine
        let mut stretch_context = Stretch::new();

        // Create the chart title
        let chart_title = CenteredLabelLayout::new_row_layout(&mut stretch_context)?;

        // Create the labels
        let (top_area, top_label, top_tick_label) =
            packed_title_label_area(&mut stretch_context, FlexDirection::Column)?;
        let (bottom_area, bottom_label, bottom_tick_label) =
            packed_title_label_area(&mut stretch_context, FlexDirection::ColumnReverse)?;
        let (left_area, left_label, left_tick_label) =
            packed_title_label_area(&mut stretch_context, FlexDirection::Row)?;
        let (right_area, right_label, right_tick_label) =
            packed_title_label_area(&mut stretch_context, FlexDirection::RowReverse)?;

        // Create the center chart area and column
        let chart_area = stretch_context.new_leaf(
            Style {
                flex_grow: 1.0,
                ..Default::default()
            },
            new_measure_func_with_defaults(),
        )?;
        let center_container = stretch_context.new_node(
            Style {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            vec![top_area, chart_area, bottom_area],
        )?;
        let chart_container = stretch_context.new_node(
            Style {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            vec![left_area, center_container, right_area],
        )?;

        // Pack everything together to make a full chart
        let outer_container = stretch_context.new_node(
            Style {
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Percent(1.0),
                },
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            vec![chart_title.outer, chart_container],
        )?;

        Ok(Self {
            extents_cache: None,
            stretch_context,
            outer_container,
            chart_title,
            top_area,
            top_label,
            top_tick_label,
            left_area,
            left_label,
            left_tick_label,
            right_area,
            right_label,
            right_tick_label,
            bottom_area,
            bottom_label,
            bottom_tick_label,
            center_container,
            chart_area,
            chart_container,
        })
    }
    /// Compute the layout of all items to fill a container of width
    /// `w` and height `h`.
    pub fn layout(&mut self, w: u32, h: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Compute the initial layout
        self.stretch_context.compute_layout(
            self.outer_container,
            Size {
                width: Number::Defined(w as f32),
                height: Number::Defined(h as f32),
            },
        )?;

        // By default the flex containers on the left and right
        // will be the full height of the `chart_container`. However, we'd
        // actually like them to be the height of the `chart_area`. To achieve
        // this, we apply margins of the appropriate size and then recompute
        // the layout.
        let top_area_layout = self.stretch_context.layout(self.top_area)?;
        let bottom_area_layout = self.stretch_context.layout(self.bottom_area)?;
        let margin = geometry::Rect {
            top: Dimension::Points(top_area_layout.size.height),
            bottom: Dimension::Points(bottom_area_layout.size.height),
            start: Dimension::Undefined,
            end: Dimension::Undefined,
        };
        let old_style = *self.stretch_context.style(self.left_area)?;
        self.stretch_context.set_style(
            self.left_area,
            Style {
                margin,
                ..old_style
            },
        )?;
        let old_style = *self.stretch_context.style(self.right_area)?;
        self.stretch_context.set_style(
            self.right_area,
            Style {
                margin,
                ..old_style
            },
        )?;

        // Recompute the layout with the new margins set.
        // According to the `stretch` documentation, this is very efficient.
        self.stretch_context.compute_layout(
            self.outer_container,
            Size {
                width: Number::Defined(w as f32),
                height: Number::Defined(h as f32),
            },
        )?;

        self.extents_cache = Some(compute_child_extents(
            &self.stretch_context,
            self.outer_container,
        ));

        Ok(())
    }

    pub fn get_chart_area_size(&self) -> Option<(i32, i32)> {
        self.extents_cache.as_ref().and_then(|extents_cache| {
            extents_cache
                .get(&self.chart_area)
                .map(|&(x1, y1, x2, y2)| (x2 - x1, y2 - y1))
        })
    }
    pub fn set_chart_area_size(
        &mut self,
        w: i32,
        h: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.stretch_context.set_measure(
            self.chart_area,
            Some(new_measure_func_with_min_sizes(w as f32, h as f32)),
        )?;
        Ok(())
    }
    // Getters for relevant box sizes
    impl_get_size!(outer_container);
    impl_get_size!(top_tick_label);
    impl_get_size!(bottom_tick_label);
    impl_get_size!(left_tick_label);
    impl_get_size!(right_tick_label);
    impl_get_size!(chart_title, inner);
    impl_get_size!(top_label, inner);
    impl_get_size!(bottom_label, inner);
    impl_get_size!(left_label, inner);
    impl_get_size!(right_label, inner);

    // Setters for relevant box sizes
    impl_set_size!(top_tick_label);
    impl_set_size!(bottom_tick_label);
    impl_set_size!(left_tick_label);
    impl_set_size!(right_tick_label);
    impl_set_size!(chart_title, inner);
    impl_set_size!(top_label, inner);
    impl_set_size!(bottom_label, inner);
    impl_set_size!(left_label, inner);
    impl_set_size!(right_label, inner);
}

/// Pack a centered title and a label-area together in a row (`FlexDirection::Row`/`RowReverse`)
/// or column (`FlexDirection::Column`/`ColumnReverse`).
///   * `stretch_context` - The `Stretch` context
///   * `flex_direction` - How the title-area and label-area are to be layed out.
///   * **Returns**: A triple `(outer_area, title_area, label_area)`. The `outer_area` contains both the `title_area` and the `label_area`.
fn packed_title_label_area(
    stretch_context: &mut Stretch,
    flex_direction: FlexDirection,
) -> Result<(Node, CenteredLabelLayout, Node), Box<dyn std::error::Error>> {
    let title = match flex_direction {
        FlexDirection::Row | FlexDirection::RowReverse => {
            // If the title and the label are packed in a row, the title should be centered in a *column*.
            CenteredLabelLayout::new_col_layout(stretch_context)?
        }
        FlexDirection::Column | FlexDirection::ColumnReverse => {
            // If the title and the label are packed in a column, the title should be centered in a *row*.
            CenteredLabelLayout::new_row_layout(stretch_context)?
        }
    };
    let label = stretch_context.new_leaf(
        Default::default(),
        Box::new(|constraint| {
            Ok(stretch::geometry::Size {
                width: constraint.width.or_else(0.0),
                height: constraint.height.or_else(0.0),
            })
        }),
    )?;
    let outer = stretch_context.new_node(
        Style {
            flex_direction,
            ..Default::default()
        },
        vec![title.outer, label],
    )?;

    Ok((outer, title, label))
}

fn new_measure_func_with_min_sizes(w: f32, h: f32) -> MeasureFunc {
    Box::new(move |constraint| {
        Ok(stretch::geometry::Size {
            width: constraint.width.or_else(w),
            height: constraint.height.or_else(h),
        })
    })
}
fn new_measure_func_with_defaults() -> MeasureFunc {
    Box::new(move |constraint| {
        Ok(stretch::geometry::Size {
            width: constraint.width.or_else(0.),
            height: constraint.height.or_else(0.),
        })
    })
}

/// When `stretch` computes the layout of a node, its
/// extents are computed relatively to the parent. We want absolute positions,
/// so we need to compute them manually.
///   * **Returns**: A `HashMap` from nodes to tuples `(x1,y1,x2,y2)` where `(x1,y1)` and `(x2,y2)` represent the upper left and lower right corners of the bounding rectangle.
fn compute_child_extents(stretch: &Stretch, node: Node) -> HashMap<Node, (i32, i32, i32, i32)> {
    const DEFAULT_CAPACITY: usize = 16;
    let mut ret = HashMap::with_capacity(DEFAULT_CAPACITY);
    fn _compute_child_extents(
        stretch: &Stretch,
        node: Node,
        offset: (i32, i32),
        store: &mut HashMap<Node, (i32, i32, i32, i32)>,
    ) {
        let layout = stretch.layout(node).unwrap();
        let geometry::Point { x, y } = layout.location;
        let geometry::Size { width, height } = layout.size;
        let (x1, y1) = (x as i32 + offset.0, y as i32 + offset.1);
        let (x2, y2) = ((width) as i32 + x1, (height) as i32 + y1);
        store.insert(node, (x1, y1, x2, y2));

        if stretch.child_count(node).unwrap() > 0 {
            for child in stretch.children(node).unwrap() {
                _compute_child_extents(stretch, child, (x1, y1), store);
            }
        }
    }
    _compute_child_extents(stretch, node, (0, 0), &mut ret);
    ret
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    /// The default layout should make the chart area take the full area.
    fn full_chart_area() {
        let mut layout = ChartLayoutNodes::new().unwrap();
        layout.layout(70, 50).unwrap();
        let extents_cache = layout.extents_cache.unwrap();
        let &(x1, y1, x2, y2) = extents_cache.get(&layout.chart_area).unwrap();

        assert_eq!(x1, 0);
        assert_eq!(y1, 0);
        assert_eq!(x2, 70);
        assert_eq!(y2, 50);
    }
    #[test]
    /// The default layout should make the chart area take the full area.
    fn full_chart_area_with_getter() {
        let mut layout = ChartLayoutNodes::new().unwrap();
        layout.layout(70, 50).unwrap();
        let (w, h) = layout.get_chart_area_size().unwrap();

        assert_eq!(w, 70);
        assert_eq!(h, 50);
    }
    #[test]
    fn full_chart_area_with_getter_without_running_layout() {
        let layout = ChartLayoutNodes::new().unwrap();
        assert_eq!(layout.get_chart_area_size(), None);
    }
    #[test]
    /// The outer container should always be the full size.
    fn full_outer_container_size_with_getter() {
        let mut layout = ChartLayoutNodes::new().unwrap();
        layout.layout(70, 50).unwrap();
        let (w, h) = layout.get_outer_container_size().unwrap();

        assert_eq!(w, 70);
        assert_eq!(h, 50);
    }
    #[test]
    fn zero_config_chart_title_size_with_getter() {
        let mut layout = ChartLayoutNodes::new().unwrap();
        layout.layout(70, 50).unwrap();
        let (w, h) = layout.get_chart_title_size().unwrap();

        assert_eq!(w, 0);
        assert_eq!(h, 0);
    }
    #[test]
    /// The outer container should always be the full size.
    fn set_chart_title_size() {
        let mut layout = ChartLayoutNodes::new().unwrap();
        layout.set_chart_title_size(20, 20).unwrap();
        layout.layout(70, 50).unwrap();
        let (w, h) = layout.get_chart_title_size().unwrap();

        assert_eq!(w, 20);
        assert_eq!(h, 20);

        let (x1, y1, x2, y2) = layout.get_chart_title_extents().unwrap();
        assert_eq!((x1, y1, x2, y2), (25, 0, 45, 20));
    }
}
