use crate::ir::*;
use std::iter::repeat;

struct Context {
    outputs: Vec<String>,
}

pub fn generate(section: &Section) -> String {
    let mut context = Context { outputs: vec![] };

    generate_section(&mut context, section, 0);

    context.outputs.concat()
}

fn generate_section(context: &mut Context, section: &Section, level: usize) {
    context.outputs.extend(repeat("#".into()).take(level + 1));
    context.outputs.push(" ".into());
    generate_text(context, &section.title);
    generate_line(context);

    for text in &section.paragraphs {
        generate_line(context);
        generate_text(context, text);
        generate_line(context);
    }

    for section in &section.children {
        generate_line(context);
        generate_section(context, section, level + 1);
    }
}

fn generate_text(context: &mut Context, text: &Text) {
    for span in &text.spans {
        generate_span(context, span);
    }
}

fn generate_span(context: &mut Context, span: &Span) {
    match span {
        Span::Normal(string) => context.outputs.push(string.into()),
        Span::Code(string) => context
            .outputs
            .extend(["`".into(), string.into(), "`".into()]),
    }
}

fn generate_line(context: &mut Context) {
    context.outputs.push("\n".into());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::build::*;
    use indoc::indoc;

    #[test]
    fn generate_title() {
        assert_eq!(
            generate(&section(text([normal("foo")]), [], [])),
            indoc!(
                "
                    # foo
                    "
            )
        );
    }

    #[test]
    fn generate_paragraph() {
        assert_eq!(
            generate(&section(
                text([normal("foo")]),
                [text([normal("I'm a programmer.")])],
                []
            )),
            indoc!(
                "
                    # foo

                    I'm a programmer.
                    "
            )
        );
    }

    #[test]
    fn generate_paragraphs() {
        assert_eq!(
            generate(&section(
                text([normal("foo")]),
                [
                    text([normal("I'm a programmer.")]),
                    text([normal("I'm a chef.")])
                ],
                []
            )),
            indoc!(
                "
                # foo

                I'm a programmer.

                I'm a chef.
                "
            )
        );
    }

    #[test]
    fn generate_paragraph_with_text_and_code() {
        assert_eq!(
            generate(&section(
                text([normal("foo")]),
                [text([normal("I'm a "), code("programmer"), normal(".")])],
                []
            )),
            indoc!(
                "
                # foo

                I'm a `programmer`.
                "
            )
        );
    }

    #[test]
    fn generate_paragraph_of_code() {
        assert_eq!(
            generate(&section(
                text([normal("foo")]),
                [text([code("I'm a programmer.")])],
                []
            )),
            indoc!(
                "
                # foo

                `I'm a programmer.`
                "
            )
        );
    }

    #[test]
    fn generate_child_section() {
        assert_eq!(
            generate(&section(
                text([normal("foo")]),
                [],
                [section(text([normal("bar")]), [], [])]
            )),
            indoc!(
                "
                # foo

                ## bar
                "
            )
        );
    }

    #[test]
    fn generate_child_sections() {
        assert_eq!(
            generate(&section(
                text([normal("foo")]),
                [],
                [
                    section(text([normal("bar")]), [], []),
                    section(text([normal("baz")]), [], [])
                ]
            )),
            indoc!(
                "
                # foo

                ## bar

                ## baz
                "
            )
        );
    }

    #[test]
    fn generate_nested_section() {
        assert_eq!(
            generate(&section(
                text([normal("foo")]),
                [],
                [section(
                    text([normal("bar")]),
                    [],
                    [section(text([normal("baz")]), [], [])]
                )]
            )),
            indoc!(
                "
                # foo

                ## bar

                ### baz
                "
            )
        );
    }
}
