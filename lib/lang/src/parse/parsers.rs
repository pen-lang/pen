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
        char::{alpha_num, char as character, digit, letter, space, spaces, string},
        combinator::{lazy, no_partial, not_followed_by},
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
use std::collections::HashSet;

const KEYWORDS: &[&str] = &["else", "export", "foreign", "if", "import", "type"];
const OPERATOR_CHARACTERS: &str = "+-*/=<>&|!?";

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
        blank(),
        many(import()),
        many(foreign_import()),
        many(type_definition()),
        many(type_alias()),
        many(definition()),
    )
        .skip(eof())
        .map(
            |(_, imports, foreign_imports, type_definitions, type_aliases, definitions)| {
                Module::new(
                    imports,
                    foreign_imports,
                    type_definitions,
                    type_aliases,
                    definitions,
                )
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
    many1(string(IDENTIFIER_SEPARATOR).with(identifier()))
}

fn foreign_import<'a>() -> impl Parser<Stream<'a>, Output = ForeignImport> {
    (
        position(),
        keyword("foreign"),
        keyword("import"),
        optional(calling_convention()),
        identifier(),
        type_(),
    )
        .map(|(position, _, _, calling_convention, name, type_)| {
            ForeignImport::new(
                &name,
                &name,
                calling_convention.unwrap_or(CallingConvention::Native),
                type_,
                position,
            )
        })
        .expected("foreign import statement")
}

fn calling_convention<'a>() -> impl Parser<Stream<'a>, Output = CallingConvention> {
    string_literal()
        .expected("calling convention")
        .then(|string| match string.value() {
            "c" => value(CallingConvention::C).left(),
            _ => unexpected_any("unknown calling convention").right(),
        })
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
        between(
            sign("{"),
            sign("}"),
            sep_end_by((identifier(), type_()), sign(",")),
        ),
    )
        .map(|(position, _, name, elements): (_, _, _, Vec<_>)| {
            TypeDefinition::new(
                name,
                elements
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
    sep_end_by1(atomic_type(), sign("|"))
        .map(|types: Vec<_>| {
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
        .skip(keyword("boolean"))
        .map(types::Boolean::new)
        .expected("boolean type")
}

fn none_type<'a>() -> impl Parser<Stream<'a>, Output = types::None> {
    position()
        .skip(keyword("none"))
        .map(types::None::new)
        .expected("none type")
}

fn number_type<'a>() -> impl Parser<Stream<'a>, Output = types::Number> {
    position()
        .skip(keyword("number"))
        .map(types::Number::new)
        .expected("number type")
}

fn string_type<'a>() -> impl Parser<Stream<'a>, Output = types::ByteString> {
    position()
        .skip(keyword("string"))
        .map(types::ByteString::new)
        .expected("string type")
}

fn any_type<'a>() -> impl Parser<Stream<'a>, Output = types::Any> {
    position()
        .skip(keyword("any"))
        .map(types::Any::new)
        .expected("any type")
}

fn reference_type<'a>() -> impl Parser<Stream<'a>, Output = types::Reference> {
    token((position(), qualified_identifier()))
        .map(|(position, identifier)| types::Reference::new(identifier, position))
        .expected("reference type")
}

fn block<'a>() -> impl Parser<Stream<'a>, Output = Block> {
    (
        position(),
        between(sign("{"), sign("}"), many1(statement())),
    )
        .then(|(position, statements): (_, Vec<_>)| {
            if let Some(statement) = statements.last() {
                if statement.name().is_none() {
                    value(Block::new(
                        statements[..statements.len() - 1].to_vec(),
                        statement.expression().clone(),
                        position,
                    ))
                    .left()
                } else {
                    unexpected_any("end of block").right()
                }
            } else {
                unexpected_any("end of block").right()
            }
        })
        .expected("block")
}

fn statement<'a>() -> impl Parser<Stream<'a>, Output = Statement> {
    (
        position(),
        optional(identifier().skip(sign("="))),
        expression(),
    )
        .map(|(position, name, expression)| Statement::new(name, expression, position))
        .expected("statement")
}

fn expression<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    lazy(|| no_partial(binary_operation_like()))
        .boxed()
        .expected("expression")
}

fn binary_operation_like<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    (
        prefix_operation_like(),
        many(
            (position(), binary_operator(), prefix_operation_like())
                .map(|(position, operator, expression)| (operator, expression, position)),
        ),
    )
        .map(|(expression, pairs): (_, Vec<_>)| reduce_operations(expression, &pairs))
}

fn binary_operator<'a>() -> impl Parser<Stream<'a>, Output = BinaryOperator> {
    choice!(
        concrete_binary_operator("+", BinaryOperator::Add),
        concrete_binary_operator("-", BinaryOperator::Subtract),
        concrete_binary_operator("*", BinaryOperator::Multiply),
        concrete_binary_operator("/", BinaryOperator::Divide),
        concrete_binary_operator("==", BinaryOperator::Equal),
        concrete_binary_operator("!=", BinaryOperator::NotEqual),
        concrete_binary_operator("<", BinaryOperator::LessThan),
        concrete_binary_operator("<=", BinaryOperator::LessThanOrEqual),
        concrete_binary_operator(">", BinaryOperator::GreaterThan),
        concrete_binary_operator(">=", BinaryOperator::GreaterThanOrEqual),
        concrete_binary_operator("&", BinaryOperator::And),
        concrete_binary_operator("|", BinaryOperator::Or),
    )
    .expected("binary operator")
}

fn concrete_binary_operator<'a>(
    literal: &'static str,
    operator: BinaryOperator,
) -> impl Parser<Stream<'a>, Output = BinaryOperator> {
    token(
        many1(one_of(OPERATOR_CHARACTERS.chars())).then(move |parsed_literal: String| {
            if parsed_literal == literal {
                value(operator).left()
            } else {
                unexpected_any("unknown binary operator").right()
            }
        }),
    )
}

fn prefix_operation_like<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    lazy(|| {
        no_partial(choice!(
            prefix_operation().map(Expression::from),
            suffix_operation_like().map(Expression::from),
        ))
    })
    .boxed()
}

