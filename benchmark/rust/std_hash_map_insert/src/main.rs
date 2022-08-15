use ordered_float::OrderedFloat;
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();

    for key in 0..100_000 {
        map.insert(OrderedFloat::from(key as f64), ());
    }

    dbg!(map.len());
}
