use crate::chart::{axes3d::Axes3dStyle, ChartContext};
use crate::coord::{
    cartesian::Cartesian3d,
    ranged1d::{Ranged, ValueFormatter},
    ranged3d::{ProjectionMatrix, ProjectionMatrixBuilder},
};
use plotters_backend::DrawingBackend;

mod draw_impl;

#[derive(Clone, Debug)]
pub(crate) enum Coord3D<X, Y, Z> {
    X(X),
    Y(Y),
    Z(Z),
}

impl<X, Y, Z> Coord3D<X, Y, Z> {
    fn get_x(&self) -> &X {
        match self {
            Coord3D::X(ret) => ret,
            _ => panic!("Invalid call!"),
        }
    }
    fn get_y(&self) -> &Y {
        match self {
            Coord3D::Y(ret) => ret,
            _ => panic!("Invalid call!"),
        }
    }
    fn get_z(&self) -> &Z {
        match self {
            Coord3D::Z(ret) => ret,
            _ => panic!("Invalid call!"),
        }
    }

    fn build_coord([x, y, z]: [&Self; 3]) -> (X, Y, Z)
    where
        X: Clone,
        Y: Clone,
        Z: Clone,
    {
        (x.get_x().clone(), y.get_y().clone(), z.get_z().clone())
    }
}

impl<'a, DB, X, Y, Z, XT, YT, ZT> ChartContext<'a, DB, Cartesian3d<X, Y, Z>>
where
    DB: DrawingBackend,
    X: Ranged<ValueType = XT> + ValueFormatter<XT>,
    Y: Ranged<ValueType = YT> + ValueFormatter<YT>,
    Z: Ranged<ValueType = ZT> + ValueFormatter<ZT>,
{
    /**
    Create an axis configuration object, to set line styles, labels, sizes, etc.

    # Example

    ```
    # use plotters::prelude::*;
    let drawing_area = BitMapBackend::new("configure_axes.png", (300, 200)).into_drawing_area();
    drawing_area.fill(&WHITE);
    let mut chart_builder = ChartBuilder::on(&drawing_area);
    let mut chart_context = chart_builder.build_cartesian_3d(0.0..4.0, 0.0..4.0, 0.0..4.0).unwrap();
    chart_context.configure_axes().x_labels(0).z_labels(0).draw().unwrap();
    ```

    The result is a chart with no labels in the X or Z axes:

    ![](https://cdn.jsdelivr.net/gh/facorread/plotters-doc-data@apidoc/apidoc/configure_axes.png)

    # See also

    [`ChartContext::configure_mesh()`], a similar function for 2D plots
    */
    pub fn configure_axes(&mut self) -> Axes3dStyle<'a, '_, X, Y, Z, DB> {
        Axes3dStyle::new(self)
    }
}

impl<'a, DB, X: Ranged, Y: Ranged, Z: Ranged> ChartContext<'a, DB, Cartesian3d<X, Y, Z>>
where
    DB: DrawingBackend,
{
    /// Override the 3D projection matrix. This function allows to override the default projection
    /// matrix.
    /// - `pf`: A function that takes the default projection matrix configuration and returns the
    /// projection matrix. This function will allow you to adjust the pitch, yaw angle and the
    /// centeral point of the projection, etc. You can also build a projection matrix which is not
    /// relies on the default configuration as well.
    pub fn with_projection<P: FnOnce(ProjectionMatrixBuilder) -> ProjectionMatrix>(
        &mut self,
        pf: P,
    ) -> &mut Self {
        let (actual_x, actual_y) = self.drawing_area.get_pixel_range();
        self.drawing_area
            .as_coord_spec_mut()
            .set_projection(actual_x, actual_y, pf);
        self
    }

    pub fn set_3d_pixel_range(&mut self, size: (i32, i32, i32)) -> &mut Self {
        let (actual_x, actual_y) = self.drawing_area.get_pixel_range();
        self.drawing_area
            .as_coord_spec_mut()
            .set_coord_pixel_range(actual_x, actual_y, size);
        self
    }
}
