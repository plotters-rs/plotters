pub trait ColorScale<ColorType: crate::prelude::Color, FloatType=f32>
where
    FloatType: Float,
{
    fn get_color(&self, h: FloatType) -> ColorType {
        self.get_color_normalized(h, FloatType::zero(), FloatType::one())
    }

    fn get_color_normalized(&self, h: FloatType, min: FloatType, max: FloatType) -> ColorType;
}
