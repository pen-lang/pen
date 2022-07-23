use crate::ir::Document;
use std::iter::repeat;

struct Context {
    outputs: Vec<String>,
    // Omit extra indent output so that we do not need to remove them later.
    next_level: usize,
    line_suffixes: Vec<String>,
}

pub fn format(document: &Document) -> String {
    let mut context = Context {
        outputs: vec![],
        next_level: 0,
        line_suffixes: vec![],
    };

    format_document(&mut context, document, 0, true);

    context.outputs.concat()
}

fn format_document(context: &mut Context, document: &Document, level: usize, broken: bool) {
    match document {
        Document::Break(broken, document) => format_document(context, document, level, *broken),
        Document::Indent(document) => format_document(context, document, level + 1, broken),
        Document::Line => {
            if broken {
                format_line(context, level);
            } else {
                context.outputs.extend([" ".into()]);
            }
        }
        Document::LineSuffix(suffix) => {
            if !suffix.is_empty() {
                flush(context);
            }

            context.line_suffixes.push(suffix.clone());
        }
        Document::Sequence(documents) => {
            for document in documents.as_ref() {
                format_document(context, document, level, broken);
            }
        }
        Document::String(string) => {
            if !string.is_empty() {
                flush(context);
            }

            context.outputs.push(string.clone());
        }
    }
}

fn format_line(context: &mut Context, level: usize) {
    context
        .outputs
        .extend(context.line_suffixes.drain(..).chain(["\n".into()]));

    context.next_level = level;
}

fn flush(context: &mut Context) {
    context
        .outputs
        .extend(repeat("  ".into()).take(context.next_level));
    context.next_level = 0;
}

#[cfg(test)]
mod tests {
    use super::{super::build::*, *};
    use indoc::indoc;

    #[test]
    fn format_string() {
        assert_eq!(format(&"foo".into()), "foo");
    }

    mod group {
        use super::*;

        fn create_group() -> Document {
            vec![
                "{".into(),
                indent(vec![line(), "foo".into(), line(), "bar".into()]),
                line(),
                "}".into(),
            ]
            .into()
        }

        #[test]
        fn format_flat_group() {
            assert_eq!(format(&flatten(create_group())), "{ foo bar }");
        }

        #[test]
        fn format_empty_line_with_indent() {
            assert_eq!(format(&indent(line())), "\n");
        }

        #[test]
        fn format_broken_group() {
            assert_eq!(
                format(&create_group()),
                indoc!(
                    "
                    {
                      foo
                      bar
                    }
                    "
                )
                .trim(),
            );
        }

        #[test]
        fn format_unbroken_group_in_broken_group() {
            assert_eq!(
                format(
                    &vec![
                        "{".into(),
                        indent(vec![line(), flatten(create_group())]),
                        line(),
                        "}".into(),
                    ]
                    .into()
                ),
                indoc!(
                    "
                    {
                      { foo bar }
                    }
                    "
                )
                .trim(),
            );
        }
    }

    mod line_suffix {
        use super::*;

        #[test]
        fn format_line_suffix_between_strings() {
            assert_eq!(
                format(&vec!["{".into(), line_suffix("foo"), "}".into(), line()].into()),
                "{}foo\n",
            );
        }

        #[test]
        fn format_two_line_suffixes_between_strings() {
            assert_eq!(
                format(
                    &vec![
                        "{".into(),
                        line_suffix("foo"),
                        line_suffix("bar"),
                        "}".into(),
                        line()
                    ]
                    .into()
                ),
                "{}foobar\n",
            );
        }
    }
}
