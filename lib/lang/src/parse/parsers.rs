use super::{
    attempt::{many, many1, optional, sep_end_by, sep_end_by1},
    utilities::*,
};
use crate::{
    ast::*,
    position::Position,
    types::{self, Type},
};
use combine::{
    easy, from_str, none_of, one_of,
    parser::{
        char::{alpha_num, char as character, letter, spaces, string},
        combinator::{lazy, look_ahead, no_partial, not_followed_by},
        regex::find,
        sequence::between,
    },
    stream::{
        position::{self, SourcePosition},
        state,
    },
    unexpected_any, value, Parser, Positioned,
};
use once_cell::sync::Lazy;
use std::{collections::HashSet, sync::Arc};

const KEYWORDS: &[&str] = &["else", "if", "import", "type"];
const OPERATOR_CHARACTERS: &str = "+-*/=<>&|";

static NUMBER_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^-?([123456789][0123456789]*|0)(\.[0123456789]+)?").unwrap());
static STRING_REGEX: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r#"^[^\\"]"#).unwrap());

pub struct State<'a> {
    path: &'a str,
    lines: Vec<&'a str>,
}

pub type Stream<'a> =
    easy::Stream<state::Stream<position::Stream<&'a str, SourcePosition>, State<'a>>>;

pub fn stream<'a>(source: &'a str, path: &'a str) -> Stream<'a> {
    state::Stream {
        stream: position::Stream::new(source),
        state: State {
            path,
            lines: source.split('\n').collect(),
        },
    }
    .into()
}

pub fn module<'a>() -> impl Parser<Stream<'a>, Output = Module> {
    (
        spaces(),
        many(import()),
        many(type_definition()),
        many(type_alias()),
        many(definition()),
    )
        .skip(eof())
        .map(
            |(_, imports, type_definitions, type_aliases, definitions)| {
                Module::new(imports, type_definitions, type_aliases, definitions)
            },
        )
}

fn import<'a>() -> impl Parser<Stream<'a>, Output = Import> {
    keyword("import")
        .with(module_path())
        .map(Import::new)
        .expected("import statement")
}

fn module_path<'a>() -> impl Parser<Stream<'a>, Output = ModulePath> {
    token(choice!(
        internal_module_path().map(ModulePath::from),
        external_module_path().map(ModulePath::from),
    ))
    .expected("module path")
}

fn internal_module_path<'a>() -> impl Parser<Stream<'a>, Output = InternalModulePath> {
    module_path_components().map(InternalModulePath::new)
}

fn external_module_path<'a>() -> impl Parser<Stream<'a>, Output = ExternalModulePath> {
    (identifier(), module_path_components())
        .map(|(package, components)| ExternalModulePath::new(package, components))
}

fn module_path_components<'a>() -> impl Parser<Stream<'a>, Output = Vec<String>> {
    many1(string(".").with(identifier()))
}

fn definition<'a>() -> impl Parser<Stream<'a>, Output = Definition> {
    (position(), identifier(), sign("="), lambda())
        .map(|(position, name, _, lambda)| Definition::new(name, lambda, position))
}

fn type_definition<'a>() -> impl Parser<Stream<'a>, Output = TypeDefinition> {
    (
        position(),
        keyword("type"),
        identifier(),
        optional(between(
            sign("{"),
            sign("}"),
            sep_end_by1((identifier(), type_()), sign(",")),
        )),
    )
        .map(|(position, _, name, elements): (_, _, _, Option<Vec<_>>)| {
            TypeDefinition::new(
                name,
                elements
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(name, type_)| types::RecordElement::new(name, type_))
                    .collect(),
                position,
            )
        })
        .expected("type definition")
}

fn type_alias<'a>() -> impl Parser<Stream<'a>, Output = TypeAlias> {
    (keyword("type"), identifier(), sign("="), type_())
        .map(|(_, name, _, type_)| TypeAlias::new(name, type_))
        .expected("type alias")
}

fn type_<'a>() -> impl Parser<Stream<'a>, Output = Type> {
    lazy(|| no_partial(choice!(function_type().map(Type::from), union_type())))
        .boxed()
        .expected("type")
}

fn function_type<'a>() -> impl Parser<Stream<'a>, Output = types::Function> {
    (
        position(),
        sign("\\("),
        sep_end_by(type_(), sign(",")),
        sign(")"),
        type_(),
    )
        .map(|(position, _, arguments, _, result)| {
            types::Function::new(arguments, result, position)
        })
        .expected("function type")
}

fn union_type<'a>() -> impl Parser<Stream<'a>, Output = Type> {
    (position(), sep_end_by1(atomic_type(), sign("|")))
        .map(|(position, types): (_, Vec<_>)| {
            types
                .into_iter()
                .reduce(|lhs, rhs| {
                    types::Union::new(lhs.clone(), rhs, lhs.position().clone()).into()
                })
                .unwrap()
        })
        .expected("union type")
}

fn list_type<'a>() -> impl Parser<Stream<'a>, Output = types::List> {
    (position(), between(sign("["), sign("]"), type_()))
        .map(|(position, element)| types::List::new(element, position))
        .expected("list type")
}

fn atomic_type<'a>() -> impl Parser<Stream<'a>, Output = Type> {
    choice!(
        boolean_type().map(Type::from),
        none_type().map(Type::from),
        number_type().map(Type::from),
        string_type().map(Type::from),
        any_type().map(Type::from),
        reference_type().map(Type::from),
        list_type().map(Type::from),
        between(sign("("), sign(")"), type_()),
    )
}

fn boolean_type<'a>() -> impl Parser<Stream<'a>, Output = types::Boolean> {
    position()
        .skip(keyword("Boolean"))
        .map(types::Boolean::new)
        .expected("boolean type")
}

fn none_type<'a>() -> impl Parser<Stream<'a>, Output = types::None> {
    position()
        .skip(keyword("None"))
        .map(types::None::new)
        .expected("none type")
}

fn number_type<'a>() -> impl Parser<Stream<'a>, Output = types::Number> {
    position()
        .skip(keyword("Number"))
        .map(types::Number::new)
        .expected("number type")
}

fn string_type<'a>() -> impl Parser<Stream<'a>, Output = types::ByteString> {
    position()
        .skip(keyword("String"))
        .map(types::ByteString::new)
        .expected("string type")
}

fn any_type<'a>() -> impl Parser<Stream<'a>, Output = types::Any> {
    position()
        .skip(keyword("Any"))
        .map(types::Any::new)
        .expected("any type")
}

fn reference_type<'a>() -> impl Parser<Stream<'a>, Output = types::Reference> {
    (position(), qualified_identifier())
        .map(|(position, identifier)| types::Reference::new(identifier, position))
        .expected("reference type")
}

