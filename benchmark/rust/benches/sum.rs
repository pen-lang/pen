use criterion::{criterion_group, criterion_main, Bencher, Criterion};

const ITERATION_COUNT: usize = 100000;

fn generate_numbers() -> Vec<f64> {
    (0..ITERATION_COUNT).map(|key| (key as f64)).collect()
}

fn sum(bencher: &mut Bencher) {
    let numbers = generate_numbers();

    bencher.iter(|| {
        let mut sum = 0.0;

        for number in &numbers {
            sum += number;
        }
    });
}

fn benchmark(criterion: &mut Criterion) {
    criterion.bench_function("sum", sum);
}

criterion_group!(benchmark_group, benchmark);
criterion_main!(benchmark_group);
