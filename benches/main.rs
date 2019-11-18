use criterion::criterion_main;

mod benches;

criterion_main! {
    benches::parallel::parallel_group,
    benches::rasterizer::rasterizer_group,
    benches::data::quartiles_group
}