fn block<'a>() -> impl Parser<Stream<'a>, Output = Block> {
    todo!()
}

fn expression<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    lazy(|| no_partial(binary_operation()))
        .boxed()
        .expected("expression")
}

fn atomic_expression<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    lazy(|| {
        no_partial(choice!(
            record().map(Expression::from),
            record_update().map(Expression::from),
            list_literal().map(Expression::from),
            boolean_literal().map(Expression::from),
            none_literal().map(Expression::from),
            number_literal().map(Expression::from),
            string_literal().map(Expression::from),
            variable().map(Expression::from),
            between(sign("("), sign(")"), expression()),
        ))
    })
    .boxed()
}

fn lambda<'a>() -> impl Parser<Stream<'a>, Output = Lambda> {
    todo!()
}

fn if_<'a>() -> impl Parser<Stream<'a>, Output = If> {
    (
        position(),
        keyword("if"),
        expression(),
        block(),
        many((keyword("else"), keyword("if"), expression(), block())),
        keyword("else"),
        block(),
    )
        .map(
            |(position, _, condition, if_block, else_if_blocks, _, else_block)| {
                If::new(condition, then, else_, position)
            },
        )
        .expected("if expression")
}

fn if_list<'a>() -> impl Parser<Stream<'a>, Output = IfList> {
    (
        position(),
        keyword("case").expected("case keyword"),
        expression(),
        sign("[]"),
        sign("=>"),
        expression(),
        sign("["),
        identifier(),
        sign(","),
        sign("..."),
        identifier(),
        sign("]"),
        sign("=>"),
        expression(),
    )
        .map(
            |(
                position,
                _,
                argument,
                _,
                _,
                empty_alternative,
                _,
                first_name,
                _,
                _,
                rest_name,
                _,
                _,
                non_empty_alternative,
            )| {
                IfList::new(
                    argument,
                    first_name,
                    rest_name,
                    non_empty_alternative,
                    empty_alternative,
                    position,
                )
            },
        )
        .expected("list case expression")
}

fn if_type<'a>() -> impl Parser<Stream<'a>, Output = Case> {
    (
        position(),
        keyword("case").expected("case keyword"),
        identifier(),
        sign("="),
        expression(),
        many1(alternative()),
    )
        .map(|(position, _, identifier, _, argument, alternatives)| {
            Case::new(identifier, argument, alternatives, position)
        })
        .expected("type case expression")
}

fn alternative<'a>() -> impl Parser<Stream<'a>, Output = Alternative> {
    (type_(), block()).map(|(type_, block)| Alternative::new(type_, block))
}

fn call<'a>() -> impl Parser<Stream<'a>, Output = Call> {
    (
        position(),
        atomic_expression(),
        between(sign("("), sign(")"), sep_end_by(expression(), sign(","))),
    )
        .map(|(position, function, arguments)| Call::new(function, arguments, position))
        .expected("function call")
}

fn record<'a>() -> impl Parser<Stream<'a>, Output = RecordConstruction> {
    (
        position(),
        reference_type(),
        string("{"),
        sep_end_by1((identifier().skip(sign(":")), expression()), sign(",")),
        sign("}"),
    )
        .then(|(position, reference_type, _, elements, _)| {
            let elements: Vec<_> = elements;

            if elements
                .iter()
                .map(|(key, _)| key.into())
                .collect::<HashSet<String>>()
                .len()
                == elements.len()
            {
                value(RecordConstruction::new(
                    reference_type,
                    elements.into_iter().collect(),
                    position,
                ))
                .left()
            } else {
                unexpected_any("duplicate keys in record construction").right()
            }
        })
        .expected("record")
}

fn record_update<'a>() -> impl Parser<Stream<'a>, Output = RecordUpdate> {
    (
        position(),
        reference_type(),
        string("{"),
        sign("..."),
        atomic_expression(),
        sign(","),
        sep_end_by1((identifier().skip(sign(":")), expression()), sign(",")),
        sign("}"),
    )
        .then(|(position, reference_type, _, _, record, _, elements, _)| {
            let elements: Vec<_> = elements;

            if elements
                .iter()
                .map(|(key, _)| key.into())
                .collect::<HashSet<String>>()
                .len()
                == elements.len()
            {
                value(RecordUpdate::new(
                    reference_type,
                    record,
                    elements.into_iter().collect(),
                    position,
                ))
                .left()
            } else {
                unexpected_any("duplicate keys in record update").right()
            }
        })
}

fn term<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    choice!(
        call(),
        if_().map(Expression::from),
        if_type().map(Expression::from),
        if_list().map(Expression::from),
    )
}

fn binary_operation<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    (
        term(),
        many(
            (position(), binary_operator(), term())
                .map(|(position, operator, expression)| (operator, expression, position)),
        ),
    )
        .map(|(expression, pairs): (_, Vec<_>)| reduce_operations(expression, &pairs))
}

fn binary_operator<'a>() -> impl Parser<Stream<'a>, Output = ParsedOperator> {
    choice!(
        concrete_operator("+", ParsedOperator::Add),
        concrete_operator("-", ParsedOperator::Subtract),
        concrete_operator("*", ParsedOperator::Multiply),
        concrete_operator("/", ParsedOperator::Divide),
        concrete_operator("==", ParsedOperator::Equal),
        concrete_operator("/=", ParsedOperator::NotEqual),
        concrete_operator("<", ParsedOperator::LessThan),
        concrete_operator("<=", ParsedOperator::LessThanOrEqual),
        concrete_operator(">", ParsedOperator::GreaterThan),
        concrete_operator(">=", ParsedOperator::GreaterThanOrEqual),
        concrete_operator("&", ParsedOperator::And),
        concrete_operator("|", ParsedOperator::Or),
    )
    .expected("binary operator")
}

fn concrete_operator<'a>(
    literal: &'static str,
    operator: ParsedOperator,
) -> impl Parser<Stream<'a>, Output = ParsedOperator> {
    token(
        many1(one_of(OPERATOR_CHARACTERS.chars())).then(move |parsed_literal: String| {
            if parsed_literal == literal {
                value(operator).left()
            } else {
                unexpected_any("unknown operator").right()
            }
        }),
    )
}

fn boolean_literal<'a>() -> impl Parser<Stream<'a>, Output = Boolean> {
    token(choice!(
        position()
            .skip(keyword("false"))
            .map(|position| Boolean::new(false, position)),
        position()
            .skip(keyword("true"))
            .map(|position| Boolean::new(true, position)),
    ))
    .expected("boolean literal")
}

fn none_literal<'a>() -> impl Parser<Stream<'a>, Output = None> {
    token(position().skip(keyword("none")))
        .map(None::new)
        .expected("none literal")
}

