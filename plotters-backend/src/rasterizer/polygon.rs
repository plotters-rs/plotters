use crate::{
    math_guard::{
        checked_add_u32, checked_add_usize, checked_mul_i64, checked_sub_i32, checked_sub_i64,
        checked_sub_u32, checked_sub_usize, i32_to_u32_checked, non_zero_f64,
    },
    BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind, MathError,
};

use std::{
    cmp::{Ord, Ordering, PartialOrd},
    convert::TryFrom,
};

#[derive(Clone, Debug)]
struct Edge {
    epoch: u32,
    total_epoch: u32,
    slave_begin: i32,
    slave_end: i32,
}

impl Edge {
    fn horizontal_sweep(
        mut from: BackendCoord,
        mut to: BackendCoord,
    ) -> Result<Option<Edge>, MathError> {
        if from.0 == to.0 {
            return Ok(None);
        }

        if from.0 > to.0 {
            std::mem::swap(&mut from, &mut to);
        }

        let total_epoch = i32_to_u32_checked(checked_sub_i32(to.0, from.0)?)?;
        Ok(Some(Edge {
            epoch: 0,
            total_epoch,
            slave_begin: from.1,
            slave_end: to.1,
        }))
    }

    fn vertical_sweep(from: BackendCoord, to: BackendCoord) -> Result<Option<Edge>, MathError> {
        Edge::horizontal_sweep((from.1, from.0), (to.1, to.0))
    }

    fn get_master_pos(&self) -> Result<i32, MathError> {
        let epoch_diff = checked_sub_u32(self.total_epoch, self.epoch)?;
        i32::try_from(epoch_diff).map_err(|_| MathError::ValueOutOfRange)
    }

    fn inc_epoch(&mut self) -> Result<(), MathError> {
        self.epoch = checked_add_u32(self.epoch, 1)?;
        Ok(())
    }

    fn get_slave_pos(&self) -> Result<f64, MathError> {
        let slave_diff = checked_sub_i64(i64::from(self.slave_end), i64::from(self.slave_begin))?;
        let product = checked_mul_i64(slave_diff, i64::from(self.epoch))? as f64;
        let total_epoch = non_zero_f64(f64::from(self.total_epoch))?;

        Ok(f64::from(self.slave_begin) + product / total_epoch)
    }
    /// Helper method to avoid returning a `Result`, necessary for ordering and equality where `Result` is not permissable
    fn get_slave_pos_unchecked_for_sort(&self) -> f64 {
        self.get_slave_pos()
            .expect("edge slave position calculation failed during sort")
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.get_slave_pos_unchecked_for_sort() == other.get_slave_pos_unchecked_for_sort()
    }
}

impl Eq for Edge {}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_slave_pos_unchecked_for_sort()
            .total_cmp(&other.get_slave_pos_unchecked_for_sort())
    }
}

