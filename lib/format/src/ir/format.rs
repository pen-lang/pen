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
        Document::SoftLine => {
            if broken {
                format_document(context, &Document::HardLine, level, broken);
            } else {
                context.outputs.extend([" ".into()]);
            }
        }
        Document::HardLine => {
            context.outputs.extend(
                [context.comments.join(" ").into(), "\n".into()]
                    .into_iter()
                    .chain(repeat("  ".into()).take(level)),
            );
            context.comments.clear();
        }
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
                indent(vec![soft_line(), "foo".into(), soft_line(), "bar".into()]),
                soft_line(),
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
                        indent(vec![soft_line(), flatten(create_group())]),
                        soft_line(),
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
}
