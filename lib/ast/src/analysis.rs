pub fn is_name_public(name: &str) -> bool {
    name.chars()
        .next()
        .map(|character| character.is_ascii_uppercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_public_name() {
        assert!(is_name_public("Foo"));
        assert!(!is_name_public("foo"));
    }
}
