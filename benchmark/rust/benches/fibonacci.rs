use criterion::{criterion_group, criterion_main, Bencher, Criterion};

fn fibonacci(n: f64) -> f64 {
    if n <= 0.0 {
        0.0
    } else if n == 1.0 {
        1.0
    } else {
        fibonacci(n - 1.0) + fibonacci(n - 2.0)
    }
}

fn run_fibonacci(bencher: &mut Bencher) {
    bencher.iter(|| fibonacci(40.0));
}

fn benchmark(criterion: &mut Criterion) {
    criterion.bench_function("fibonacci", run_fibonacci);
}

criterion_group!(benchmark_group, benchmark);
criterion_main!(benchmark_group);
