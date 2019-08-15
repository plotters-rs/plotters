use num_traits::Float;

use palette::encoding::Linear;
use palette::luma::{Luma, LumaStandard};
use palette::rgb::RgbStandard;
use palette::rgb::{Rgb, RgbSpace};
use palette::white_point::D65;
use palette::{Alpha, Component, Hsl, Hsv, Hwb, Lab, Lch, LinSrgb, Xyz, Yxy};

use super::color::Color;

impl<S: RgbStandard, T: Component> Color for Rgb<S, T> {
    fn rgb(&self) -> (u8, u8, u8) {
        self.into_format::<u8>().into_components()
    }

    #[inline]
    fn alpha(&self) -> f64 {
        1.0
    }
}

impl<S: LumaStandard, T: Component> Color for Luma<S, T> {
    fn rgb(&self) -> (u8, u8, u8) {
        let (luma,) = self.into_format::<u8>().into_components();
        (luma, luma, luma)
    }

    #[inline]
    fn alpha(&self) -> f64 {
        1.0
    }
}

impl<S: RgbSpace, T: Component + Float> Color for Hsl<S, T> {
    fn rgb(&self) -> (u8, u8, u8) {
        Rgb::<Linear<S>, T>::from(*self)
            .into_format::<u8>()
            .into_components()
    }

    #[inline]
    fn alpha(&self) -> f64 {
        1.0
    }
}

impl<S: RgbSpace, T: Component + Float> Color for Hsv<S, T> {
    fn rgb(&self) -> (u8, u8, u8) {
        Rgb::<Linear<S>, T>::from(*self)
            .into_format::<u8>()
            .into_components()
    }

    #[inline]
    fn alpha(&self) -> f64 {
        1.0
    }
}

impl<S: RgbSpace, T: Component + Float> Color for Hwb<S, T> {
    fn rgb(&self) -> (u8, u8, u8) {
        Rgb::<Linear<S>, T>::from(*self)
            .into_format::<u8>()
            .into_components()
    }

    #[inline]
    fn alpha(&self) -> f64 {
        1.0
    }
}

impl<T: Component + Float> Color for Lab<D65, T> {
    fn rgb(&self) -> (u8, u8, u8) {
        LinSrgb::<T>::from(*self)
            .into_format::<u8>()
            .into_components()
    }

    #[inline]
    fn alpha(&self) -> f64 {
        1.0
    }
}

impl<T: Component + Float> Color for Lch<D65, T> {
    fn rgb(&self) -> (u8, u8, u8) {
        LinSrgb::<T>::from(*self)
            .into_format::<u8>()
            .into_components()
    }

    #[inline]
    fn alpha(&self) -> f64 {
        1.0
    }
}

impl<T: Component + Float> Color for Xyz<D65, T> {
    fn rgb(&self) -> (u8, u8, u8) {
        LinSrgb::<T>::from(*self)
            .into_format::<u8>()
            .into_components()
    }

    #[inline]
    fn alpha(&self) -> f64 {
        1.0
    }
}

impl<T: Component + Float> Color for Yxy<D65, T> {
    fn rgb(&self) -> (u8, u8, u8) {
        LinSrgb::<T>::from(*self)
            .into_format::<u8>()
            .into_components()
    }

    #[inline]
    fn alpha(&self) -> f64 {
        1.0
    }
}

impl<C: Color, T: Component> Color for Alpha<C, T> {
    #[inline]
    fn rgb(&self) -> (u8, u8, u8) {
        self.color.rgb()
    }

    #[inline]
    fn alpha(&self) -> f64 {
        self.alpha.convert()
    }
}