fn number_literal<'a>() -> impl Parser<Stream<'a>, Output = Number> {
    let regex: &'static regex::Regex = &NUMBER_REGEX;

    token((position(), from_str(find(regex))))
        .map(|(position, number)| Number::new(number, position))
        .expected("number literal")
}

fn string_literal<'a>() -> impl Parser<Stream<'a>, Output = ByteString> {
    let regex: &'static regex::Regex = &STRING_REGEX;

    token((
        position(),
        character('"'),
        many(choice!(
            from_str(find(regex)),
            string("\\\\").map(|_| "\\".into()),
            string("\\\"").map(|_| "\"".into()),
            string("\\n").map(|_| "\n".into()),
            string("\\t").map(|_| "\t".into())
        )),
        character('"'),
    ))
    .map(|(position, _, strings, _): (_, _, Vec<String>, _)| {
        ByteString::new(strings.join(""), position)
    })
    .expected("string literal")
}

fn list_literal<'a>() -> impl Parser<Stream<'a>, Output = List> {
    (
        position(),
        sign("["),
        type_(),
        sign(";"),
        sep_end_by(list_element(), sign(",")),
        sign("]"),
    )
        .map(|(position, _, type_, _, elements, _)| List::new(type_, elements, position))
        .expected("list literal")
}

fn list_element<'a>() -> impl Parser<Stream<'a>, Output = ListElement> {
    (optional(sign("...")), expression()).map(|(ellipsis, expression)| {
        if ellipsis.is_some() {
            ListElement::Multiple(expression)
        } else {
            ListElement::Single(expression)
        }
    })
}

fn variable<'a>() -> impl Parser<Stream<'a>, Output = Variable> {
    token((position(), qualified_identifier()))
        .map(|(position, identifier)| Variable::new(identifier, position))
        .expected("variable")
}

fn qualified_identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    (
        optional(raw_identifier().skip(string("."))),
        raw_identifier(),
    )
        .map(|(prefix, identifier)| {
            prefix
                .map(|prefix| [&prefix, ".", &identifier].concat())
                .unwrap_or(identifier)
        })
}

fn identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    token(raw_identifier()).expected("identifier")
}

fn raw_identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    (letter(), many(alpha_num()))
        .map(|(head, tail): (char, String)| [head.into(), tail].concat())
        .then(|identifier| {
            if KEYWORDS.contains(&identifier.as_str()) {
                unexpected_any("keyword").left()
            } else {
                value(identifier).right()
            }
        })
}

fn keyword<'a>(name: &'static str) -> impl Parser<Stream<'a>, Output = ()> {
    token(string(name).skip(not_followed_by(alpha_num())))
        .with(value(()))
        .expected("keyword")
}

fn sign<'a>(sign: &'static str) -> impl Parser<Stream<'a>, Output = ()> {
    token(string(sign)).with(value(())).expected(sign)
}

fn token<'a, O, P: Parser<Stream<'a>, Output = O>>(p: P) -> impl Parser<Stream<'a>, Output = O> {
    p.skip(spaces())
}

fn position<'a>() -> impl Parser<Stream<'a>, Output = Position> {
    value(())
        .map_input(|_, stream: &mut Stream<'a>| {
            let position = stream.position();

            Position::new(
                stream.0.state.path,
                position.line as usize,
                position.column as usize,
                stream.0.state.lines[position.line as usize - 1],
            )
        })
        .expected("position")
}

fn eof<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    spaces().with(combine::eof()).expected("end of file")
}

