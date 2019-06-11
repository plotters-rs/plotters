use wasm_bindgen::prelude::*;

mod func_plot;
mod mandelbrot;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub use func_plot::draw_func;
pub use mandelbrot::draw_mandelbrot;

pub fn make_coord_mapping_closure<T: Into<f64> + 'static>(
    map_func: Option<Box<dyn Fn((i32, i32)) -> Option<(T, T)>>>,
) -> JsValue {
    if let Some(mapping_func) = map_func {
        let closure = Closure::wrap(Box::new(move |x: i32, y: i32, idx: u32| {
            if let Some((x, y)) = mapping_func((x, y)) {
                if idx == 0 {
                    return x.into();
                }
                return y.into();
            } else {
                return std::f64::NAN;
            }
        }) as Box<dyn FnMut(i32, i32, u32) -> f64>);

        let js_value = closure.as_ref().clone();
        closure.forget();

        return js_value;
    } else {
        return JsValue::null();
    }
}
