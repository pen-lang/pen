pub fn qualify(prefix: &str, name: &str) -> String {
    prefix.to_owned() + ast::IDENTIFIER_SEPARATOR + name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualify_name() {
        assert_eq!(qualify("foo", "bar"), "foo'bar");
    }
}
