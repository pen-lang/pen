use crate::ir::Document;
use std::iter::repeat;

struct Context {
    outputs: Vec<String>,
    comments: Vec<String>,
}

pub fn format(document: &Document) -> String {
    let mut context = Context {
        outputs: vec![],
        comments: vec![],
    };

    format_document(&mut context, &document, 0, true);

    context.outputs.concat()
}

fn format_document(context: &mut Context, document: &Document, level: usize, broken: bool) {
    match document {
        Document::Comment(comment) => context.comments.push(comment.clone()),
        Document::Sequence(documents) => {
            for document in documents {
                format_document(context, document, level, broken);
            }
        }
        Document::Flatten(document) => format_document(context, document, level, false),
        Document::Indent(document) => format_document(context, document, level + 1, broken),
        Document::Line => {
            if broken {
                context.outputs.extend(
                    if context.comments.is_empty() {
                        None
                    } else {
                        Some(" # ".to_owned() + &context.comments.join(" "))
                    }
                    .into_iter()
                    .chain(["\n".into()])
                    .chain(repeat("  ".into()).take(level)),
                );
                context.comments.clear();
            } else {
                context.outputs.extend([" ".into()]);
            }
        }
        Document::Break => {}
        Document::String(string) => context.outputs.push(string.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::build::*;
    use super::*;
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
        fn format_broken_group() {
            assert_eq!(
                format(&create_group().into()),
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

    mod comment {
        use super::*;

        #[test]
        fn format_comment_between_strings() {
            assert_eq!(
                format(&vec!["{".into(), comment("foo"), "}".into(), line()].into()),
                "{} # foo\n",
            );
        }
    }
}
