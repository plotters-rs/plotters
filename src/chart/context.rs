mod chart_context;
pub use chart_context::ChartContext;

pub(super) mod cartesian2d;
pub(super) mod cartesian3d;

pub(super) use cartesian3d::Coord3D;


#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_chart_context() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});

        drawing_area.fill(&WHITE).expect("Fill");

        let mut chart = ChartBuilder::on(&drawing_area)
            .caption("Test Title", ("serif", 10))
            .x_label_area_size(20)
            .y_label_area_size(20)
            .set_label_area_size(LabelAreaPosition::Top, 20)
            .set_label_area_size(LabelAreaPosition::Right, 20)
            .build_cartesian_2d(0..10, 0..10)
            .expect("Create chart")
            .set_secondary_coord(0.0..1.0, 0.0..1.0);

        chart
            .configure_mesh()
            .x_desc("X")
            .y_desc("Y")
            .draw()
            .expect("Draw mesh");
        chart
            .configure_secondary_axes()
            .x_desc("X")
            .y_desc("Y")
            .draw()
            .expect("Draw Secondary axes");

        chart
            .draw_series(std::iter::once(Circle::new((5, 5), 5, &RED)))
            .expect("Drawing error");
        chart
            .draw_secondary_series(std::iter::once(Circle::new((0.3, 0.8), 5, &GREEN)))
            .expect("Drawing error")
            .label("Test label")
            .legend(|(x, y)| Rectangle::new([(x - 10, y - 5), (x, y + 5)], &GREEN));

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperMiddle)
            .draw()
            .expect("Drawing error");
    }

    #[test]
    fn test_chart_context_3d() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});

        drawing_area.fill(&WHITE).expect("Fill");

        let mut chart = ChartBuilder::on(&drawing_area)
            .caption("Test Title", ("serif", 10))
            .x_label_area_size(20)
            .y_label_area_size(20)
            .set_label_area_size(LabelAreaPosition::Top, 20)
            .set_label_area_size(LabelAreaPosition::Right, 20)
            .build_cartesian_3d(0..10, 0..10, 0..10)
            .expect("Create chart");

        chart.with_projection(|mut pb| {
            pb.yaw = 0.5;
            pb.pitch = 0.5;
            pb.scale = 0.5;
            pb.into_matrix()
        });

        chart.configure_axes().draw().expect("Drawing axes");

        chart
            .draw_series(std::iter::once(Circle::new((5, 5, 5), 5, &RED)))
            .expect("Drawing error");
    }
}
