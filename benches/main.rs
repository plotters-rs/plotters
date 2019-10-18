use criterion::criterion_main;

mod benches;

criterion_main! {
    benches::parallel::parallel_group
}
