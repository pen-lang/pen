use std::sync::LazyLock;
use regex::bytes::{Captures, Regex};
use std::str;

static HEX_CHARACTER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\x([0-9a-fA-F][0-9a-fA-F])").unwrap());

pub fn compile(string: &str) -> Vec<u8> {
    HEX_CHARACTER_REGEX
        .replace_all(
            string
                .replace("\\\\", "\\")
                .replace("\\\"", "\"")
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
                .as_bytes(),
            |captures: &Captures| {
                vec![u8::from_str_radix(str::from_utf8(&captures[1]).unwrap(), 16).unwrap()]
            },
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_back_slash() {
        assert_eq!(compile("\\\\"), Vec::from("\\"));
    }

    #[test]
    fn compile_double_quote() {
        assert_eq!(compile("\\\""), Vec::from("\""));
    }

    #[test]
    fn compile_newline() {
        assert_eq!(compile("\\n"), Vec::from("\n"));
    }

    #[test]
    fn compile_newlines() {
        assert_eq!(compile("\\n\\n"), Vec::from("\n\n"));
    }

    #[test]
    fn compile_carriage_return() {
        assert_eq!(compile("\\r"), Vec::from("\r"));
    }

    #[test]
    fn compile_tab() {
        assert_eq!(compile("\\t"), Vec::from("\t"));
    }

    #[test]
    fn compile_byte_escape() {
        assert_eq!(compile("\\x42"), vec![0x42]);
    }

    #[test]
    fn compile_sequence_of_byte_escape() {
        assert_eq!(compile("\\x01\\x02\\x03"), vec![1, 2, 3]);
    }

    #[test]
    fn compile_byte_escape_invalid_as_utf8() {
        assert_eq!(compile("\\x80"), vec![0x80]);
    }
}