pub fn fill_polygon<DB: DrawingBackend, S: BackendStyle>(
    back: &mut DB,
    vertices: &[BackendCoord],
    style: &S,
) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
    if let Some((x_span, y_span)) =
        vertices
            .iter()
            .fold(None, |res: Option<((i32, i32), (i32, i32))>, (x, y)| {
                Some(
                    res.map(|((min_x, max_x), (min_y, max_y))| {
                        (
                            (min_x.min(*x), max_x.max(*x)),
                            (min_y.min(*y), max_y.max(*y)),
                        )
                    })
                    .unwrap_or(((*x, *x), (*y, *y))),
                )
            })
    {
        // First of all, let's handle the case that all the points is in a same vertical or
        // horizontal line
        if x_span.0 == x_span.1 || y_span.0 == y_span.1 {
            return back.draw_line((x_span.0, y_span.0), (x_span.1, y_span.1), style);
        }
        let x_diff = checked_sub_i32(x_span.1, x_span.0)?;
        let y_diff = checked_sub_i32(y_span.1, y_span.0)?;
        let horizontal_sweep = x_diff > y_diff;
        let last_idx = checked_sub_usize(vertices.len(), 1)?;
        let mut edges: Vec<_> = vertices
            .iter()
            .zip(vertices.iter().skip(1))
            .map(|(a, b)| (*a, *b))
            .collect();
        edges.push((vertices[last_idx], vertices[0]));
        edges.sort_by_key(|((x1, y1), (x2, y2))| {
            if horizontal_sweep {
                *x1.min(x2)
            } else {
                *y1.min(y2)
            }
        });

        for edge in &mut edges.iter_mut() {
            if horizontal_sweep {
                if (edge.0).0 > (edge.1).0 {
                    std::mem::swap(&mut edge.0, &mut edge.1);
                }
            } else if (edge.0).1 > (edge.1).1 {
                std::mem::swap(&mut edge.0, &mut edge.1);
            }
        }

        let (low, high) = if horizontal_sweep { x_span } else { y_span };

        let mut idx = 0;

        let mut active_edge: Vec<Edge> = vec![];

        for sweep_line in low..=high {
            let mut new_vec = vec![];

            for mut e in active_edge {
                if e.get_master_pos()? > 0 {
                    e.inc_epoch()?;
                    new_vec.push(e);
                }
            }

            active_edge = new_vec;

            loop {
                if idx >= edges.len() {
                    break;
                }
                let line = if horizontal_sweep {
                    (edges[idx].0).0
                } else {
                    (edges[idx].0).1
                };
                if line > sweep_line {
                    break;
                }

                let edge_obj = if horizontal_sweep {
                    Edge::horizontal_sweep(edges[idx].0, edges[idx].1)
                } else {
                    Edge::vertical_sweep(edges[idx].0, edges[idx].1)
                }?;

                if let Some(edge_obj) = edge_obj {
                    active_edge.push(edge_obj);
                }

                idx = checked_add_usize(idx, 1)?;
            }

            active_edge.sort();

            let mut first = None;
            let mut second = None;

            for edge in active_edge.iter() {
                if first.is_none() {
                    first = Some(edge.clone())
                } else if second.is_none() {
                    second = Some(edge.clone())
                }

                if let Some(a) = first.clone() {
                    if let Some(b) = second.clone() {
                        if a.get_master_pos()? == 0 && b.get_master_pos()? != 0 {
                            first = Some(b);
                            second = None;
                            continue;
                        }

                        if a.get_master_pos()? != 0 && b.get_master_pos()? == 0 {
                            first = Some(a);
                            second = None;
                            continue;
                        }

                        let from = a.get_slave_pos()?;
                        let to = b.get_slave_pos()?;

                        if a.get_master_pos()? == 0 && b.get_master_pos()? == 0 && to - from > 1.0 {
                            first = None;
                            second = None;
                            continue;
                        }

                        if horizontal_sweep {
                            check_result!(back.draw_line(
                                (sweep_line, from.ceil() as i32),
                                (sweep_line, to.floor() as i32),
                                &style.color(),
                            ));
                            check_result!(back.draw_pixel(
                                (sweep_line, from.floor() as i32),
                                style.color().mix(from.ceil() - from),
                            ));
                            check_result!(back.draw_pixel(
                                (sweep_line, to.ceil() as i32),
                                style.color().mix(to - to.floor()),
                            ));
                        } else {
                            check_result!(back.draw_line(
                                (from.ceil() as i32, sweep_line),
                                (to.floor() as i32, sweep_line),
                                &style.color(),
                            ));
                            check_result!(back.draw_pixel(
                                (from.floor() as i32, sweep_line),
                                style.color().mix(from.ceil() - from),
                            ));
                            check_result!(back.draw_pixel(
                                (to.ceil() as i32, sweep_line),
                                style.color().mix(to.floor() - to),
                            ));
                        }

                        first = None;
                        second = None;
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BackendColor, BackendStyle};

    #[derive(Debug)]
    struct TestBackendError;

    impl std::fmt::Display for TestBackendError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test backend error")
        }
    }

    impl std::error::Error for TestBackendError {}

    #[derive(Clone, Copy)]
    struct TestStyle {
        color: BackendColor,
    }

    impl BackendStyle for TestStyle {
        fn color(&self) -> BackendColor {
            self.color
        }

        fn stroke_width(&self) -> u32 {
            1
        }
    }

    #[derive(Default)]
    struct TestBackend {
        lines: Vec<(BackendCoord, BackendCoord)>,
        pixels: Vec<(BackendCoord, BackendColor)>,
    }

    impl DrawingBackend for TestBackend {
        type ErrorType = TestBackendError;

        fn get_size(&self) -> (u32, u32) {
            (100, 100)
        }

        fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            Ok(())
        }

        fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            Ok(())
        }

        fn draw_pixel(
            &mut self,
            point: BackendCoord,
            color: BackendColor,
        ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            self.pixels.push((point, color));
            Ok(())
        }

        fn draw_line<S: BackendStyle>(
            &mut self,
            from: BackendCoord,
            to: BackendCoord,
            _style: &S,
        ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
            self.lines.push((from, to));
            Ok(())
        }
    }

    fn visible_style() -> TestStyle {
        TestStyle {
            color: BackendColor {
                rgb: (0, 0, 0),
                alpha: 1.0,
            },
        }
    }

    #[test]
    fn horizontal_sweep_returns_none_for_vertical_edge() {
        assert!(Edge::horizontal_sweep((1, 2), (1, 5)).unwrap().is_none());
    }

    #[test]
    fn horizontal_sweep_normalizes_direction() {
        let edge = Edge::horizontal_sweep((5, 10), (2, 20)).unwrap().unwrap();

        assert_eq!(edge.epoch, 0);
        assert_eq!(edge.total_epoch, 3);
        assert_eq!(edge.slave_begin, 20);
        assert_eq!(edge.slave_end, 10);
    }

    #[test]
    fn horizontal_sweep_reports_overflow_for_extreme_span() {
        let err = Edge::horizontal_sweep((i32::MIN, 0), (i32::MAX, 0)).unwrap_err();

        assert_eq!(err, MathError::ValueOutOfRange);
    }

    #[test]
    fn vertical_sweep_uses_y_axis_as_master_axis() {
        let edge = Edge::vertical_sweep((10, 2), (20, 5)).unwrap().unwrap();

        assert_eq!(edge.epoch, 0);
        assert_eq!(edge.total_epoch, 3);
        assert_eq!(edge.slave_begin, 10);
        assert_eq!(edge.slave_end, 20);
    }

    #[test]
    fn get_master_pos_returns_remaining_epoch_distance() {
        let edge = Edge {
            epoch: 2,
            total_epoch: 5,
            slave_begin: 0,
            slave_end: 10,
        };

        assert_eq!(edge.get_master_pos(), Ok(3));
    }

    #[test]
    fn get_master_pos_reports_underflow_when_epoch_exceeds_total_epoch() {
        let edge = Edge {
            epoch: 6,
            total_epoch: 5,
            slave_begin: 0,
            slave_end: 10,
        };

        assert_eq!(edge.get_master_pos(), Err(MathError::ValueOutOfRange));
    }

    #[test]
    fn get_master_pos_reports_out_of_range_for_large_u32_result() {
        let edge = Edge {
            epoch: 0,
            total_epoch: u32::MAX,
            slave_begin: 0,
            slave_end: 10,
        };

        assert_eq!(edge.get_master_pos(), Err(MathError::ValueOutOfRange));
    }

    #[test]
    fn inc_epoch_advances_epoch() {
        let mut edge = Edge {
            epoch: 0,
            total_epoch: 5,
            slave_begin: 0,
            slave_end: 10,
        };

        assert_eq!(edge.inc_epoch(), Ok(()));
        assert_eq!(edge.epoch, 1);
    }

    #[test]
    fn inc_epoch_reports_overflow_at_max_epoch() {
        let mut edge = Edge {
            epoch: u32::MAX,
            total_epoch: u32::MAX,
            slave_begin: 0,
            slave_end: 10,
        };

        assert_eq!(edge.inc_epoch(), Err(MathError::ValueOutOfRange));
        assert_eq!(edge.epoch, u32::MAX);
    }

    #[test]
    fn get_slave_pos_interpolates_between_slave_points() {
        let edge = Edge {
            epoch: 5,
            total_epoch: 10,
            slave_begin: 0,
            slave_end: 20,
        };

        assert_eq!(edge.get_slave_pos(), Ok(10.0));
    }

    #[test]
    fn get_slave_pos_includes_slave_begin_offset() {
        let edge = Edge {
            epoch: 5,
            total_epoch: 10,
            slave_begin: 10,
            slave_end: 30,
        };

        assert_eq!(edge.get_slave_pos(), Ok(20.0));
    }

    #[test]
    fn get_slave_pos_rejects_zero_total_epoch() {
        let edge = Edge {
            epoch: 5,
            total_epoch: 0,
            slave_begin: 0,
            slave_end: 20,
        };

        assert_eq!(edge.get_slave_pos(), Err(MathError::ZeroDivision));
    }

    #[test]
    fn edge_ordering_sorts_by_slave_position() {
        let low = Edge {
            epoch: 5,
            total_epoch: 10,
            slave_begin: 0,
            slave_end: 10,
        };

        let high = Edge {
            epoch: 5,
            total_epoch: 10,
            slave_begin: 0,
            slave_end: 20,
        };

        assert!(low < high);
    }

    #[test]
    fn edge_ordering_treats_equal_slave_positions_as_equal() {
        let a = Edge {
            epoch: 5,
            total_epoch: 10,
            slave_begin: 0,
            slave_end: 20,
        };

        let b = Edge {
            epoch: 10,
            total_epoch: 20,
            slave_begin: 0,
            slave_end: 20,
        };

        assert_eq!(a.cmp(&b), Ordering::Equal);
        assert_eq!(a, b);
    }

    #[test]
    fn edge_ordering_sorts_vec_by_slave_position() {
        let mut edges = [
            Edge {
                epoch: 5,
                total_epoch: 10,
                slave_begin: 0,
                slave_end: 30,
            },
            Edge {
                epoch: 5,
                total_epoch: 10,
                slave_begin: 0,
                slave_end: 10,
            },
            Edge {
                epoch: 5,
                total_epoch: 10,
                slave_begin: 0,
                slave_end: 20,
            },
        ];

        edges.sort();

        let positions: Vec<f64> = edges
            .iter()
            .map(|edge| edge.get_slave_pos().expect("test edge should be valid"))
            .collect();

        assert_eq!(positions, vec![5.0, 10.0, 15.0]);
    }

    #[test]
    fn fill_polygon_with_empty_vertices_draws_nothing() {
        let mut backend = TestBackend::default();

        fill_polygon(&mut backend, &[], &visible_style()).unwrap();

        assert!(backend.lines.is_empty());
        assert!(backend.pixels.is_empty());
    }

    #[test]
    fn fill_polygon_with_horizontal_line_draws_single_line() {
        let mut backend = TestBackend::default();

        fill_polygon(&mut backend, &[(1, 2), (4, 2), (7, 2)], &visible_style()).unwrap();

        assert_eq!(backend.lines, vec![((1, 2), (7, 2))]);
        assert!(backend.pixels.is_empty());
    }

    #[test]
    fn fill_polygon_with_vertical_line_draws_single_line() {
        let mut backend = TestBackend::default();

        fill_polygon(&mut backend, &[(3, 1), (3, 4), (3, 7)], &visible_style()).unwrap();

        assert_eq!(backend.lines, vec![((3, 1), (3, 7))]);
        assert!(backend.pixels.is_empty());
    }

    #[test]
    fn fill_polygon_fills_simple_rectangle() {
        let mut backend = TestBackend::default();

        fill_polygon(
            &mut backend,
            &[(1, 1), (4, 1), (4, 3), (1, 3)],
            &visible_style(),
        )
        .unwrap();

        assert!(!backend.lines.is_empty());
    }

    #[test]
    fn fill_polygon_propagates_math_error_from_extreme_horizontal_span() {
        let mut backend = TestBackend::default();

        let err = fill_polygon(
            &mut backend,
            &[(i32::MIN, 0), (i32::MAX, 1), (i32::MAX, 2), (i32::MIN, 2)],
            &visible_style(),
        )
        .unwrap_err();
        dbg!(&err);
        assert!(matches!(
            err,
            DrawingErrorKind::MathError(MathError::ValueOutOfRange)
        ));
    }
}
