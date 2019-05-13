mod mandelbrot;
mod func_plot;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub use func_plot::draw_func;
pub use mandelbrot::draw_mandelbrot;
