fn main() {
    dbg!(fibonacci(40.0));
}

fn fibonacci(n: f64) -> f64 {
    if n <= 0.0 {
        0.0
    } else if n == 1.0 {
        1.0
    } else {
        fibonacci(n - 1.0) + fibonacci(n - 2.0)
    }
}
