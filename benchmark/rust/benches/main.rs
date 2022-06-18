use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use ordered_float::OrderedFloat;
use std::collections::HashMap;

const INSERT_COUNT: usize = 100000;

fn generate_insert_keys() -> Vec<OrderedFloat<f64>> {
    (0..INSERT_COUNT).map(|key| (key as f64).into()).collect()
}

fn im_hash_map_insert(bencher: &mut Bencher) {
    let keys = generate_insert_keys();

    bencher.iter(|| {
        let mut map = im::HashMap::new();

        for key in &keys {
            map.insert(key, ());
        }
    });
}

fn im_hash_map_update(bencher: &mut Bencher) {
    let keys = generate_insert_keys();

    bencher.iter(|| {
        let mut map = im::HashMap::new();

        for key in &keys {
            map = map.update(key, ());
        }
    });
}

fn std_hash_map_insert(bencher: &mut Bencher) {
    let keys = generate_insert_keys();

    bencher.iter(|| {
        let mut map = HashMap::new();

        for key in &keys {
            map.insert(key, ());
        }
    });
}

fn benchmark(criterion: &mut Criterion) {
    criterion.bench_function("im_hash_map_insert", im_hash_map_insert);
    criterion.bench_function("im_hash_map_update", im_hash_map_update);

    criterion.bench_function("std_hash_map_insert", std_hash_map_insert);
}

criterion_group!(benchmark_group, benchmark);
criterion_main!(benchmark_group);
