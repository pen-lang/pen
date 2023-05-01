use regex::Regex;
use std::{error::Error, str, sync::Arc};

#[ffi::into_any(into_fn = "_pen_regex_expression_to_any")]
#[repr(C)]
struct Expression(ffi::Arc<ffi::Any>);

#[ffi::any]
#[derive(Clone)]
struct ExpressionInner(Arc<Regex>);

impl Expression {
    fn as_regex(&self) -> &Regex {
        let inner: &ExpressionInner = TryFrom::try_from(&*self.0).unwrap();

        &inner.0
    }
}

#[ffi::bindgen]
fn _pen_regex_expression_new(string: ffi::ByteString) -> Result<Expression, Box<dyn Error>> {
    Ok(Expression(ffi::Arc::new(
        ExpressionInner(Arc::new(Regex::new(str::from_utf8(string.as_slice())?)?)).into(),
    )))
}

#[ffi::bindgen]
fn _pen_regex_expression_matches(expression: Expression, string: ffi::ByteString) -> ffi::Boolean {
    if let Ok(string) = str::from_utf8(string.as_slice()) {
        expression.as_regex().is_match(string)
    } else {
        false
    }
    .into()
}

#[ffi::bindgen]
fn _pen_regex_expression_match(expression: Expression, string: ffi::ByteString) -> ffi::List {
    if let Ok(string) = str::from_utf8(string.as_slice()) {
        if let Some(matches) = expression.as_regex().captures(string) {
            matches
                .iter()
                .map(|capture| {
                    if let Some(capture) = capture {
                        ffi::ByteString::from(capture.as_str()).into()
                    } else {
                        ffi::None::new().into()
                    }
                })
                .collect::<Vec<ffi::Any>>()
                .into()
        } else {
            ffi::List::new()
        }
    } else {
        ffi::List::new()
    }
}
