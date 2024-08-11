use regex::Regex;
use std::sync::LazyLock;

const REPLACEMENT_STRING: &str = "_";

static REPLACEMENT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[.:/]+").unwrap());

pub fn calculate(url: &url::Url) -> String {
    REPLACEMENT_REGEX
        .replace_all(&format!("{url}"), REPLACEMENT_STRING)
        .into()
}