fn prefix_operation<'a>() -> impl Parser<Stream<'a>, Output = UnaryOperation> {
    (position(), prefix_operator(), prefix_operation_like())
        .map(|(position, operator, expression)| UnaryOperation::new(operator, expression, position))
}

fn prefix_operator<'a>() -> impl Parser<Stream<'a>, Output = UnaryOperator> {
    choice!(
        concrete_prefix_operator("!", UnaryOperator::Not),
        concrete_prefix_operator("?", UnaryOperator::Try),
    )
    .expected("unary operator")
}

fn concrete_prefix_operator<'a>(
    literal: &'static str,
    operator: UnaryOperator,
) -> impl Parser<Stream<'a>, Output = UnaryOperator> {
    token(
        one_of(OPERATOR_CHARACTERS.chars()).then(move |parsed_literal: char| {
            if parsed_literal.to_string() == literal {
                value(operator).left()
            } else {
                unexpected_any("unknown unary operator").right()
            }
        }),
    )
}

fn suffix_operation_like<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    (atomic_expression(), many((position(), suffix_operator()))).map(
        |(expression, suffix_operators): (_, Vec<_>)| {
            suffix_operators
                .into_iter()
                .fold(
                    expression,
                    |expression, (position, operator)| match operator {
                        SuffixOperator::Call(arguments) => {
                            Call::new(expression, arguments, position).into()
                        }
                        SuffixOperator::Element(name) => {
                            RecordDeconstruction::new(expression, name, position).into()
                        }
                    },
                )
        },
    )
}

fn suffix_operator<'a>() -> impl Parser<Stream<'a>, Output = SuffixOperator> {
    choice!(
        call_operator().map(SuffixOperator::Call),
        element_operator().map(SuffixOperator::Element),
    )
}

fn call_operator<'a>() -> impl Parser<Stream<'a>, Output = Vec<Expression>> {
    between(sign("("), sign(")"), sep_end_by(expression(), sign(",")))
}

fn element_operator<'a>() -> impl Parser<Stream<'a>, Output = String> {
    sign(".").with(identifier())
}

fn atomic_expression<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    lazy(|| {
        no_partial(choice!(
            if_().map(Expression::from),
            if_type().map(Expression::from),
            if_list().map(Expression::from),
            lambda().map(Expression::from),
            record().map(Expression::from),
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
    (
        position(),
        sign("\\("),
        sep_end_by(argument(), sign(",")),
        sign(")"),
        type_(),
        block(),
    )
        .map(|(position, _, arguments, _, result_type, body)| {
            Lambda::new(arguments, result_type, body, position)
        })
        .expected("function expression")
}

fn argument<'a>() -> impl Parser<Stream<'a>, Output = Argument> {
    (identifier(), type_()).map(|(name, type_)| Argument::new(name, type_))
}

fn if_<'a>() -> impl Parser<Stream<'a>, Output = If> {
    (
        position(),
        keyword("if"),
        if_branch(),
        many((keyword("else"), keyword("if")).with(if_branch())),
        keyword("else"),
        block(),
    )
        .map(
            |(position, _, first_branch, branches, _, else_block): (_, _, _, Vec<_>, _, _)| {
                If::new(
                    vec![first_branch].into_iter().chain(branches).collect(),
                    else_block,
                    position,
                )
            },
        )
        .expected("if expression")
}

fn if_branch<'a>() -> impl Parser<Stream<'a>, Output = IfBranch> {
    (expression(), block()).map(|(expression, block)| IfBranch::new(expression, block))
}

fn if_list<'a>() -> impl Parser<Stream<'a>, Output = IfList> {
    (
        position(),
        keyword("if"),
        sign("["),
        identifier(),
        sign(","),
        sign("..."),
        identifier(),
        sign("]"),
        sign("="),
        expression(),
        block(),
        keyword("else"),
        block(),
    )
        .map(
            |(position, _, _, first_name, _, _, rest_name, _, _, argument, then, _, else_)| {
                IfList::new(argument, first_name, rest_name, then, else_, position)
            },
        )
        .expected("if-list expression")
}

fn if_type<'a>() -> impl Parser<Stream<'a>, Output = IfType> {
    (
        position(),
        keyword("if"),
        identifier(),
        sign("="),
        expression(),
        sign(";"),
        if_type_branch(),
        many((keyword("else"), keyword("if")).with(if_type_branch())),
        optional(keyword("else").with(block())),
    )
        .map(
            |(position, _, identifier, _, argument, _, first_branch, branches, else_): (
                _,
                _,
                _,
                _,
                _,
                _,
                _,
                Vec<_>,
                _,
            )| {
                IfType::new(
                    identifier,
                    argument,
                    vec![first_branch].into_iter().chain(branches).collect(),
                    else_,
                    position,
                )
            },
        )
        .expected("type case expression")
}

fn if_type_branch<'a>() -> impl Parser<Stream<'a>, Output = IfTypeBranch> {
    (type_(), block()).map(|(type_, block)| IfTypeBranch::new(type_, block))
}

fn record<'a>() -> impl Parser<Stream<'a>, Output = Record> {
    (
        position(),
        reference_type(),
        sign("{"),
        optional(between(sign("..."), sign(","), expression())),
        sep_end_by(record_element(), sign(",")),
        sign("}"),
    )
        .then(|(position, reference_type, _, record, elements, _)| {
            let elements: Vec<_> = elements;

            if elements
                .iter()
                .map(|element| element.name())
                .collect::<HashSet<_>>()
                .len()
                == elements.len()
            {
                value(Record::new(reference_type, record, elements, position)).left()
            } else {
                unexpected_any("duplicate keys in record literal").right()
            }
        })
}

fn record_element<'a>() -> impl Parser<Stream<'a>, Output = RecordElement> {
    (position(), identifier(), sign(":"), expression())
        .map(|(position, name, _, expression)| RecordElement::new(name, expression, position))
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
        .skip(not_followed_by(digit()))
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
        optional(raw_identifier().skip(string(IDENTIFIER_SEPARATOR))),
        raw_identifier(),
    )
        .map(|(prefix, identifier)| {
            prefix
                .map(|prefix| [&prefix, IDENTIFIER_SEPARATOR, &identifier].concat())
                .unwrap_or(identifier)
        })
}

fn identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    token(raw_identifier()).expected("identifier")
}

fn raw_identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    (
        choice!(letter(), combine::parser::char::char('_')),
        many(choice!(alpha_num(), combine::parser::char::char('_'))),
    )
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
    p.skip(blank())
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
    combine::eof().expected("end of file")
}

