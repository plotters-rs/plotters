use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingErrorKind};
use crate::drawing::DrawingBackend;

use crate::style::Color;

use std::cmp::{Ord, Ordering, PartialOrd};

#[derive(Clone, Debug)]
struct Edge {
    master_pos: i32,
    slave_pos: f64,
    slave_step: f64,
}

impl Edge {
    fn horizental_sweep(mut from: BackendCoord, mut to: BackendCoord) -> Option<Edge> {
        if from.0 == to.0 {
            return None;
        }

        if from.0 > to.0 {
            std::mem::swap(&mut from, &mut to);
        }

        let d = (to.1 - from.1) as f64 / (to.0 - from.0) as f64;

        Some(Edge {
            master_pos: to.0 - from.0,
            slave_pos: from.1 as f64,
            slave_step: d,
        })
    }

    fn vertical_sweep(from: BackendCoord, to: BackendCoord) -> Option<Edge> {
        Edge::horizental_sweep((from.1, from.0), (to.1, to.0))
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.slave_pos.partial_cmp(&other.slave_pos)
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.slave_pos == other.slave_pos
    }
}

impl Eq for Edge {}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.slave_pos.partial_cmp(&other.slave_pos).unwrap()
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
                if e.master_pos > 1 {
                    e.master_pos -= 1;
                    e.slave_pos += e.slave_step;
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
                } else {
                    back.draw_line(edges[idx].0, edges[idx].1, style)?;
                }

                idx += 1;
            }

            active_edge.sort();

            for idx in 0..(active_edge.len() / 2) {
                let (a, b) = (
                    active_edge[idx * 2].clone(),
                    active_edge[idx * 2 + 1].clone(),
                );

                let from = a.slave_pos;
                let to = b.slave_pos;

                if horizental_sweep {
                    back.draw_line(
                        (sweep_line, from.ceil() as i32),
                        (sweep_line, to.floor() as i32),
                        style,
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
                        style,
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
            }
        }
    }

    Ok(())
}