fn comment<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    string("#")
        .with(many::<Vec<_>, _, _>(none_of("\n".chars())))
        .with(combine::parser::char::newline())
        .with(spaces())
        .expected("comment")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_module() {
        assert_eq!(
            module().parse(stream("", "")).unwrap().0,
            Module::from_definitions(vec![])
        );
        assert_eq!(
            module().parse(stream(" ", "")).unwrap().0,
            Module::from_definitions(vec![])
        );
        assert_eq!(
            module().parse(stream("\n", "")).unwrap().0,
            Module::from_definitions(vec![])
        );
        assert_eq!(
            module().parse(stream("export { foo }", "")).unwrap().0,
            Module::new(
                Export::new(vec!["foo".into()].drain(..).collect()),
                ExportForeign::new(Default::default()),
                vec![],
                vec![],
                vec![],
                vec![]
            )
        );
        assert_eq!(
            module()
                .parse(stream("export { foo }\nimport Foo.Bar", ""))
                .unwrap()
                .0,
            Module::new(
                Export::new(vec!["foo".into()].drain(..).collect()),
                ExportForeign::new(Default::default()),
                vec![Import::new(ExternalModulePath::new(
                    "Foo",
                    vec!["Bar".into()]
                ))],
                vec![],
                vec![],
                vec![]
            )
        );
        assert_eq!(
            module().parse(stream("x : Number\nx = 42", "")).unwrap().0,
            Module::new(
                Export::new(Default::default()),
                ExportForeign::new(Default::default()),
                vec![],
                vec![],
                vec![],
                vec![VariableDefinition::new(
                    "x",
                    Number::new(42.0, Position::dummy()),
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into()]
            )
        );
        assert_eq!(
            module()
                .parse(stream("x : Number\nx = 42\ny : Number\ny = 42", ""))
                .unwrap()
                .0,
            Module::new(
                Export::new(Default::default()),
                ExportForeign::new(Default::default()),
                vec![],
                vec![],
                vec![],
                vec![
                    VariableDefinition::new(
                        "x",
                        Number::new(42.0, Position::dummy()),
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    )
                    .into(),
                    VariableDefinition::new(
                        "y",
                        Number::new(42.0, Position::dummy()),
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    )
                    .into()
                ]
            )
        );
        assert_eq!(
            module()
                .parse(stream("main : Number -> Number\nmain x = 42", ""))
                .unwrap()
                .0,
            Module::new(
                Export::new(Default::default()),
                ExportForeign::new(Default::default()),
                vec![],
                vec![],
                vec![],
                vec![FunctionDefinition::new(
                    "main",
                    vec!["x".into()],
                    Number::new(42.0, Position::dummy()),
                    types::Function::new(
                        types::Number::new(Position::dummy()),
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into(),]
            )
        );
    }

    #[test]
    fn parse_export() {
        assert!(export().parse(stream("export {}", "")).is_err());
        assert_eq!(
            export().parse(stream("export { foo }", "")).unwrap().0,
            Export::new(vec!["foo".into()].drain(..).collect()),
        );
        assert_eq!(
            export().parse(stream("export { foo, }", "")).unwrap().0,
            Export::new(vec!["foo".into()].drain(..).collect()),
        );
        assert_eq!(
            export().parse(stream("export { foo, bar }", "")).unwrap().0,
            Export::new(vec!["foo".into(), "bar".into()].drain(..).collect()),
        );
        assert_eq!(
            export()
                .parse(stream("export { foo, bar, }", ""))
                .unwrap()
                .0,
            Export::new(vec!["foo".into(), "bar".into()].drain(..).collect()),
        );
        assert_eq!(
            export().parse(stream("export {\nfoo }", "")).unwrap().0,
            Export::new(vec!["foo".into()].drain(..).collect()),
        );
    }

    #[test]
    fn parse_import() {
        assert_eq!(
            import().parse(stream("import .Foo", "")).unwrap().0,
            Import::new(InternalModulePath::new(vec!["Foo".into()])),
        );
        assert_eq!(
            import().parse(stream("import Foo.Bar", "")).unwrap().0,
            Import::new(ExternalModulePath::new("Foo", vec!["Bar".into()])),
        );
    }

    #[test]
    fn parse_module_path() {
        assert!(module_path().parse(stream("?", "")).is_err());
        assert_eq!(
            module_path().parse(stream(".Foo", "")).unwrap().0,
            ModulePath::Internal(InternalModulePath::new(vec!["Foo".into()])),
        );
        assert_eq!(
            module_path().parse(stream("Foo.Bar", "")).unwrap().0,
            ModulePath::External(ExternalModulePath::new("Foo", vec!["Bar".into()])),
        );
        assert_eq!(
            module_path().parse(stream(" .Foo", "")).unwrap().0,
            ModulePath::Internal(InternalModulePath::new(vec!["Foo".into()])),
        );
    }

    #[test]
    fn parse_internal_module_path() {
        assert!(internal_module_path().parse(stream("?", "")).is_err());
        assert_eq!(
            internal_module_path().parse(stream(".Foo", "")).unwrap().0,
            InternalModulePath::new(vec!["Foo".into()]),
        );
        assert_eq!(
            internal_module_path()
                .parse(stream(".Foo.Bar", ""))
                .unwrap()
                .0,
            InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
        );
    }

    #[test]
    fn parse_external_module_path() {
        assert!(external_module_path().parse(stream("?", "")).is_err());
        assert_eq!(
            external_module_path()
                .parse(stream("Foo.Bar", ""))
                .unwrap()
                .0,
            ExternalModulePath::new("Foo", vec!["Bar".into()]),
        );
    }

    #[test]
    fn parse_export_foreign() {
        assert_eq!(
            export_foreign()
                .parse(stream("export foreign { foo }", ""))
                .unwrap()
                .0,
            ExportForeign::new(vec!["foo".into()].into_iter().collect()),
        );
    }

    #[test]
    fn parse_import_foreign() {
        assert_eq!(
            import_foreign()
                .parse(stream("import foreign foo : Number -> Number", ""))
                .unwrap()
                .0,
            ImportForeign::new(
                "foo",
                "foo",
                CallingConvention::Native,
                types::Function::new(
                    types::Number::new(Position::dummy()),
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                ),
                Position::dummy()
            ),
        );
    }

    #[test]
    fn parse_import_foreign_with_calling_convention() {
        assert_eq!(
            import_foreign()
                .parse(stream("import foreign \"c\" foo : Number -> Number", ""))
                .unwrap()
                .0,
            ImportForeign::new(
                "foo",
                "foo",
                CallingConvention::C,
                types::Function::new(
                    types::Number::new(Position::dummy()),
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                ),
                Position::dummy()
            ),
        );
    }

    #[test]
    fn parse_definition() {
        assert_eq!(
            definition()
                .parse(stream("x : Number\nx = 0", ""))
                .unwrap()
                .0,
            VariableDefinition::new(
                "x",
                Number::new(0.0, Position::dummy()),
                types::Number::new(Position::dummy()),
                Position::dummy()
            )
            .into()
        );
        assert_eq!(
            definition()
                .parse(stream("main : Number -> Number\nmain x = 42", ""))
                .unwrap()
                .0,
            FunctionDefinition::new(
                "main",
                vec!["x".into()],
                Number::new(42.0, Position::dummy()),
                types::Function::new(
                    types::Number::new(Position::dummy()),
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                ),
                Position::dummy()
            )
            .into()
        );
    }

    #[test]
    fn parse_variable_definition() {
        assert_eq!(
            variable_definition()
                .parse(stream("x : Number\nx = 0", ""))
                .unwrap()
                .0,
            VariableDefinition::new(
                "x",
                Number::new(0.0, Position::dummy()),
                types::Number::new(Position::dummy()),
                Position::dummy()
            )
        );
    }

    #[test]
    fn parse_untyped_definition() {
        assert_eq!(
            untyped_variable_definition()
                .parse(stream("x = 0", ""))
                .unwrap()
                .0,
            VariableDefinition::new(
                "x",
                Number::new(0.0, Position::dummy()),
                types::Unknown::new(Position::dummy()),
                Position::dummy()
            )
        );
        assert_eq!(
            untyped_function_definition()
                .parse(stream("main x = 42", ""))
                .unwrap()
                .0,
            FunctionDefinition::new(
                "main",
                vec!["x".into()],
                Number::new(42.0, Position::dummy()),
                types::Unknown::new(Position::dummy()),
                Position::dummy()
            )
        );
        assert_eq!(
            (untyped_function_definition(), untyped_variable_definition())
                .parse(stream(
                    indoc!(
                        "
                        f x = x
                         y = (
                             f x
                         )
                        "
                    ),
                    ""
                ))
                .unwrap()
                .0,
            (
                FunctionDefinition::new(
                    "f",
                    vec!["x".into()],
                    Variable::new("x", Position::dummy()),
                    types::Unknown::new(Position::dummy()),
                    Position::dummy()
                ),
                VariableDefinition::new(
                    "y",
                    Call::new(
                        Variable::new("f", Position::dummy()),
                        Variable::new("x", Position::dummy()),
                        Position::dummy()
                    ),
                    types::Unknown::new(Position::dummy()),
                    Position::dummy()
                )
            )
        );
    }

    #[test]
    fn parse_type_definition() {
        for (source, expected) in &[
            (
                "type Foo",
                TypeDefinition::new(
                    "Foo",
                    types::Record::new("Foo", Default::default(), Position::dummy()),
                ),
            ),
            (
                "type Foo ( foo : Number )",
                TypeDefinition::new(
                    "Foo",
                    types::Record::new(
                        "Foo",
                        vec![types::RecordElement::new(
                            "foo",
                            types::Number::new(Position::dummy()),
                        )],
                        Position::dummy(),
                    ),
                ),
            ),
            (
                "type Foo ( foo : Number, )",
                TypeDefinition::new(
                    "Foo",
                    types::Record::new(
                        "Foo",
                        vec![types::RecordElement::new(
                            "foo",
                            types::Number::new(Position::dummy()),
                        )],
                        Position::dummy(),
                    ),
                ),
            ),
            (
                "type Foo ( foo : Number, bar : Number )",
                TypeDefinition::new(
                    "Foo",
                    types::Record::new(
                        "Foo",
                        vec![
                            types::RecordElement::new("foo", types::Number::new(Position::dummy())),
                            types::RecordElement::new("bar", types::Number::new(Position::dummy())),
                        ],
                        Position::dummy(),
                    ),
                ),
            ),
            (
                "type Foo ( foo : Number, bar : Number, )",
                TypeDefinition::new(
                    "Foo",
                    types::Record::new(
                        "Foo",
                        vec![
                            types::RecordElement::new("foo", types::Number::new(Position::dummy())),
                            types::RecordElement::new("bar", types::Number::new(Position::dummy())),
                        ],
                        Position::dummy(),
                    ),
                ),
            ),
            (
                "type Foo = Boolean | None",
                TypeDefinition::new(
                    "Foo",
                    types::Union::new(
                        vec![
                            types::Boolean::new(Position::dummy()).into(),
                            types::None::new(Position::dummy()).into(),
                        ]
                        .into_iter()
                        .collect(),
                        Position::dummy(),
                    ),
                ),
            ),
        ] {
            assert_eq!(
                &type_definition().parse(stream(source, "")).unwrap().0,
                expected
            );
        }
    }

    #[test]
    fn parse_type_alias_definition() {
        for (source, expected) in &[
            (
                "type Foo = Number",
                TypeDefinition::new("Foo", types::Number::new(Position::dummy())),
            ),
            (
                "type Foo = Number | None",
                TypeDefinition::new(
                    "Foo",
                    types::Union::new(
                        vec![
                            types::Number::new(Position::dummy()).into(),
                            types::None::new(Position::dummy()).into(),
                        ]
                        .into_iter()
                        .collect(),
                        Position::dummy(),
                    ),
                ),
            ),
        ] {
            assert_eq!(&type_alias().parse(stream(source, "")).unwrap().0, expected);
        }
    }

    mod types_ {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_type() {
            assert!(type_().parse(stream("?", "")).is_err());
            assert_eq!(
                type_().parse(stream("Boolean", "")).unwrap().0,
                types::Boolean::new(Position::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("None", "")).unwrap().0,
                types::None::new(Position::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("Number", "")).unwrap().0,
                types::Number::new(Position::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("Number -> Number", "")).unwrap().0,
                types::Function::new(
                    types::Number::new(Position::dummy()),
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("Number -> Number -> Number", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    types::Number::new(Position::dummy()),
                    types::Function::new(
                        types::Number::new(Position::dummy()),
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("(Number -> Number) -> Number", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    types::Function::new(
                        types::Number::new(Position::dummy()),
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    ),
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_().parse(stream("Number | None", "")).unwrap().0,
                types::Union::new(
                    vec![
                        types::Number::new(Position::dummy()).into(),
                        types::None::new(Position::dummy()).into(),
                    ],
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("Boolean | Number | None", ""))
                    .unwrap()
                    .0,
                types::Union::new(
                    vec![
                        types::Boolean::new(Position::dummy()).into(),
                        types::Number::new(Position::dummy()).into(),
                        types::None::new(Position::dummy()).into(),
                    ],
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("Number -> Number | None", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    types::Number::new(Position::dummy()),
                    types::Union::new(
                        vec![
                            types::Number::new(Position::dummy()).into(),
                            types::None::new(Position::dummy()).into(),
                        ],
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("Number | None -> Number", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    types::Union::new(
                        vec![
                            types::Number::new(Position::dummy()).into(),
                            types::None::new(Position::dummy()).into(),
                        ],
                        Position::dummy()
                    ),
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("(Number -> Number) | None", ""))
                    .unwrap()
                    .0,
                types::Union::new(
                    vec![
                        types::Function::new(
                            types::Number::new(Position::dummy()),
                            types::Number::new(Position::dummy()),
                            Position::dummy()
                        )
                        .into(),
                        types::None::new(Position::dummy()).into(),
                    ],
                    Position::dummy()
                )
                .into()
            );
        }

        #[test]
        fn parse_any_type() {
            assert_eq!(
                any_type().parse(stream("Any", "")).unwrap().0,
                types::Any::new(Position::dummy())
            );
        }

        #[test]
        fn parse_reference_type() {
            assert!(type_().parse(stream("", "")).is_err());
            assert_eq!(
                type_().parse(stream("Foo", "")).unwrap().0,
                types::Reference::new("Foo", Position::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("Foo.Bar", "")).unwrap().0,
                types::Reference::new("Foo.Bar", Position::dummy()).into()
            );
        }

        #[test]
        fn parse_list_type() {
            assert_eq!(
                type_().parse(stream("List Number", "")).unwrap().0,
                types::List::new(types::Number::new(Position::dummy()), Position::dummy()).into()
            );

            assert_eq!(
                type_().parse(stream("List (List Number)", "")).unwrap().0,
                types::List::new(
                    types::List::new(types::Number::new(Position::dummy()), Position::dummy()),
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                type_()
                    .parse(stream("List Number | List None", ""))
                    .unwrap()
                    .0,
                types::Union::new(
                    vec![
                        types::List::new(types::Number::new(Position::dummy()), Position::dummy())
                            .into(),
                        types::List::new(types::None::new(Position::dummy()), Position::dummy())
                            .into()
                    ],
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                type_()
                    .parse(stream("List Number -> List None", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    types::List::new(types::Number::new(Position::dummy()), Position::dummy()),
                    types::List::new(types::None::new(Position::dummy()), Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
        }
    }

    mod expressions {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_expression() {
            assert!(expression().parse(stream("?", "")).is_err());
            assert!(expression()
                .skip(eof())
                .parse(stream("Foo () foo", ""))
                .is_err());
            assert!(expression()
                .skip(eof())
                .parse(stream("Foo ( foo = 42 ) foo", ""))
                .is_err());
            assert_eq!(
                expression().parse(stream("1", "")).unwrap().0,
                Number::new(1.0, Position::dummy()).into()
            );
            assert_eq!(
                expression().parse(stream("x", "")).unwrap().0,
                Variable::new("x", Position::dummy()).into()
            );
            assert_eq!(
                expression().parse(stream("x + 1", "")).unwrap().0,
                ArithmeticOperation::new(
                    ArithmeticOperator::Add,
                    Variable::new("x", Position::dummy()),
                    Number::new(1.0, Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("x + y z", "")).unwrap().0,
                ArithmeticOperation::new(
                    ArithmeticOperator::Add,
                    Variable::new("x", Position::dummy()),
                    Call::new(
                        Variable::new("y", Position::dummy()),
                        Variable::new("z", Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("(x + y) z", "")).unwrap().0,
                Call::new(
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Variable::new("x", Position::dummy()),
                        Variable::new("y", Position::dummy()),
                        Position::dummy()
                    ),
                    Variable::new("z", Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression()
                    .parse(stream(
                        indoc!(
                            "
                        (f x
                         )
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    Variable::new("x", Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
        }

        #[test]
        fn parse_deeply_nested_expression() {
            assert_eq!(
                expression()
                    .parse(stream("((((((((((((((42))))))))))))))", ""))
                    .unwrap()
                    .0,
                Number::new(42.0, Position::dummy()).into()
            )
        }

        #[test]
        fn parse_atomic_expression() {
            assert!(atomic_expression().parse(stream("?", "")).is_err());
            assert_eq!(
                atomic_expression().parse(stream("1", "")).unwrap().0,
                Number::new(1.0, Position::dummy()).into()
            );
            assert_eq!(
                atomic_expression().parse(stream("x", "")).unwrap().0,
                Variable::new("x", Position::dummy()).into()
            );
            assert_eq!(
                atomic_expression().parse(stream(" x", "")).unwrap().0,
                Variable::new("x", Position::dummy()).into()
            );
        }

        #[test]
        fn parse_if() {
            assert_eq!(
                if_()
                    .parse(stream("if True then 42 else 13", ""))
                    .unwrap()
                    .0,
                If::new(
                    Boolean::new(true, Position::dummy()),
                    Number::new(42.0, Position::dummy()),
                    Number::new(13.0, Position::dummy()),
                    Position::dummy(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream(
                        "if if True then False else True then 42 else 13",
                        ""
                    ))
                    .unwrap()
                    .0,
                If::new(
                    If::new(
                        Boolean::new(true, Position::dummy()),
                        Boolean::new(false, Position::dummy()),
                        Boolean::new(true, Position::dummy()),
                        Position::dummy(),
                    ),
                    Number::new(42.0, Position::dummy()),
                    Number::new(13.0, Position::dummy()),
                    Position::dummy(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream("if True then if False then 1 else 2 else 3", ""))
                    .unwrap()
                    .0,
                If::new(
                    Boolean::new(true, Position::dummy()),
                    If::new(
                        Boolean::new(false, Position::dummy()),
                        Number::new(1.0, Position::dummy()),
                        Number::new(2.0, Position::dummy()),
                        Position::dummy(),
                    ),
                    Number::new(3.0, Position::dummy()),
                    Position::dummy(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream("if True then 1 else if False then 2 else 3", ""))
                    .unwrap()
                    .0,
                If::new(
                    Boolean::new(true, Position::dummy()),
                    Number::new(1.0, Position::dummy()),
                    If::new(
                        Boolean::new(false, Position::dummy()),
                        Number::new(2.0, Position::dummy()),
                        Number::new(3.0, Position::dummy()),
                        Position::dummy(),
                    ),
                    Position::dummy(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream("if x < 0 then 42 else 13", ""))
                    .unwrap()
                    .0,
                If::new(
                    OrderOperation::new(
                        OrderOperator::LessThan,
                        Variable::new("x", Position::dummy()),
                        Number::new(0.0, Position::dummy()),
                        Position::dummy()
                    ),
                    Number::new(42.0, Position::dummy()),
                    Number::new(13.0, Position::dummy()),
                    Position::dummy(),
                )
            );
        }

        #[test]
        fn parse_case() {
            assert_eq!(
                if_type()
                    .parse(stream(
                        indoc!(
                            "
                          case foo = True
                            Boolean => foo
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Case::new(
                    "foo",
                    Boolean::new(true, Position::dummy()),
                    vec![Alternative::new(
                        types::Boolean::new(Position::dummy()),
                        Variable::new("foo", Position::dummy())
                    )],
                    Position::dummy(),
                )
            );
            assert_eq!(
                if_type()
                    .parse(stream(
                        indoc!(
                            "
                          case foo = True
                            Boolean => True
                            None => False
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Case::new(
                    "foo",
                    Boolean::new(true, Position::dummy()),
                    vec![
                        Alternative::new(
                            types::Boolean::new(Position::dummy()),
                            Boolean::new(true, Position::dummy())
                        ),
                        Alternative::new(
                            types::None::new(Position::dummy()),
                            Boolean::new(false, Position::dummy())
                        )
                    ],
                    Position::dummy()
                )
            );
        }

        #[test]
        fn parse_list_case() {
            assert_eq!(
                if_list()
                    .parse(stream(
                        indoc!(
                            "
                            case xs
                                [] => None
                                [ x, ...xs ] => None
                            "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                ListCase::new(
                    Variable::new("xs", Position::dummy()),
                    types::Unknown::new(Position::dummy()),
                    "x",
                    "xs",
                    None::new(Position::dummy()),
                    None::new(Position::dummy()),
                    Position::dummy(),
                )
            );
        }

        #[test]
        fn parse_call() {
            assert_eq!(
                expression().parse(stream("f 1", "")).unwrap().0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    Number::new(1.0, Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("f x", "")).unwrap().0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    Variable::new("x", Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("f 1 2", "")).unwrap().0,
                Call::new(
                    Call::new(
                        Variable::new("f", Position::dummy()),
                        Number::new(1.0, Position::dummy()),
                        Position::dummy()
                    ),
                    Number::new(2.0, Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression()
                    .parse(stream(
                        indoc!(
                            "
                        f x
                        g x =
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    Variable::new("x", Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression()
                    .parse(stream(
                        indoc!(
                            "
                        f x
                         g x =
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    Variable::new("x", Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression()
                    .parse(stream(
                        indoc!(
                            "
                        f
                        x)
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    Variable::new("x", Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression()
                    .parse(stream(
                        indoc!(
                            "
                        f
                        x then
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    Variable::new("x", Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
        }

        #[test]
        fn parse_call_terminator() {
            for source in &[
                "", "\n", " \n", "\n\n", "+", ")", "\n)", "\n )", "}", "then",
            ] {
                assert!(call_terminator().parse(stream(source, "")).is_ok());
            }
        }

        #[test]
        fn parse_operation() {
            for (source, target) in vec![
                (
                    "1 + 1",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(1.0, Position::dummy()),
                        Number::new(1.0, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 + 1 then",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(1.0, Position::dummy()),
                        Number::new(1.0, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 + 1 + 1",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        ArithmeticOperation::new(
                            ArithmeticOperator::Add,
                            Number::new(1.0, Position::dummy()),
                            Number::new(1.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Number::new(1.0, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 + (1 + 1)",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(1.0, Position::dummy()),
                        ArithmeticOperation::new(
                            ArithmeticOperator::Add,
                            Number::new(1.0, Position::dummy()),
                            Number::new(1.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 * 2 - 3",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Subtract,
                        ArithmeticOperation::new(
                            ArithmeticOperator::Multiply,
                            Number::new(1.0, Position::dummy()),
                            Number::new(2.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Number::new(3.0, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 + 2 * 3",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(1.0, Position::dummy()),
                        ArithmeticOperation::new(
                            ArithmeticOperator::Multiply,
                            Number::new(2.0, Position::dummy()),
                            Number::new(3.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 * 2 - 3 / 4",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Subtract,
                        ArithmeticOperation::new(
                            ArithmeticOperator::Multiply,
                            Number::new(1.0, Position::dummy()),
                            Number::new(2.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        ArithmeticOperation::new(
                            ArithmeticOperator::Divide,
                            Number::new(3.0, Position::dummy()),
                            Number::new(4.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 == 1",
                    EqualityOperation::new(
                        EqualityOperator::Equal,
                        Number::new(1.0, Position::dummy()),
                        Number::new(1.0, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "True && True",
                    BooleanOperation::new(
                        BooleanOperator::And,
                        Boolean::new(true, Position::dummy()),
                        Boolean::new(true, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "True || True",
                    BooleanOperation::new(
                        BooleanOperator::Or,
                        Boolean::new(true, Position::dummy()),
                        Boolean::new(true, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "True && 1 < 2",
                    BooleanOperation::new(
                        BooleanOperator::And,
                        Boolean::new(true, Position::dummy()),
                        OrderOperation::new(
                            OrderOperator::LessThan,
                            Number::new(1.0, Position::dummy()),
                            Number::new(2.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "True || True && True",
                    BooleanOperation::new(
                        BooleanOperator::Or,
                        Boolean::new(true, Position::dummy()),
                        BooleanOperation::new(
                            BooleanOperator::And,
                            Boolean::new(true, Position::dummy()),
                            Boolean::new(true, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "42 |> f",
                    PipeOperation::new(
                        Number::new(42.0, Position::dummy()),
                        Variable::new("f", Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "42 |> f |> g",
                    PipeOperation::new(
                        PipeOperation::new(
                            Number::new(42.0, Position::dummy()),
                            Variable::new("f", Position::dummy()),
                            Position::dummy(),
                        ),
                        Variable::new("g", Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
            ] {
                assert_eq!(expression().parse(stream(source, "")).unwrap().0, target);
            }
        }

        #[test]
        fn parse_record_construction() {
            assert!(record().parse(stream("Foo", "")).is_err());

            assert!(record().parse(stream("Foo{}", "")).is_err());

            assert_eq!(
                expression()
                    .parse(stream("Foo { foo = 42 }", ""))
                    .unwrap()
                    .0,
                Variable::new("Foo", Position::dummy()).into()
            );

            assert_eq!(
                record().parse(stream("Foo{ foo = 42 }", "")).unwrap().0,
                RecordConstruction::new(
                    types::Reference::new("Foo", Position::dummy()),
                    vec![("foo".into(), Number::new(42.0, Position::dummy()).into())]
                        .into_iter()
                        .collect(),
                    Position::dummy()
                )
            );

            assert_eq!(
                record()
                    .parse(stream("Foo{ foo = 42, bar = 42 }", ""))
                    .unwrap()
                    .0,
                RecordConstruction::new(
                    types::Reference::new("Foo", Position::dummy()),
                    vec![
                        ("foo".into(), Number::new(42.0, Position::dummy()).into()),
                        ("bar".into(), Number::new(42.0, Position::dummy()).into())
                    ]
                    .into_iter()
                    .collect(),
                    Position::dummy()
                )
            );

            assert!(record()
                .parse(stream("Foo{ foo = 42, foo = 42 }", ""))
                .is_err());

            assert_eq!(
                expression()
                    .parse(stream("foo Foo{ foo = 42 }", ""))
                    .unwrap()
                    .0,
                Call::new(
                    Variable::new("foo", Position::dummy()),
                    RecordConstruction::new(
                        types::Reference::new("Foo", Position::dummy()),
                        vec![("foo".into(), Number::new(42.0, Position::dummy()).into())]
                            .into_iter()
                            .collect(),
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                record()
                    .parse(stream("Foo{ foo = bar\n42, }", ""))
                    .unwrap()
                    .0,
                RecordConstruction::new(
                    types::Reference::new("Foo", Position::dummy()),
                    vec![(
                        "foo".into(),
                        Call::new(
                            Variable::new("bar", Position::dummy()),
                            Number::new(42.0, Position::dummy()),
                            Position::dummy()
                        )
                        .into()
                    )]
                    .into_iter()
                    .collect(),
                    Position::dummy()
                )
            );
        }

        #[test]
        fn parse_record_update() {
            assert_eq!(
                record_update()
                    .parse(stream("Foo{ ...foo, bar = 42 }", ""))
                    .unwrap()
                    .0,
                RecordUpdate::new(
                    types::Reference::new("Foo", Position::dummy()),
                    Variable::new("foo", Position::dummy()),
                    vec![("bar".into(), Number::new(42.0, Position::dummy()).into())]
                        .into_iter()
                        .collect(),
                    Position::dummy()
                )
            );

            assert_eq!(
                record_update()
                    .parse(stream("Foo{ ...foo, bar = 42, }", ""))
                    .unwrap()
                    .0,
                RecordUpdate::new(
                    types::Reference::new("Foo", Position::dummy()),
                    Variable::new("foo", Position::dummy()),
                    vec![("bar".into(), Number::new(42.0, Position::dummy()).into())]
                        .into_iter()
                        .collect(),
                    Position::dummy()
                )
            );

            assert_eq!(
                expression()
                    .parse(stream("Foo { ...foo, bar = 42 }", ""))
                    .unwrap()
                    .0,
                Variable::new("Foo", Position::dummy()).into(),
            );

            assert!(record_update().parse(stream("Foo{ ...foo }", "")).is_err());
            assert!(record_update()
                .parse(stream("Foo{ ...foo, bar = 42, bar = 42 }", ""))
                .is_err());
            assert!(record_update()
                .parse(stream("Foo{ ...(foo bar), baz = 42 }", ""))
                .is_ok());
            assert!(record_update()
                .parse(stream("Foo{ ...foo bar, baz = 42 }", ""))
                .is_err());
        }

        #[test]
        fn parse_operator() {
            assert!(binary_operator().parse(stream("", "")).is_err());
            assert!(binary_operator().parse(stream("++", "")).is_err());

            for (source, expected) in &[
                ("+", ParsedOperator::Add),
                ("-", ParsedOperator::Subtract),
                ("*", ParsedOperator::Multiply),
                ("/", ParsedOperator::Divide),
                ("==", ParsedOperator::Equal),
                ("/=", ParsedOperator::NotEqual),
                ("<", ParsedOperator::LessThan),
                ("<=", ParsedOperator::LessThanOrEqual),
                (">", ParsedOperator::GreaterThan),
                (">=", ParsedOperator::GreaterThanOrEqual),
            ] {
                assert_eq!(
                    binary_operator().parse(stream(source, "")).unwrap().0,
                    *expected
                );
            }
        }

        #[test]
        fn parse_variable() {
            assert!(variable().parse(stream("Foo. x", "")).is_err());
            assert_eq!(
                variable().parse(stream("x", "")).unwrap().0,
                Variable::new("x", Position::dummy()),
            );
            assert_eq!(
                variable().parse(stream("Foo.x", "")).unwrap().0,
                Variable::new("Foo.x", Position::dummy()),
            );
            assert_eq!(
                variable().parse(stream("Foo .x", "")).unwrap().0,
                Variable::new("Foo", Position::dummy()),
            );
        }

        #[test]
        fn parse_boolean_literal() {
            assert!(boolean_literal().parse(stream("", "")).is_err());
            assert_eq!(
                boolean_literal().parse(stream("False", "")).unwrap().0,
                Boolean::new(false, Position::dummy())
            );
            assert_eq!(
                boolean_literal().parse(stream("True", "")).unwrap().0,
                Boolean::new(true, Position::dummy())
            );
        }

        #[test]
        fn parse_none_literal() {
            assert!(none_literal().parse(stream("", "")).is_err());
            assert_eq!(
                none_literal().parse(stream("None", "")).unwrap().0,
                None::new(Position::dummy())
            );
        }

        #[test]
        fn parse_number_literal() {
            assert!(number_literal().parse(stream("", "")).is_err());
            assert!(number_literal().parse(stream("foo", "")).is_err());
            assert!(number_literal().parse(stream("x1", "")).is_err());

            for (source, value) in &[
                ("01", 0.0),
                ("0", 0.0),
                ("1", 1.0),
                ("123456789", 123456789.0),
                ("-1", -1.0),
                ("0.1", 0.1),
                ("0.01", 0.01),
            ] {
                assert_eq!(
                    number_literal().parse(stream(source, "")).unwrap().0,
                    Number::new(*value, Position::dummy())
                );
            }
        }

        #[test]
        fn parse_string_literal() {
            assert!(string_literal().parse(stream("", "")).is_err());
            assert!(string_literal().parse(stream("foo", "")).is_err());

            for (source, value) in &[
                ("\"\"", ""),
                ("\"foo\"", "foo"),
                ("\"foo bar\"", "foo bar"),
                ("\"\\\"\"", "\""),
                ("\"\\n\"", "\n"),
                ("\"\\t\"", "\t"),
                ("\"\\\\\"", "\\"),
                ("\"\\n\\n\"", "\n\n"),
            ] {
                assert_eq!(
                    string_literal().parse(stream(source, "")).unwrap().0,
                    ByteString::new(*value, Position::dummy())
                );
            }
        }

        #[test]
        fn parse_list() {
            for (source, target) in vec![
                ("[]", List::new(vec![], Position::dummy())),
                (
                    "[42]",
                    List::new(
                        vec![ListElement::Single(
                            Number::new(42.0, Position::dummy()).into(),
                        )],
                        Position::dummy(),
                    ),
                ),
                (
                    "[42,]",
                    List::new(
                        vec![ListElement::Single(
                            Number::new(42.0, Position::dummy()).into(),
                        )],
                        Position::dummy(),
                    ),
                ),
                (
                    "[42,42]",
                    List::new(
                        vec![
                            ListElement::Single(Number::new(42.0, Position::dummy()).into()),
                            ListElement::Single(Number::new(42.0, Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[42,42,]",
                    List::new(
                        vec![
                            ListElement::Single(Number::new(42.0, Position::dummy()).into()),
                            ListElement::Single(Number::new(42.0, Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[...foo]",
                    List::new(
                        vec![ListElement::Multiple(
                            Variable::new("foo", Position::dummy()).into(),
                        )],
                        Position::dummy(),
                    ),
                ),
                (
                    "[...foo,]",
                    List::new(
                        vec![ListElement::Multiple(
                            Variable::new("foo", Position::dummy()).into(),
                        )],
                        Position::dummy(),
                    ),
                ),
                (
                    "[...foo,...bar]",
                    List::new(
                        vec![
                            ListElement::Multiple(Variable::new("foo", Position::dummy()).into()),
                            ListElement::Multiple(Variable::new("bar", Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[...foo,...bar,]",
                    List::new(
                        vec![
                            ListElement::Multiple(Variable::new("foo", Position::dummy()).into()),
                            ListElement::Multiple(Variable::new("bar", Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[foo,...bar]",
                    List::new(
                        vec![
                            ListElement::Single(Variable::new("foo", Position::dummy()).into()),
                            ListElement::Multiple(Variable::new("bar", Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[...foo,bar]",
                    List::new(
                        vec![
                            ListElement::Multiple(Variable::new("foo", Position::dummy()).into()),
                            ListElement::Single(Variable::new("bar", Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
            ] {
                assert_eq!(
                    expression().parse(stream(source, "")).unwrap().0,
                    target.into()
                );
            }
        }
    }

    #[test]
    fn parse_identifier() {
        assert!(identifier().parse(stream("let", "")).is_err());
        assert!(identifier().parse(stream("1foo", "")).is_err());
        assert_eq!(
            identifier().parse(stream("foo", "")).unwrap().0,
            "foo".to_string()
        );
        assert_eq!(
            identifier().parse(stream("foo1", "")).unwrap().0,
            "foo1".to_string()
        );
        assert_eq!(
            identifier().parse(stream(" foo", "")).unwrap().0,
            "foo".to_string()
        );
    }

    #[test]
    fn parse_keyword() {
        assert!(keyword("foo").parse(stream("bar", "")).is_err());
        assert!(keyword("foo").parse(stream("fool", "")).is_err());
        assert!(keyword("foo").parse(stream("foo", "")).is_ok());
        assert!(keyword("foo").parse(stream(" foo", "")).is_ok());
    }

    #[test]
    fn parse_sign() {
        assert!(sign("+").parse(stream("", "")).is_err());
        assert!(sign("+").parse(stream("-", "")).is_err());
        assert!(sign("+").parse(stream("+", "")).is_ok());
        assert!(sign("+").parse(stream(" +", "")).is_ok());
        assert!(sign("+").parse(stream(" +x", "")).is_ok());
    }

    #[test]
    fn parse_position() {
        assert!(position()
            .with(combine::eof())
            .parse(stream(" \n \n \n", ""))
            .is_ok());
    }

    #[test]
    fn parse_comment() {
        assert!(comment().parse(stream("#\n", "")).is_ok());
        assert!(comment().parse(stream("#x\n", "")).is_ok());
    }
}