fn blank<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    many::<Vec<_>, _, _>(choice!(space().with(value(())), comment())).with(value(()))
}

fn comment<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    string("#")
        .with(many::<Vec<_>, _, _>(none_of("\n".chars())))
        .with(choice!(
            combine::parser::char::newline().with(value(())),
            eof()
        ))
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
            Module::new(vec![], vec![], vec![], vec![], vec![])
        );
        assert_eq!(
            module().parse(stream(" ", "")).unwrap().0,
            Module::new(vec![], vec![], vec![], vec![], vec![])
        );
        assert_eq!(
            module().parse(stream("\n", "")).unwrap().0,
            Module::new(vec![], vec![], vec![], vec![], vec![])
        );
        assert_eq!(
            module().parse(stream("import Foo'Bar", "")).unwrap().0,
            Module::new(
                vec![Import::new(ExternalModulePath::new(
                    "Foo",
                    vec!["Bar".into()]
                ))],
                vec![],
                vec![],
                vec![],
                vec![]
            )
        );
        assert_eq!(
            module().parse(stream("type foo = number", "")).unwrap().0,
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![TypeAlias::new("foo", types::Number::new(Position::dummy()))],
                vec![]
            )
        );
        assert_eq!(
            module()
                .parse(stream("x=\\(x number)number{42}", ""))
                .unwrap()
                .0,
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", types::Number::new(Position::dummy()))],
                        types::Number::new(Position::dummy()),
                        Block::new(
                            vec![],
                            Number::new(42.0, Position::dummy()),
                            Position::dummy()
                        ),
                        Position::dummy()
                    ),
                    Position::dummy()
                )]
            )
        );
        assert_eq!(
            module()
                .parse(stream(
                    "x=\\(x number)number{42}y=\\(y number)number{42}",
                    ""
                ))
                .unwrap()
                .0,
            Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                vec![
                    Definition::new(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", types::Number::new(Position::dummy()))],
                            types::Number::new(Position::dummy()),
                            Block::new(
                                vec![],
                                Number::new(42.0, Position::dummy()),
                                Position::dummy()
                            ),
                            Position::dummy()
                        ),
                        Position::dummy()
                    ),
                    Definition::new(
                        "y",
                        Lambda::new(
                            vec![Argument::new("y", types::Number::new(Position::dummy()))],
                            types::Number::new(Position::dummy()),
                            Block::new(
                                vec![],
                                Number::new(42.0, Position::dummy()),
                                Position::dummy()
                            ),
                            Position::dummy()
                        ),
                        Position::dummy()
                    )
                ]
            )
        );
    }

    #[test]
    fn parse_import() {
        assert_eq!(
            import().parse(stream("import 'Foo", "")).unwrap().0,
            Import::new(InternalModulePath::new(vec!["Foo".into()])),
        );
        assert_eq!(
            import().parse(stream("import Foo'Bar", "")).unwrap().0,
            Import::new(ExternalModulePath::new("Foo", vec!["Bar".into()])),
        );
    }

    #[test]
    fn parse_module_path() {
        assert!(module_path().parse(stream("", "")).is_err());
        assert_eq!(
            module_path().parse(stream("'Foo", "")).unwrap().0,
            InternalModulePath::new(vec!["Foo".into()]).into(),
        );
        assert_eq!(
            module_path().parse(stream("Foo'Bar", "")).unwrap().0,
            ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
        );
    }

    #[test]
    fn parse_internal_module_path() {
        assert!(internal_module_path().parse(stream("", "")).is_err());
        assert_eq!(
            internal_module_path().parse(stream("'Foo", "")).unwrap().0,
            InternalModulePath::new(vec!["Foo".into()]),
        );
        assert_eq!(
            internal_module_path()
                .parse(stream("'Foo'Bar", ""))
                .unwrap()
                .0,
            InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
        );
    }

    #[test]
    fn parse_external_module_path() {
        assert!(external_module_path().parse(stream("", "")).is_err());
        assert_eq!(
            external_module_path()
                .parse(stream("Foo'Bar", ""))
                .unwrap()
                .0,
            ExternalModulePath::new("Foo", vec!["Bar".into()]),
        );
    }

    #[test]
    fn parse_foreign_import() {
        assert_eq!(
            foreign_import()
                .parse(stream("foreign import foo \\(number) number", ""))
                .unwrap()
                .0,
            ForeignImport::new(
                "foo",
                "foo",
                CallingConvention::Native,
                types::Function::new(
                    vec![types::Number::new(Position::dummy()).into()],
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                ),
                Position::dummy()
            ),
        );

        assert_eq!(
            foreign_import()
                .parse(stream("foreign import \"c\" foo \\(number) number", ""))
                .unwrap()
                .0,
            ForeignImport::new(
                "foo",
                "foo",
                CallingConvention::C,
                types::Function::new(
                    vec![types::Number::new(Position::dummy()).into()],
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
                .parse(stream("x=\\(x number)number{42}", ""))
                .unwrap()
                .0,
            Definition::new(
                "x",
                Lambda::new(
                    vec![Argument::new("x", types::Number::new(Position::dummy()))],
                    types::Number::new(Position::dummy()),
                    Block::new(
                        vec![],
                        Number::new(42.0, Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                ),
                Position::dummy()
            ),
        );
    }

    #[test]
    fn parse_type_definition() {
        for (source, expected) in &[
            (
                "type Foo {}",
                TypeDefinition::new("Foo", vec![], Position::dummy()),
            ),
            (
                "type Foo {foo number}",
                TypeDefinition::new(
                    "Foo",
                    vec![types::RecordElement::new(
                        "foo",
                        types::Number::new(Position::dummy()),
                    )],
                    Position::dummy(),
                ),
            ),
            (
                "type Foo {foo number,}",
                TypeDefinition::new(
                    "Foo",
                    vec![types::RecordElement::new(
                        "foo",
                        types::Number::new(Position::dummy()),
                    )],
                    Position::dummy(),
                ),
            ),
            (
                "type Foo {foo number,bar number}",
                TypeDefinition::new(
                    "Foo",
                    vec![
                        types::RecordElement::new("foo", types::Number::new(Position::dummy())),
                        types::RecordElement::new("bar", types::Number::new(Position::dummy())),
                    ],
                    Position::dummy(),
                ),
            ),
            (
                "type Foo {foo number,bar number,}",
                TypeDefinition::new(
                    "Foo",
                    vec![
                        types::RecordElement::new("foo", types::Number::new(Position::dummy())),
                        types::RecordElement::new("bar", types::Number::new(Position::dummy())),
                    ],
                    Position::dummy(),
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
    fn parse_type_alias() {
        for (source, expected) in &[
            (
                "type foo=number",
                TypeAlias::new("foo", types::Number::new(Position::dummy())),
            ),
            (
                "type foo = number",
                TypeAlias::new("foo", types::Number::new(Position::dummy())),
            ),
            (
                "type foo=number|none",
                TypeAlias::new(
                    "foo",
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::None::new(Position::dummy()),
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
            assert!(type_().parse(stream("", "")).is_err());
            assert_eq!(
                type_().parse(stream("boolean", "")).unwrap().0,
                types::Boolean::new(Position::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("none", "")).unwrap().0,
                types::None::new(Position::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("number", "")).unwrap().0,
                types::Number::new(Position::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("Foo", "")).unwrap().0,
                types::Reference::new("Foo", Position::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("Foo'Bar", "")).unwrap().0,
                types::Reference::new("Foo'Bar", Position::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("\\(number)number", "")).unwrap().0,
                types::Function::new(
                    vec![types::Number::new(Position::dummy()).into()],
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("\\(number,number)number", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    vec![
                        types::Number::new(Position::dummy()).into(),
                        types::Number::new(Position::dummy()).into(),
                    ],
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("\\(\\(number)number)number", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    vec![types::Function::new(
                        vec![types::Number::new(Position::dummy()).into()],
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    )
                    .into()],
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_().parse(stream("number|none", "")).unwrap().0,
                types::Union::new(
                    types::Number::new(Position::dummy()),
                    types::None::new(Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_().parse(stream("boolean|number|none", "")).unwrap().0,
                types::Union::new(
                    types::Union::new(
                        types::Boolean::new(Position::dummy()),
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    ),
                    types::None::new(Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("\\(number)number|none", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    vec![types::Number::new(Position::dummy()).into()],
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::None::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("(\\(number)number)|none", ""))
                    .unwrap()
                    .0,
                types::Union::new(
                    types::Function::new(
                        vec![types::Number::new(Position::dummy()).into()],
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    ),
                    types::None::new(Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
        }

        #[test]
        fn parse_any_type() {
            assert_eq!(
                any_type().parse(stream("any", "")).unwrap().0,
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
                type_().parse(stream("Foo'Bar", "")).unwrap().0,
                types::Reference::new("Foo'Bar", Position::dummy()).into()
            );
        }

        #[test]
        fn parse_list_type() {
            assert_eq!(
                type_().parse(stream("[number]", "")).unwrap().0,
                types::List::new(types::Number::new(Position::dummy()), Position::dummy()).into()
            );

            assert_eq!(
                type_().parse(stream("[[number]]", "")).unwrap().0,
                types::List::new(
                    types::List::new(types::Number::new(Position::dummy()), Position::dummy()),
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                type_().parse(stream("[number]|[none]", "")).unwrap().0,
                types::Union::new(
                    types::List::new(types::Number::new(Position::dummy()), Position::dummy()),
                    types::List::new(types::None::new(Position::dummy()), Position::dummy()),
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                type_().parse(stream("\\([number])[none]", "")).unwrap().0,
                types::Function::new(
                    vec![types::List::new(
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    )
                    .into()],
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
            assert!(expression().parse(stream("", "")).is_err());
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
                BinaryOperation::new(
                    BinaryOperator::Add,
                    Variable::new("x", Position::dummy()),
                    Number::new(1.0, Position::dummy()),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("x + y(z)", "")).unwrap().0,
                BinaryOperation::new(
                    BinaryOperator::Add,
                    Variable::new("x", Position::dummy()),
                    Call::new(
                        Variable::new("y", Position::dummy()),
                        vec![Variable::new("z", Position::dummy()).into()],
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("(x + y)(z)", "")).unwrap().0,
                Call::new(
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        Variable::new("x", Position::dummy()),
                        Variable::new("y", Position::dummy()),
                        Position::dummy()
                    ),
                    vec![Variable::new("z", Position::dummy()).into()],
                    Position::dummy()
                )
                .into()
            );
        }

        #[test]
        fn parse_deeply_nested_expression() {
            assert_eq!(
                expression().parse(stream("(((((42)))))", "")).unwrap().0,
                Number::new(42.0, Position::dummy()).into()
            )
        }

        #[test]
        fn parse_atomic_expression() {
            assert!(atomic_expression().parse(stream("", "")).is_err());
            assert_eq!(
                atomic_expression().parse(stream("1", "")).unwrap().0,
                Number::new(1.0, Position::dummy()).into()
            );
            assert_eq!(
                atomic_expression().parse(stream("x", "")).unwrap().0,
                Variable::new("x", Position::dummy()).into()
            );
            assert_eq!(
                atomic_expression().parse(stream("(x)", "")).unwrap().0,
                Variable::new("x", Position::dummy()).into()
            );
        }

        #[test]
        fn parse_lambda() {
            assert_eq!(
                lambda()
                    .parse(stream("\\(x number)number{42}", ""))
                    .unwrap()
                    .0,
                Lambda::new(
                    vec![Argument::new("x", types::Number::new(Position::dummy()))],
                    types::Number::new(Position::dummy()),
                    Block::new(
                        vec![],
                        Number::new(42.0, Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                ),
            );

            assert_eq!(
                lambda()
                    .parse(stream("\\(x number,y number)number{42}", ""))
                    .unwrap()
                    .0,
                Lambda::new(
                    vec![
                        Argument::new("x", types::Number::new(Position::dummy())),
                        Argument::new("y", types::Number::new(Position::dummy()))
                    ],
                    types::Number::new(Position::dummy()),
                    Block::new(
                        vec![],
                        Number::new(42.0, Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                ),
            );
        }

        #[test]
        fn parse_lambda_with_reference_type() {
            assert_eq!(
                lambda().parse(stream("\\() Foo { 42 }", "")).unwrap().0,
                Lambda::new(
                    vec![],
                    types::Reference::new("Foo", Position::dummy()),
                    Block::new(
                        vec![],
                        Number::new(42.0, Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                ),
            );
        }

        #[test]
        fn parse_block() {
            assert_eq!(
                block().parse(stream("{none}", "")).unwrap().0,
                Block::new(vec![], None::new(Position::dummy()), Position::dummy()),
            );
            assert_eq!(
                block().parse(stream("{none none}", "")).unwrap().0,
                Block::new(
                    vec![Statement::new(
                        None,
                        None::new(Position::dummy()),
                        Position::dummy()
                    )],
                    None::new(Position::dummy()),
                    Position::dummy()
                ),
            );
            assert_eq!(
                block().parse(stream("{none none none}", "")).unwrap().0,
                Block::new(
                    vec![
                        Statement::new(None, None::new(Position::dummy()), Position::dummy()),
                        Statement::new(None, None::new(Position::dummy()), Position::dummy())
                    ],
                    None::new(Position::dummy()),
                    Position::dummy()
                ),
            );
            assert_eq!(
                block().parse(stream("{x=none none}", "")).unwrap().0,
                Block::new(
                    vec![Statement::new(
                        Some("x".into()),
                        None::new(Position::dummy()),
                        Position::dummy()
                    )],
                    None::new(Position::dummy()),
                    Position::dummy()
                ),
            );
        }

        #[test]
        fn parse_if() {
            assert_eq!(
                if_()
                    .parse(stream("if true { 42 } else { 13 }", ""))
                    .unwrap()
                    .0,
                If::new(
                    vec![IfBranch::new(
                        Boolean::new(true, Position::dummy()),
                        Block::new(
                            vec![],
                            Number::new(42.0, Position::dummy()),
                            Position::dummy()
                        ),
                    )],
                    Block::new(
                        vec![],
                        Number::new(13.0, Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream("if if true{true}else{true}{42}else{13}", ""))
                    .unwrap()
                    .0,
                If::new(
                    vec![IfBranch::new(
                        If::new(
                            vec![IfBranch::new(
                                Boolean::new(true, Position::dummy()),
                                Block::new(
                                    vec![],
                                    Boolean::new(true, Position::dummy()),
                                    Position::dummy()
                                ),
                            )],
                            Block::new(
                                vec![],
                                Boolean::new(true, Position::dummy()),
                                Position::dummy()
                            ),
                            Position::dummy(),
                        ),
                        Block::new(
                            vec![],
                            Number::new(42.0, Position::dummy()),
                            Position::dummy()
                        ),
                    )],
                    Block::new(
                        vec![],
                        Number::new(13.0, Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream("if true{1}else if true{2}else{3}", ""))
                    .unwrap()
                    .0,
                If::new(
                    vec![
                        IfBranch::new(
                            Boolean::new(true, Position::dummy()),
                            Block::new(
                                vec![],
                                Number::new(1.0, Position::dummy()),
                                Position::dummy()
                            ),
                        ),
                        IfBranch::new(
                            Boolean::new(true, Position::dummy()),
                            Block::new(
                                vec![],
                                Number::new(2.0, Position::dummy()),
                                Position::dummy()
                            ),
                        )
                    ],
                    Block::new(
                        vec![],
                        Number::new(3.0, Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy(),
                )
            );
        }

        #[test]
        fn parse_if_type() {
            assert_eq!(
                if_type()
                    .parse(stream("if x=y;boolean{none}else{none}", ""))
                    .unwrap()
                    .0,
                IfType::new(
                    "x",
                    Variable::new("y", Position::dummy()),
                    vec![IfTypeBranch::new(
                        types::Boolean::new(Position::dummy()),
                        Block::new(vec![], None::new(Position::dummy()), Position::dummy()),
                    )],
                    Some(Block::new(
                        vec![],
                        None::new(Position::dummy()),
                        Position::dummy()
                    )),
                    Position::dummy(),
                )
            );

            assert_eq!(
                if_type()
                    .parse(stream(
                        "if x=y;boolean{none}else if none{none}else{none}",
                        ""
                    ))
                    .unwrap()
                    .0,
                IfType::new(
                    "x",
                    Variable::new("y", Position::dummy()),
                    vec![
                        IfTypeBranch::new(
                            types::Boolean::new(Position::dummy()),
                            Block::new(vec![], None::new(Position::dummy()), Position::dummy()),
                        ),
                        IfTypeBranch::new(
                            types::None::new(Position::dummy()),
                            Block::new(vec![], None::new(Position::dummy()), Position::dummy()),
                        )
                    ],
                    Some(Block::new(
                        vec![],
                        None::new(Position::dummy()),
                        Position::dummy()
                    )),
                    Position::dummy()
                )
            );

            assert_eq!(
                if_type()
                    .parse(stream("if x=y;boolean{none}else if none{none}", ""))
                    .unwrap()
                    .0,
                IfType::new(
                    "x",
                    Variable::new("y", Position::dummy()),
                    vec![
                        IfTypeBranch::new(
                            types::Boolean::new(Position::dummy()),
                            Block::new(vec![], None::new(Position::dummy()), Position::dummy()),
                        ),
                        IfTypeBranch::new(
                            types::None::new(Position::dummy()),
                            Block::new(vec![], None::new(Position::dummy()), Position::dummy()),
                        )
                    ],
                    None,
                    Position::dummy()
                )
            );
        }

        #[test]
        fn parse_if_list() {
            assert_eq!(
                if_list()
                    .parse(stream("if[x,...xs]=xs{none}else{none}", ""))
                    .unwrap()
                    .0,
                IfList::new(
                    Variable::new("xs", Position::dummy()),
                    "x",
                    "xs",
                    Block::new(vec![], None::new(Position::dummy()), Position::dummy()),
                    Block::new(vec![], None::new(Position::dummy()), Position::dummy()),
                    Position::dummy(),
                )
            );
        }

        #[test]
        fn parse_call() {
            assert_eq!(
                expression().parse(stream("f()", "")).unwrap().0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    vec![],
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                expression().parse(stream("f()()", "")).unwrap().0,
                Call::new(
                    Call::new(
                        Variable::new("f", Position::dummy()),
                        vec![],
                        Position::dummy()
                    ),
                    vec![],
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                expression().parse(stream("f(1)", "")).unwrap().0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    vec![Number::new(1.0, Position::dummy()).into()],
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                expression().parse(stream("f(1,)", "")).unwrap().0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    vec![Number::new(1.0, Position::dummy()).into()],
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                expression().parse(stream("f(1, 2)", "")).unwrap().0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    vec![
                        Number::new(1.0, Position::dummy()).into(),
                        Number::new(2.0, Position::dummy()).into()
                    ],
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                expression().parse(stream("f(1, 2,)", "")).unwrap().0,
                Call::new(
                    Variable::new("f", Position::dummy()),
                    vec![
                        Number::new(1.0, Position::dummy()).into(),
                        Number::new(2.0, Position::dummy()).into()
                    ],
                    Position::dummy()
                )
                .into()
            );
        }

        #[test]
        fn parse_unary_operation() {
            assert!(prefix_operation().parse(stream("", "")).is_err());

            for (source, expected) in &[
                (
                    "!42",
                    UnaryOperation::new(
                        UnaryOperator::Not,
                        Number::new(42.0, Position::dummy()),
                        Position::dummy(),
                    ),
                ),
                (
                    "?42",
                    UnaryOperation::new(
                        UnaryOperator::Try,
                        Number::new(42.0, Position::dummy()),
                        Position::dummy(),
                    ),
                ),
                (
                    "!f(42)",
                    UnaryOperation::new(
                        UnaryOperator::Not,
                        Call::new(
                            Variable::new("f", Position::dummy()),
                            vec![Number::new(42.0, Position::dummy()).into()],
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                ),
                (
                    "!if true{true}else{true}",
                    UnaryOperation::new(
                        UnaryOperator::Not,
                        If::new(
                            vec![IfBranch::new(
                                Boolean::new(true, Position::dummy()),
                                Block::new(
                                    vec![],
                                    Boolean::new(true, Position::dummy()),
                                    Position::dummy(),
                                ),
                            )],
                            Block::new(
                                vec![],
                                Boolean::new(true, Position::dummy()),
                                Position::dummy(),
                            ),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                ),
                (
                    "!!42",
                    UnaryOperation::new(
                        UnaryOperator::Not,
                        UnaryOperation::new(
                            UnaryOperator::Not,
                            Number::new(42.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                ),
            ] {
                assert_eq!(
                    prefix_operation().parse(stream(source, "")).unwrap().0,
                    *expected
                );
            }
        }

        #[test]
        fn parse_unary_operator() {
            assert!(prefix_operator().parse(stream("", "")).is_err());

            for (source, expected) in &[("!", UnaryOperator::Not), ("?", UnaryOperator::Try)] {
                assert_eq!(
                    prefix_operator().parse(stream(source, "")).unwrap().0,
                    *expected
                );
            }
        }

        #[test]
        fn parse_binary_operation() {
            for (source, target) in vec![
                (
                    "1+1",
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        Number::new(1.0, Position::dummy()),
                        Number::new(1.0, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1+1+1",
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        BinaryOperation::new(
                            BinaryOperator::Add,
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
                    "1+(1+1)",
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        Number::new(1.0, Position::dummy()),
                        BinaryOperation::new(
                            BinaryOperator::Add,
                            Number::new(1.0, Position::dummy()),
                            Number::new(1.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1*2-3",
                    BinaryOperation::new(
                        BinaryOperator::Subtract,
                        BinaryOperation::new(
                            BinaryOperator::Multiply,
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
                    "1+2*3",
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        Number::new(1.0, Position::dummy()),
                        BinaryOperation::new(
                            BinaryOperator::Multiply,
                            Number::new(2.0, Position::dummy()),
                            Number::new(3.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1*2-3/4",
                    BinaryOperation::new(
                        BinaryOperator::Subtract,
                        BinaryOperation::new(
                            BinaryOperator::Multiply,
                            Number::new(1.0, Position::dummy()),
                            Number::new(2.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        BinaryOperation::new(
                            BinaryOperator::Divide,
                            Number::new(3.0, Position::dummy()),
                            Number::new(4.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "1==1",
                    BinaryOperation::new(
                        BinaryOperator::Equal,
                        Number::new(1.0, Position::dummy()),
                        Number::new(1.0, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "true&true",
                    BinaryOperation::new(
                        BinaryOperator::And,
                        Boolean::new(true, Position::dummy()),
                        Boolean::new(true, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "true|true",
                    BinaryOperation::new(
                        BinaryOperator::Or,
                        Boolean::new(true, Position::dummy()),
                        Boolean::new(true, Position::dummy()),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "true&1<2",
                    BinaryOperation::new(
                        BinaryOperator::And,
                        Boolean::new(true, Position::dummy()),
                        BinaryOperation::new(
                            BinaryOperator::LessThan,
                            Number::new(1.0, Position::dummy()),
                            Number::new(2.0, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
                (
                    "true|true&true",
                    BinaryOperation::new(
                        BinaryOperator::Or,
                        Boolean::new(true, Position::dummy()),
                        BinaryOperation::new(
                            BinaryOperator::And,
                            Boolean::new(true, Position::dummy()),
                            Boolean::new(true, Position::dummy()),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    )
                    .into(),
                ),
            ] {
                assert_eq!(expression().parse(stream(source, "")).unwrap().0, target);
            }
        }

        #[test]
        fn parse_binary_operator() {
            assert!(binary_operator().parse(stream("", "")).is_err());
            assert!(binary_operator().parse(stream("++", "")).is_err());

            for (source, expected) in &[
                ("+", BinaryOperator::Add),
                ("-", BinaryOperator::Subtract),
                ("*", BinaryOperator::Multiply),
                ("/", BinaryOperator::Divide),
                ("==", BinaryOperator::Equal),
                ("!=", BinaryOperator::NotEqual),
                ("<", BinaryOperator::LessThan),
                ("<=", BinaryOperator::LessThanOrEqual),
                (">", BinaryOperator::GreaterThan),
                (">=", BinaryOperator::GreaterThanOrEqual),
                ("&", BinaryOperator::And),
                ("|", BinaryOperator::Or),
            ] {
                assert_eq!(
                    binary_operator().parse(stream(source, "")).unwrap().0,
                    *expected
                );
            }
        }

        #[test]
        fn parse_record() {
            assert!(record().parse(stream("Foo", "")).is_err());

            assert_eq!(
                record().parse(stream("Foo {}", "")).unwrap().0,
                Record::new(
                    types::Reference::new("Foo", Position::dummy()),
                    None,
                    vec![],
                    Position::dummy()
                )
            );

            assert_eq!(
                expression().parse(stream("Foo {foo:42}", "")).unwrap().0,
                Record::new(
                    types::Reference::new("Foo", Position::dummy()),
                    None,
                    vec![RecordElement::new(
                        "foo",
                        Number::new(42.0, Position::dummy()),
                        Position::dummy()
                    )],
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                record().parse(stream("Foo{foo:42}", "")).unwrap().0,
                Record::new(
                    types::Reference::new("Foo", Position::dummy()),
                    None,
                    vec![RecordElement::new(
                        "foo",
                        Number::new(42.0, Position::dummy()),
                        Position::dummy()
                    )],
                    Position::dummy()
                )
            );

            assert_eq!(
                record().parse(stream("Foo{foo:42,bar:42}", "")).unwrap().0,
                Record::new(
                    types::Reference::new("Foo", Position::dummy()),
                    None,
                    vec![
                        RecordElement::new(
                            "foo",
                            Number::new(42.0, Position::dummy()),
                            Position::dummy()
                        ),
                        RecordElement::new(
                            "bar",
                            Number::new(42.0, Position::dummy()),
                            Position::dummy()
                        )
                    ],
                    Position::dummy()
                )
            );

            assert!(record().parse(stream("Foo{foo:42,foo:42}", "")).is_err());

            assert_eq!(
                expression()
                    .parse(stream("foo(Foo{foo:42})", ""))
                    .unwrap()
                    .0,
                Call::new(
                    Variable::new("foo", Position::dummy()),
                    vec![Record::new(
                        types::Reference::new("Foo", Position::dummy()),
                        None,
                        vec![RecordElement::new(
                            "foo",
                            Number::new(42.0, Position::dummy()),
                            Position::dummy()
                        )],
                        Position::dummy()
                    )
                    .into()],
                    Position::dummy()
                )
                .into()
            );

            assert_eq!(
                record().parse(stream("Foo{foo:bar(42)}", "")).unwrap().0,
                Record::new(
                    types::Reference::new("Foo", Position::dummy()),
                    None,
                    vec![RecordElement::new(
                        "foo",
                        Call::new(
                            Variable::new("bar", Position::dummy()),
                            vec![Number::new(42.0, Position::dummy()).into()],
                            Position::dummy()
                        ),
                        Position::dummy()
                    )],
                    Position::dummy()
                )
            );

            assert_eq!(
                record().parse(stream("Foo{...foo,bar:42}", "")).unwrap().0,
                Record::new(
                    types::Reference::new("Foo", Position::dummy()),
                    Some(Variable::new("foo", Position::dummy()).into()),
                    vec![RecordElement::new(
                        "bar",
                        Number::new(42.0, Position::dummy()),
                        Position::dummy()
                    )],
                    Position::dummy()
                )
            );

            assert_eq!(
                record().parse(stream("Foo{...foo,bar:42,}", "")).unwrap().0,
                Record::new(
                    types::Reference::new("Foo", Position::dummy()),
                    Some(Variable::new("foo", Position::dummy()).into()),
                    vec![RecordElement::new(
                        "bar",
                        Number::new(42.0, Position::dummy()),
                        Position::dummy()
                    )],
                    Position::dummy()
                )
            );

            assert_eq!(
                expression()
                    .parse(stream("Foo {...foo,bar:42}", ""))
                    .unwrap()
                    .0,
                Record::new(
                    types::Reference::new("Foo", Position::dummy()),
                    Some(Variable::new("foo", Position::dummy()).into()),
                    vec![RecordElement::new(
                        "bar",
                        Number::new(42.0, Position::dummy()),
                        Position::dummy()
                    )],
                    Position::dummy()
                )
                .into(),
            );

            assert!(record().parse(stream("Foo{...foo}", "")).is_err());
            assert!(record()
                .parse(stream("Foo{...foo,bar:42,bar:42}", ""))
                .is_err());
            assert!(record().parse(stream("Foo{...(foo),bar:42}", "")).is_ok());
            assert!(record()
                .parse(stream("Foo{...foo(bar),bar:42}", ""))
                .is_ok());
            assert!(record()
                .parse(stream("Foo{...if true { none } else { none },bar:42}", ""))
                .is_ok());
        }

        #[test]
        fn parse_variable() {
            assert!(variable().parse(stream("", "")).is_err());

            assert_eq!(
                variable().parse(stream("x", "")).unwrap().0,
                Variable::new("x", Position::dummy()),
            );

            assert_eq!(
                variable().parse(stream("Foo.x", "")).unwrap().0,
                Variable::new("Foo", Position::dummy()),
            );
        }

        #[test]
        fn parse_boolean_literal() {
            assert!(boolean_literal().parse(stream("", "")).is_err());
            assert_eq!(
                boolean_literal().parse(stream("false", "")).unwrap().0,
                Boolean::new(false, Position::dummy())
            );
            assert_eq!(
                boolean_literal().parse(stream("true", "")).unwrap().0,
                Boolean::new(true, Position::dummy())
            );
        }

        #[test]
        fn parse_none_literal() {
            assert!(none_literal().parse(stream("", "")).is_err());
            assert_eq!(
                none_literal().parse(stream("none", "")).unwrap().0,
                None::new(Position::dummy())
            );
        }

        #[test]
        fn parse_number_literal() {
            assert!(number_literal().parse(stream("", "")).is_err());
            assert!(number_literal().parse(stream("foo", "")).is_err());
            assert!(number_literal().parse(stream("01", "")).is_err());

            for (source, value) in &[
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
                (
                    "[none;]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;none]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![ListElement::Single(None::new(Position::dummy()).into())],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;none,]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![ListElement::Single(None::new(Position::dummy()).into())],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;none,none]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![
                            ListElement::Single(None::new(Position::dummy()).into()),
                            ListElement::Single(None::new(Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;none,none,]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![
                            ListElement::Single(None::new(Position::dummy()).into()),
                            ListElement::Single(None::new(Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;...foo]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![ListElement::Multiple(
                            Variable::new("foo", Position::dummy()).into(),
                        )],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;...foo,]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![ListElement::Multiple(
                            Variable::new("foo", Position::dummy()).into(),
                        )],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;...foo,...bar]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![
                            ListElement::Multiple(Variable::new("foo", Position::dummy()).into()),
                            ListElement::Multiple(Variable::new("bar", Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;...foo,...bar,]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![
                            ListElement::Multiple(Variable::new("foo", Position::dummy()).into()),
                            ListElement::Multiple(Variable::new("bar", Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;foo,...bar]",
                    List::new(
                        types::None::new(Position::dummy()),
                        vec![
                            ListElement::Single(Variable::new("foo", Position::dummy()).into()),
                            ListElement::Multiple(Variable::new("bar", Position::dummy()).into()),
                        ],
                        Position::dummy(),
                    ),
                ),
                (
                    "[none;...foo,bar]",
                    List::new(
                        types::None::new(Position::dummy()),
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
        assert!(identifier().parse(stream("if", "")).is_err());
        assert!(identifier().parse(stream("1foo", "")).is_err());
        assert_eq!(
            identifier().parse(stream("foo", "")).unwrap().0,
            "foo".to_string()
        );
        assert_eq!(
            identifier().parse(stream("foo42", "")).unwrap().0,
            "foo42".to_string()
        );
    }

    #[test]
    fn parse_keyword() {
        assert!(keyword("foo").parse(stream("bar", "")).is_err());
        assert!(keyword("fo").parse(stream("foo", "")).is_err());
        assert!(keyword("foo").parse(stream("foo", "")).is_ok());
    }

    #[test]
    fn parse_sign() {
        assert!(sign("+").parse(stream("", "")).is_err());
        assert!(sign("+").parse(stream("-", "")).is_err());
        assert!(sign("+").parse(stream("+", "")).is_ok());
        assert!(sign("++").parse(stream("++", "")).is_ok());
    }

    #[test]
    fn parse_position() {
        assert!(position().parse(stream("", "")).is_ok());
    }

    #[test]
    fn parse_blank() {
        assert!(blank().with(eof()).parse(stream(" ", "")).is_ok());
        assert!(blank().with(eof()).parse(stream("\n", "")).is_ok());
        assert!(blank().with(eof()).parse(stream(" \n", "")).is_ok());
        assert!(blank().with(eof()).parse(stream("\t", "")).is_ok());
        assert!(blank().with(eof()).parse(stream("# foo", "")).is_ok());
    }

    #[test]
    fn parse_comment() {
        assert!(comment().parse(stream("#", "")).is_ok());
        assert!(comment().parse(stream("#\n", "")).is_ok());
        assert!(comment().parse(stream("#x\n", "")).is_ok());
    }
}
