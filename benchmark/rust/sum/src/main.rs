fn main() {
    let mut sum = 0.0;

    for i in 0..=100000000 {
        sum += i as f64;
    }

    dbg!(sum);
}
