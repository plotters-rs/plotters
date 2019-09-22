use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingErrorKind};
use crate::drawing::DrawingBackend;

use crate::style::Color;

use std::cmp::{Ord, Ordering, PartialOrd};

#[derive(Clone, Debug)]
struct Edge {
    epoch: u32,
    total_epoch: u32,
    slave_begin: i32,
    slave_end: i32,
}

impl Edge {
    fn horizental_sweep(mut from: BackendCoord, mut to: BackendCoord) -> Option<Edge> {
        if from.0 == to.0 {
            return None;
        }

        if from.0 > to.0 {
            std::mem::swap(&mut from, &mut to);
        }

        Some(Edge {
            epoch: 0,
            total_epoch: (to.0 - from.0) as u32,
            slave_begin: from.1,
            slave_end: to.1,
        })
    }

    fn vertical_sweep(from: BackendCoord, to: BackendCoord) -> Option<Edge> {
        Edge::horizental_sweep((from.1, from.0), (to.1, to.0))
    }

    fn get_master_pos(&self) -> i32 {
        (self.total_epoch - self.epoch) as i32
    }

    fn inc_epoch(&mut self) {
        self.epoch += 1;
    }

    fn get_slave_pos(&self) -> f64 {
        self.slave_begin as f64
            + ((self.slave_end - self.slave_begin) as i64 * self.epoch as i64) as f64
                / self.total_epoch as f64
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_slave_pos().partial_cmp(&other.get_slave_pos())
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.get_slave_pos() == other.get_slave_pos()
    }
}

impl Eq for Edge {}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_slave_pos()
            .partial_cmp(&other.get_slave_pos())
            .unwrap()
    }
}

pub(crate) fn fill_polygon<DB: DrawingBackend, S: BackendStyle>(
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
        // First of all, let's handle the case that all the points is in a same vertica or
        // horizental line
        if x_span.0 == x_span.1 || y_span.0 == y_span.1 {
            return back.draw_line((x_span.0, y_span.0), (x_span.1, y_span.1), style);
        }

        let horizental_sweep = x_span.1 - x_span.0 > y_span.1 - y_span.0;

        let mut edges: Vec<_> = vertices
            .iter()
            .zip(vertices.iter().skip(1))
            .map(|(a, b)| (*a, *b))
            .collect();
        edges.push((vertices[vertices.len() - 1], vertices[0]));
        edges.sort_by_key(|((x1, y1), (x2, y2))| {
            if horizental_sweep {
                *x1.min(x2)
            } else {
                *y1.min(y2)
            }
        });

        for ref mut edge in edges.iter_mut() {
            if horizental_sweep {
                if (edge.0).0 > (edge.1).0 {
                    std::mem::swap(&mut edge.0, &mut edge.1);
                }
            } else {
                if (edge.0).1 > (edge.1).1 {
                    std::mem::swap(&mut edge.0, &mut edge.1);
                }
            }
        }

        let (low, high) = if horizental_sweep { x_span } else { y_span };

        let mut idx = 0;

        let mut active_edge: Vec<Edge> = vec![];

        for sweep_line in low..=high {
            let mut new_vec = vec![];

            for mut e in active_edge {
                if e.get_master_pos() > 0 {
                    e.inc_epoch();
                    new_vec.push(e);
                }
            }

            active_edge = new_vec;

            loop {
                if idx >= edges.len() {
                    break;
                }
                let line = if horizental_sweep {
                    (edges[idx].0).0
                } else {
                    (edges[idx].0).1
                };
                if line > sweep_line {
                    break;
                }

                let edge_obj = if horizental_sweep {
                    Edge::horizental_sweep(edges[idx].0, edges[idx].1)
                } else {
                    Edge::vertical_sweep(edges[idx].0, edges[idx].1)
                };

                if let Some(edge_obj) = edge_obj {
                    active_edge.push(edge_obj);
                }

                idx += 1;
            }

            active_edge.sort();

            let mut first = None;
            let mut second = None;

            for idx in 0..active_edge.len() {
                if first.is_none() {
                    first = Some(active_edge[idx].clone())
                } else if second.is_none() {
                    second = Some(active_edge[idx].clone())
                }

                if let Some(a) = first.clone() {
                    if let Some(b) = second.clone() {
                        if a.get_master_pos() == 0 && b.get_master_pos() != 0 {
                            first = Some(b);
                            second = None;
                            continue;
                        }

                        if a.get_master_pos() != 0 && b.get_master_pos() == 0 {
                            first = Some(a);
                            second = None;
                            continue;
                        }

                        let from = a.get_slave_pos();
                        let to = b.get_slave_pos();

                        if a.get_master_pos() == 0 && b.get_master_pos() == 0 && to - from > 1.0 {
                            first = None;
                            second = None;
                            continue;
                        }

                        if horizental_sweep {
                            back.draw_line(
                                (sweep_line, from.ceil() as i32),
                                (sweep_line, to.floor() as i32),
                                &style.as_color(),
                            )?;
                            back.draw_pixel(
                                (sweep_line, from.floor() as i32),
                                &style.as_color().mix(from.ceil() - from),
                            )?;
                            back.draw_pixel(
                                (sweep_line, to.ceil() as i32),
                                &style.as_color().mix(to - to.floor()),
                            )?;
                        } else {
                            back.draw_line(
                                (from.ceil() as i32, sweep_line),
                                (to.floor() as i32, sweep_line),
                                &style.as_color(),
                            )?;
                            back.draw_pixel(
                                (from.floor() as i32, sweep_line),
                                &style.as_color().mix(from.ceil() - from),
                            )?;
                            back.draw_pixel(
                                (to.ceil() as i32, sweep_line),
                                &style.as_color().mix(to.floor() - to),
                            )?;
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
