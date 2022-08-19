use ordered_float::OrderedFloat;

fn main() {
    let mut map = im::HashMap::new();

    for key in 0..100_000 {
        map = map.update(OrderedFloat::from(key as f64), ());
    }

    dbg!(map.len());
}
