use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};

const ITERATION_COUNT: usize = 100_000_000;

fn generate_numbers() -> impl Iterator<Item = f64> + Clone {
    (0..ITERATION_COUNT).map(|key| (key as f64))
}

fn sum(bencher: &mut Bencher) {
    let numbers = generate_numbers();

    bencher.iter(|| {
        let mut sum = 0.0;

        for number in numbers.clone() {
            sum = black_box(sum + number);
        }

        let _ = sum;
    });
}

fn benchmark(criterion: &mut Criterion) {
    criterion.bench_function("sum", sum);
}

criterion_group!(benchmark_group, benchmark);
criterion_main!(benchmark_group);
