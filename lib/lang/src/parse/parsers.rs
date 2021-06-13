use super::{
    attempt::{many, many1, optional, sep_end_by, sep_end_by1},
    utilities::*,
};
use crate::{
    ast::*,
    debug::*,
    path::*,
    types::{self, Type},
};
use combine::{
    easy, from_str, none_of, one_of,
    parser::{
        char::{alpha_num, char as character, letter, string},
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

const KEYWORDS: &[&str] = &[
    "case", "else", "export", "foreign", "if", "import", "in", "let", "then", "type",
];
const OPERATOR_CHARACTERS: &str = "+-*/=<>&|";
const SPACE_CHARACTERS: &str = " \t\r";

static NUMBER_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^-?([123456789][0123456789]*|0)(\.[0123456789]+)?").unwrap());
static STRING_REGEX: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r#"^[^\\"]"#).unwrap());

pub struct State<'a> {
    source_name: &'a str,
    lines: Vec<&'a str>,
}

pub type Stream<'a> =
    easy::Stream<state::Stream<position::Stream<&'a str, SourcePosition>, State<'a>>>;

pub fn stream<'a>(source: &'a str, source_name: &'a str) -> Stream<'a> {
    state::Stream {
        stream: position::Stream::new(source),
        state: State {
            source_name,
            lines: source.split('\n').collect(),
        },
    }
    .into()
}

pub fn module<'a>() -> impl Parser<Stream<'a>, Output = UnresolvedModule> {
    (
        optional(export()),
        optional(export_foreign()),
        many(import()),
        many(import_foreign()),
        many(type_definition()),
        many(definition()),
    )
        .skip(blank())
        .skip(eof())
        .map(
            |(export, export_foreign, imports, import_foreigns, type_definitions, definitions)| {
                UnresolvedModule::new(
                    export.unwrap_or_else(|| Export::new(Default::default())),
                    export_foreign.unwrap_or_else(|| ExportForeign::new(Default::default())),
                    imports,
                    import_foreigns,
                    type_definitions,
                    definitions,
                )
            },
        )
}

fn export<'a>() -> impl Parser<Stream<'a>, Output = Export> {
    keyword("export")
        .with(between(
            sign("{"),
            sign("}"),
            sep_end_by1(identifier(), sign(",")),
        ))
        .map(Export::new)
        .expected("export statement")
}

fn import<'a>() -> impl Parser<Stream<'a>, Output = UnresolvedImport> {
    keyword("import")
        .with(module_path())
        .map(UnresolvedImport::new)
        .expected("import statement")
}

fn module_path<'a>() -> impl Parser<Stream<'a>, Output = UnresolvedModulePath> {
    token(choice!(
        internal_module_path().map(UnresolvedModulePath::from),
        external_module_path().map(UnresolvedModulePath::from),
    ))
    .expected("module path")
}

fn internal_module_path<'a>() -> impl Parser<Stream<'a>, Output = InternalUnresolvedModulePath> {
    module_path_components().map(InternalUnresolvedModulePath::new)
}

fn external_module_path<'a>() -> impl Parser<Stream<'a>, Output = ExternalUnresolvedModulePath> {
    (identifier(), module_path_components()).map(|(package_name, path_components)| {
        ExternalUnresolvedModulePath::new(package_name, path_components)
    })
}

fn module_path_components<'a>() -> impl Parser<Stream<'a>, Output = Vec<String>> {
    many1(string(".").with(identifier()))
}

fn export_foreign<'a>() -> impl Parser<Stream<'a>, Output = ExportForeign> {
    (keyword("export"), keyword("foreign"))
        .with(between(
            sign("{"),
            sign("}"),
            sep_end_by1(identifier(), sign(",")),
        ))
        .map(ExportForeign::new)
        .expected("export foreign statement")
}

fn import_foreign<'a>() -> impl Parser<Stream<'a>, Output = ImportForeign> {
    (
        source_information(),
        keyword("import"),
        keyword("foreign"),
        optional(calling_convention()),
        type_annotation(),
    )
        .map(
            |(source_information, _, _, calling_convention, (name, type_))| {
                ImportForeign::new(
                    &name,
                    &name,
                    calling_convention.unwrap_or(CallingConvention::Native),
                    type_,
                    source_information,
                )
            },
        )
        .expected("import foreign")
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
    choice!(
        function_definition().map(Definition::from),
        variable_definition().map(Definition::from),
    )
    .expected("definition")
}

fn function_definition<'a>() -> impl Parser<Stream<'a>, Output = FunctionDefinition> {
    (
        source_information(),
        type_annotation(),
        identifier(),
        many1(identifier()),
        sign("="),
        expression(),
    )
        .then(
            |(source_information, (typed_name, type_), name, arguments, _, expression)| {
                if typed_name == name {
                    value(FunctionDefinition::new(
                        name,
                        arguments,
                        expression,
                        type_,
                        source_information,
                    ))
                    .left()
                } else {
                    unexpected_any("unmatched identifiers in definition").right()
                }
            },
        )
}

fn variable_definition<'a>() -> impl Parser<Stream<'a>, Output = VariableDefinition> {
    (
        source_information(),
        type_annotation(),
        identifier(),
        sign("="),
        expression(),
    )
        .then(
            |(source_information, (typed_name, type_), name, _, expression)| {
                if typed_name == name {
                    value(VariableDefinition::new(
                        name,
                        expression,
                        type_,
                        source_information,
                    ))
                    .left()
                } else {
                    unexpected_any("unmatched identifiers in definition").right()
                }
            },
        )
}

fn result_definition<'a>() -> impl Parser<Stream<'a>, Output = VariableDefinition> {
    (
        source_information(),
        type_annotation(),
        identifier(),
        sign("?="),
        expression(),
    )
        .then(
            |(source_information, (typed_name, type_), name, _, expression)| {
                if typed_name == name {
                    value(VariableDefinition::new(
                        name,
                        expression,
                        type_,
                        source_information,
                    ))
                    .left()
                } else {
                    unexpected_any("unmatched identifiers in definition").right()
                }
            },
        )
}

fn type_annotation<'a>() -> impl Parser<Stream<'a>, Output = (String, Type)> {
    (identifier(), sign(":").with(type_()))
}

fn untyped_function_definition<'a>() -> impl Parser<Stream<'a>, Output = FunctionDefinition> {
    (
        source_information(),
        identifier(),
        many1(identifier()),
        sign("="),
        expression(),
    )
        .map(|(source_information, name, arguments, _, expression)| {
            let source_information = Arc::new(source_information);
            FunctionDefinition::new(
                name,
                arguments,
                expression,
                types::Unknown::new(source_information.clone()),
                source_information,
            )
        })
}

fn untyped_variable_definition<'a>() -> impl Parser<Stream<'a>, Output = VariableDefinition> {
    (source_information(), identifier(), sign("="), expression()).map(
        |(source_information, name, _, expression)| {
            let source_information = Arc::new(source_information);
            VariableDefinition::new(
                name,
                expression,
                types::Unknown::new(source_information.clone()),
                source_information,
            )
        },
    )
}

fn untyped_result_definition<'a>() -> impl Parser<Stream<'a>, Output = VariableDefinition> {
    (source_information(), identifier(), sign("?="), expression()).map(
        |(source_information, name, _, expression)| {
            let source_information = Arc::new(source_information);
            VariableDefinition::new(
                name,
                expression,
                types::Unknown::new(source_information.clone()),
                source_information,
            )
        },
    )
}

fn type_definition<'a>() -> impl Parser<Stream<'a>, Output = TypeDefinition> {
    choice!(type_alias_definition(), record_type_definition()).expected("type definition")
}

fn record_type_definition<'a>() -> impl Parser<Stream<'a>, Output = TypeDefinition> {
    (
        keyword("type"),
        source_information(),
        identifier(),
        optional(between(
            sign("{"),
            sign("}"),
            sep_end_by1((identifier().skip(sign(":")), type_()), sign(",")),
        )),
    )
        .map(
            |(_, source_information, name, elements): (_, _, _, Option<Vec<_>>)| {
                TypeDefinition::new(
                    &name,
                    types::Record::new(
                        &name,
                        elements
                            .unwrap_or_default()
                            .into_iter()
                            .map(|(name, type_)| types::RecordElement::new(name, type_))
                            .collect(),
                        source_information,
                    ),
                )
            },
        )
        .expected("record type definition")
}

fn type_alias_definition<'a>() -> impl Parser<Stream<'a>, Output = TypeDefinition> {
    (keyword("type"), identifier(), sign("="), type_())
        .map(|(_, name, _, type_)| TypeDefinition::new(&name, type_))
        .expected("type alias definition")
}

fn type_<'a>() -> impl Parser<Stream<'a>, Output = Type> {
    lazy(|| no_partial(choice!(function_type().map(Type::from), union_type())))
        .boxed()
        .expected("type")
}

fn function_type<'a>() -> impl Parser<Stream<'a>, Output = types::Function> {
    (source_information(), union_type(), sign("->"), type_())
        .map(|(source_information, argument, _, result)| {
            types::Function::new(argument, result, source_information)
        })
        .expected("function type")
}

fn union_type<'a>() -> impl Parser<Stream<'a>, Output = Type> {
    (
        source_information(),
        sep_end_by1(type_application(), sign("|")),
    )
        .map(|(source_information, types)| {
            let types: Vec<_> = types;

            if types.len() == 1 {
                types[0].clone()
            } else {
                types::Union::new(types, source_information).into()
            }
        })
        .expected("union type")
}

fn type_application<'a>() -> impl Parser<Stream<'a>, Output = Type> {
    choice!(list_type().map(Type::from), atomic_type())
}

fn list_type<'a>() -> impl Parser<Stream<'a>, Output = types::List> {
    (source_information(), keyword("List"), atomic_type())
        .map(|(source_information, _, element)| types::List::new(element, source_information))
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
        between(sign("("), sign(")"), type_()),
    )
}

fn boolean_type<'a>() -> impl Parser<Stream<'a>, Output = types::Boolean> {
    source_information()
        .skip(keyword("Boolean"))
        .map(types::Boolean::new)
        .expected("boolean type")
}

fn none_type<'a>() -> impl Parser<Stream<'a>, Output = types::None> {
    source_information()
        .skip(keyword("None"))
        .map(types::None::new)
        .expected("none type")
}

fn number_type<'a>() -> impl Parser<Stream<'a>, Output = types::Number> {
    source_information()
        .skip(keyword("Number"))
        .map(types::Number::new)
        .expected("number type")
}

fn string_type<'a>() -> impl Parser<Stream<'a>, Output = types::ByteString> {
    source_information()
        .skip(keyword("String"))
        .map(types::ByteString::new)
        .expected("string type")
}

fn any_type<'a>() -> impl Parser<Stream<'a>, Output = types::Any> {
    source_information()
        .skip(keyword("Any"))
        .map(types::Any::new)
        .expected("any type")
}

fn reference_type<'a>() -> impl Parser<Stream<'a>, Output = types::Reference> {
    (source_information(), qualified_identifier())
        .map(|(source_information, identifier)| {
            types::Reference::new(identifier, source_information)
        })
        .expected("reference type")
}

fn expression<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    lazy(|| no_partial(operation_or_term()))
        .boxed()
        .expected("expression")
}

fn atomic_expression<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    lazy(|| no_partial(strict_atomic_expression())).boxed()
}

fn strict_atomic_expression<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    choice!(
        record_construction().map(Expression::from),
        record_update().map(Expression::from),
        list_literal().map(Expression::from),
        boolean_literal().map(Expression::from),
        none_literal().map(Expression::from),
        number_literal().map(Expression::from),
        string_literal().map(Expression::from),
        variable().map(Expression::from),
        between(sign("("), sign(")"), expression()),
    )
}

fn if_<'a>() -> impl Parser<Stream<'a>, Output = If> {
    (
        source_information(),
        keyword("if").expected("if keyword"),
        expression(),
        keyword("then").expected("then keyword"),
        expression(),
        keyword("else").expected("else keyword"),
        expression(),
    )
        .map(|(source_information, _, condition, _, then, _, else_)| {
            If::new(condition, then, else_, source_information)
        })
        .expected("if expression")
}

fn case<'a>() -> impl Parser<Stream<'a>, Output = Case> {
    (
        source_information(),
        keyword("case").expected("case keyword"),
        identifier(),
        sign("="),
        expression(),
        many1(alternative()),
    )
        .map(
            |(source_information, _, identifier, _, argument, alternatives)| {
                Case::new(identifier, argument, alternatives, source_information)
            },
        )
        .expected("type case expression")
}

fn list_case<'a>() -> impl Parser<Stream<'a>, Output = ListCase> {
    (
        source_information(),
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
                source_information,
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
                ListCase::new(
                    argument,
                    types::Unknown::new(source_information.clone()),
                    first_name,
                    rest_name,
                    empty_alternative,
                    non_empty_alternative,
                    source_information,
                )
            },
        )
        .expected("list case expression")
}

fn alternative<'a>() -> impl Parser<Stream<'a>, Output = Alternative> {
    (type_(), sign("=>"), expression())
        .map(|(type_, _, expression)| Alternative::new(type_, expression))
}

fn let_<'a>() -> impl Parser<Stream<'a>, Output = Let> {
    (
        source_information(),
        keyword("let").expected("let keyword"),
        many1(choice!(
            variable_definition().map(From::from),
            untyped_variable_definition().map(From::from),
            function_definition().map(From::from),
            untyped_function_definition().map(From::from),
        )),
        keyword("in").expected("in keyword"),
        expression(),
    )
        .map(|(source_information, _, definitions, _, expression)| {
            Let::new(definitions, expression, source_information)
        })
        .expected("let expression")
}

fn let_error<'a>() -> impl Parser<Stream<'a>, Output = LetError> {
    (
        source_information(),
        keyword("let").expected("let keyword"),
        many1(choice!(result_definition(), untyped_result_definition())),
        keyword("in").expected("in keyword"),
        expression(),
    )
        .map(|(source_information, _, definitions, _, expression)| {
            LetError::new(definitions, expression, source_information)
        })
        .expected("let-error expression")
}

fn application_or_atomic_expression<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    (
        source_information(),
        atomic_expression(),
        many((
            many(atomic_expression().skip(not_followed_by(application_terminator()))),
            atomic_expression().skip(look_ahead(application_terminator())),
        )),
    )
        .map(|(source_information, function, argument_sets)| {
            let source_information = Arc::new(source_information);
            let argument_sets: Vec<(Vec<Expression>, _)> = argument_sets;

            let mut all_arguments = vec![];

            for (arguments, argument) in argument_sets {
                all_arguments.extend(arguments);
                all_arguments.push(argument);
            }

            all_arguments
                .into_iter()
                .fold(function, |application, argument| {
                    Application::new(application, argument, source_information.clone()).into()
                })
        })
        .expected("application")
}

fn application_terminator<'a>() -> impl Parser<Stream<'a>, Output = &'static str> {
    choice!(
        newlines1(),
        sign(","),
        sign(")"),
        sign("}"),
        sign("]"),
        operator().with(value(())),
        any_keyword(),
    )
    .with(value("application terminator"))
    .expected("application terminator")
}

fn record_construction<'a>() -> impl Parser<Stream<'a>, Output = RecordConstruction> {
    (
        source_information(),
        reference_type(),
        string("{"),
        sep_end_by1((identifier().skip(sign("=")), expression()), sign(",")),
        sign("}"),
    )
        .then(|(source_information, reference_type, _, elements, _)| {
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
                    source_information,
                ))
                .left()
            } else {
                unexpected_any("duplicate keys in record construction").right()
            }
        })
        .expected("record construction")
}

fn record_update<'a>() -> impl Parser<Stream<'a>, Output = RecordUpdate> {
    (
        source_information(),
        reference_type(),
        string("{"),
        sign("..."),
        atomic_expression(),
        sign(","),
        sep_end_by1((identifier().skip(sign("=")), expression()), sign(",")),
        sign("}"),
    )
        .then(
            |(source_information, reference_type, _, _, argument, _, elements, _)| {
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
                        argument,
                        elements.into_iter().collect(),
                        source_information,
                    ))
                    .left()
                } else {
                    unexpected_any("duplicate keys in record update").right()
                }
            },
        )
}

fn term<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    choice!(
        application_or_atomic_expression(),
        if_().map(Expression::from),
        case().map(Expression::from),
        list_case().map(Expression::from),
        let_().map(Expression::from),
        let_error().map(Expression::from),
        let_().map(Expression::from),
    )
}

fn operation_or_term<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    (
        term(),
        many((source_information(), operator(), term()).map(
            |(source_information, operator, expression)| (operator, expression, source_information),
        )),
    )
        .map(|(expression, pairs): (_, Vec<_>)| reduce_operations(expression, &pairs))
}

fn operator<'a>() -> impl Parser<Stream<'a>, Output = ParsedOperator> {
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
        concrete_operator("&&", ParsedOperator::And),
        concrete_operator("||", ParsedOperator::Or),
    )
    .expected("operator")
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
        source_information()
            .skip(keyword("False"))
            .map(|source_information| Boolean::new(false, source_information)),
        source_information()
            .skip(keyword("True"))
            .map(|source_information| Boolean::new(true, source_information)),
    ))
    .expected("boolean literal")
}

fn none_literal<'a>() -> impl Parser<Stream<'a>, Output = None> {
    token(source_information().skip(keyword("None")))
        .map(None::new)
        .expected("none literal")
}

fn number_literal<'a>() -> impl Parser<Stream<'a>, Output = Number> {
    let regex: &'static regex::Regex = &NUMBER_REGEX;

    token((source_information(), from_str(find(regex))))
        .map(|(source_information, number)| Number::new(number, source_information))
        .expected("number literal")
}

fn string_literal<'a>() -> impl Parser<Stream<'a>, Output = ByteString> {
    let regex: &'static regex::Regex = &STRING_REGEX;

    token((
        source_information(),
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
    .map(
        |(source_information, _, strings, _): (_, _, Vec<String>, _)| {
            ByteString::new(strings.join(""), source_information)
        },
    )
    .expected("string literal")
}

fn list_literal<'a>() -> impl Parser<Stream<'a>, Output = List> {
    (
        source_information(),
        between(sign("["), sign("]"), sep_end_by(list_element(), sign(","))),
    )
        .map(|(source_information, elements)| List::new(elements, source_information))
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
    token((source_information(), qualified_identifier()))
        .map(|(source_information, identifier)| Variable::new(identifier, source_information))
        .expected("variable")
}

fn qualified_identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    (optional((raw_identifier(), string("."))), raw_identifier()).map(|(prefix, identifier)| {
        prefix
            .map(|(prefix, _)| [&prefix, ".", &identifier].concat())
            .unwrap_or(identifier)
    })
}

fn identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    token(raw_identifier()).expected("identifier")
}

fn raw_identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    unchecked_identifier().then(|identifier| {
        if KEYWORDS.contains(&identifier.as_str()) {
            unexpected_any("keyword").left()
        } else {
            value(identifier).right()
        }
    })
}

fn unchecked_identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    choice!(
        (letter(), many(alpha_num())).boxed(),
        (character('_'), many(choice!(alpha_num(), character('_')))).boxed(),
    )
    .map(|(head, tail): (char, String)| [head.into(), tail].concat())
}

fn keyword<'a>(name: &'static str) -> impl Parser<Stream<'a>, Output = ()> {
    token(string(name).skip(not_followed_by(alpha_num())))
        .with(value(()))
        .expected("keyword")
}

fn any_keyword<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    token(unchecked_identifier().then(|keyword| {
        if KEYWORDS.contains(&keyword.as_str()) {
            value(()).left()
        } else {
            unexpected_any("non-keyword").right()
        }
    }))
}

fn sign<'a>(sign: &'static str) -> impl Parser<Stream<'a>, Output = ()> {
    token(string(sign)).with(value(())).expected(sign)
}

fn token<'a, O, P: Parser<Stream<'a>, Output = O>>(p: P) -> impl Parser<Stream<'a>, Output = O> {
    blank().with(p)
}

fn source_information<'a>() -> impl Parser<Stream<'a>, Output = SourceInformation> {
    blank()
        .map_input(|_, stream: &mut Stream<'a>| {
            let position = stream.position();
            SourceInformation::new(
                stream.0.state.source_name,
                Location::new(position.line as usize, position.column as usize),
                stream.0.state.lines[position.line as usize - 1],
            )
        })
        .expected("source information")
}

fn blank<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    many::<Vec<_>, _, _>(choice!(spaces1(), newline()))
        .with(value(()))
        .expected("blank")
}

fn spaces1<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    many1::<String, _, _>(one_of(SPACE_CHARACTERS.chars())).with(value(()))
}

fn newlines1<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    choice!(
        many1(newline()),
        many::<Vec<_>, _, _>(newline()).with(eof()),
    )
}

fn newline<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    optional(spaces1())
        .with(choice!(
            combine::parser::char::newline().with(value(())),
            comment(),
        ))
        .expected("newline")
}

fn eof<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    optional(spaces1())
        .with(combine::eof())
        .expected("end of file")
}

fn comment<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    string("#")
        .with(many::<Vec<_>, _, _>(none_of("\n".chars())))
        .with(combine::parser::char::newline())
        .with(value(()))
        .expected("comment")
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_module() {
        assert_eq!(
            module().parse(stream("", "")).unwrap().0,
            UnresolvedModule::from_definitions(vec![])
        );
        assert_eq!(
            module().parse(stream(" ", "")).unwrap().0,
            UnresolvedModule::from_definitions(vec![])
        );
        assert_eq!(
            module().parse(stream("\n", "")).unwrap().0,
            UnresolvedModule::from_definitions(vec![])
        );
        assert_eq!(
            module().parse(stream("export { foo }", "")).unwrap().0,
            UnresolvedModule::new(
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
            UnresolvedModule::new(
                Export::new(vec!["foo".into()].drain(..).collect()),
                ExportForeign::new(Default::default()),
                vec![UnresolvedImport::new(ExternalUnresolvedModulePath::new(
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
            UnresolvedModule::new(
                Export::new(Default::default()),
                ExportForeign::new(Default::default()),
                vec![],
                vec![],
                vec![],
                vec![VariableDefinition::new(
                    "x",
                    Number::new(42.0, SourceInformation::dummy()),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into()]
            )
        );
        assert_eq!(
            module()
                .parse(stream("x : Number\nx = 42\ny : Number\ny = 42", ""))
                .unwrap()
                .0,
            UnresolvedModule::new(
                Export::new(Default::default()),
                ExportForeign::new(Default::default()),
                vec![],
                vec![],
                vec![],
                vec![
                    VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into(),
                    VariableDefinition::new(
                        "y",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
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
            UnresolvedModule::new(
                Export::new(Default::default()),
                ExportForeign::new(Default::default()),
                vec![],
                vec![],
                vec![],
                vec![FunctionDefinition::new(
                    "main",
                    vec!["x".into()],
                    Number::new(42.0, SourceInformation::dummy()),
                    types::Function::new(
                        types::Number::new(SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
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
            UnresolvedImport::new(InternalUnresolvedModulePath::new(vec!["Foo".into()])),
        );
        assert_eq!(
            import().parse(stream("import Foo.Bar", "")).unwrap().0,
            UnresolvedImport::new(ExternalUnresolvedModulePath::new("Foo", vec!["Bar".into()])),
        );
    }

    #[test]
    fn parse_module_path() {
        assert!(module_path().parse(stream("?", "")).is_err());
        assert_eq!(
            module_path().parse(stream(".Foo", "")).unwrap().0,
            UnresolvedModulePath::Internal(InternalUnresolvedModulePath::new(vec!["Foo".into()])),
        );
        assert_eq!(
            module_path().parse(stream("Foo.Bar", "")).unwrap().0,
            UnresolvedModulePath::External(ExternalUnresolvedModulePath::new(
                "Foo",
                vec!["Bar".into()]
            )),
        );
        assert_eq!(
            module_path().parse(stream(" .Foo", "")).unwrap().0,
            UnresolvedModulePath::Internal(InternalUnresolvedModulePath::new(vec!["Foo".into()])),
        );
    }

    #[test]
    fn parse_internal_module_path() {
        assert!(internal_module_path().parse(stream("?", "")).is_err());
        assert_eq!(
            internal_module_path().parse(stream(".Foo", "")).unwrap().0,
            InternalUnresolvedModulePath::new(vec!["Foo".into()]),
        );
        assert_eq!(
            internal_module_path()
                .parse(stream(".Foo.Bar", ""))
                .unwrap()
                .0,
            InternalUnresolvedModulePath::new(vec!["Foo".into(), "Bar".into()]),
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
            ExternalUnresolvedModulePath::new("Foo", vec!["Bar".into()]),
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
                    types::Number::new(SourceInformation::dummy()),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
                ),
                SourceInformation::dummy()
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
                    types::Number::new(SourceInformation::dummy()),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
                ),
                SourceInformation::dummy()
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
                Number::new(0.0, SourceInformation::dummy()),
                types::Number::new(SourceInformation::dummy()),
                SourceInformation::dummy()
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
                Number::new(42.0, SourceInformation::dummy()),
                types::Function::new(
                    types::Number::new(SourceInformation::dummy()),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
                ),
                SourceInformation::dummy()
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
                Number::new(0.0, SourceInformation::dummy()),
                types::Number::new(SourceInformation::dummy()),
                SourceInformation::dummy()
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
                Number::new(0.0, SourceInformation::dummy()),
                types::Unknown::new(SourceInformation::dummy()),
                SourceInformation::dummy()
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
                Number::new(42.0, SourceInformation::dummy()),
                types::Unknown::new(SourceInformation::dummy()),
                SourceInformation::dummy()
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
                    Variable::new("x", SourceInformation::dummy()),
                    types::Unknown::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
                ),
                VariableDefinition::new(
                    "y",
                    Application::new(
                        Variable::new("f", SourceInformation::dummy()),
                        Variable::new("x", SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    types::Unknown::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
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
                    types::Record::new("Foo", Default::default(), SourceInformation::dummy()),
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
                            types::Number::new(SourceInformation::dummy()),
                        )],
                        SourceInformation::dummy(),
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
                            types::Number::new(SourceInformation::dummy()),
                        )],
                        SourceInformation::dummy(),
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
                            types::RecordElement::new(
                                "foo",
                                types::Number::new(SourceInformation::dummy()),
                            ),
                            types::RecordElement::new(
                                "bar",
                                types::Number::new(SourceInformation::dummy()),
                            ),
                        ],
                        SourceInformation::dummy(),
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
                            types::RecordElement::new(
                                "foo",
                                types::Number::new(SourceInformation::dummy()),
                            ),
                            types::RecordElement::new(
                                "bar",
                                types::Number::new(SourceInformation::dummy()),
                            ),
                        ],
                        SourceInformation::dummy(),
                    ),
                ),
            ),
            (
                "type Foo = Boolean | None",
                TypeDefinition::new(
                    "Foo",
                    types::Union::new(
                        vec![
                            types::Boolean::new(SourceInformation::dummy()).into(),
                            types::None::new(SourceInformation::dummy()).into(),
                        ]
                        .into_iter()
                        .collect(),
                        SourceInformation::dummy(),
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
                TypeDefinition::new("Foo", types::Number::new(SourceInformation::dummy())),
            ),
            (
                "type Foo = Number | None",
                TypeDefinition::new(
                    "Foo",
                    types::Union::new(
                        vec![
                            types::Number::new(SourceInformation::dummy()).into(),
                            types::None::new(SourceInformation::dummy()).into(),
                        ]
                        .into_iter()
                        .collect(),
                        SourceInformation::dummy(),
                    ),
                ),
            ),
        ] {
            assert_eq!(
                &type_alias_definition().parse(stream(source, "")).unwrap().0,
                expected
            );
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
                types::Boolean::new(SourceInformation::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("None", "")).unwrap().0,
                types::None::new(SourceInformation::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("Number", "")).unwrap().0,
                types::Number::new(SourceInformation::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("Number -> Number", "")).unwrap().0,
                types::Function::new(
                    types::Number::new(SourceInformation::dummy()),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("Number -> Number -> Number", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    types::Number::new(SourceInformation::dummy()),
                    types::Function::new(
                        types::Number::new(SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
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
                        types::Number::new(SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into()
            );
            assert_eq!(
                type_().parse(stream("Number | None", "")).unwrap().0,
                types::Union::new(
                    vec![
                        types::Number::new(SourceInformation::dummy()).into(),
                        types::None::new(SourceInformation::dummy()).into(),
                    ],
                    SourceInformation::dummy()
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
                        types::Boolean::new(SourceInformation::dummy()).into(),
                        types::Number::new(SourceInformation::dummy()).into(),
                        types::None::new(SourceInformation::dummy()).into(),
                    ],
                    SourceInformation::dummy()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("Number -> Number | None", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    types::Number::new(SourceInformation::dummy()),
                    types::Union::new(
                        vec![
                            types::Number::new(SourceInformation::dummy()).into(),
                            types::None::new(SourceInformation::dummy()).into(),
                        ],
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
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
                            types::Number::new(SourceInformation::dummy()).into(),
                            types::None::new(SourceInformation::dummy()).into(),
                        ],
                        SourceInformation::dummy()
                    ),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
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
                            types::Number::new(SourceInformation::dummy()),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into(),
                        types::None::new(SourceInformation::dummy()).into(),
                    ],
                    SourceInformation::dummy()
                )
                .into()
            );
        }

        #[test]
        fn parse_any_type() {
            assert_eq!(
                any_type().parse(stream("Any", "")).unwrap().0,
                types::Any::new(SourceInformation::dummy())
            );
        }

        #[test]
        fn parse_reference_type() {
            assert!(type_().parse(stream("", "")).is_err());
            assert_eq!(
                type_().parse(stream("Foo", "")).unwrap().0,
                types::Reference::new("Foo", SourceInformation::dummy()).into()
            );
            assert_eq!(
                type_().parse(stream("Foo.Bar", "")).unwrap().0,
                types::Reference::new("Foo.Bar", SourceInformation::dummy()).into()
            );
        }

        #[test]
        fn parse_list_type() {
            assert_eq!(
                type_().parse(stream("List Number", "")).unwrap().0,
                types::List::new(
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into()
            );

            assert_eq!(
                type_().parse(stream("List (List Number)", "")).unwrap().0,
                types::List::new(
                    types::List::new(
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
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
                        types::List::new(
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into(),
                        types::List::new(
                            types::None::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into()
                    ],
                    SourceInformation::dummy()
                )
                .into()
            );

            assert_eq!(
                type_()
                    .parse(stream("List Number -> List None", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    types::List::new(
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    types::List::new(
                        types::None::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
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
                Number::new(1.0, SourceInformation::dummy()).into()
            );
            assert_eq!(
                expression().parse(stream("x", "")).unwrap().0,
                Variable::new("x", SourceInformation::dummy()).into()
            );
            assert_eq!(
                expression().parse(stream("x + 1", "")).unwrap().0,
                ArithmeticOperation::new(
                    ArithmeticOperator::Add,
                    Variable::new("x", SourceInformation::dummy()),
                    Number::new(1.0, SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("x + y z", "")).unwrap().0,
                ArithmeticOperation::new(
                    ArithmeticOperator::Add,
                    Variable::new("x", SourceInformation::dummy()),
                    Application::new(
                        Variable::new("y", SourceInformation::dummy()),
                        Variable::new("z", SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("(x + y) z", "")).unwrap().0,
                Application::new(
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Variable::new("x", SourceInformation::dummy()),
                        Variable::new("y", SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    Variable::new("z", SourceInformation::dummy()),
                    SourceInformation::dummy()
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
                Application::new(
                    Variable::new("f", SourceInformation::dummy()),
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
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
                Number::new(42.0, SourceInformation::dummy()).into()
            )
        }

        #[test]
        fn parse_atomic_expression() {
            assert!(atomic_expression().parse(stream("?", "")).is_err());
            assert_eq!(
                atomic_expression().parse(stream("1", "")).unwrap().0,
                Number::new(1.0, SourceInformation::dummy()).into()
            );
            assert_eq!(
                atomic_expression().parse(stream("x", "")).unwrap().0,
                Variable::new("x", SourceInformation::dummy()).into()
            );
            assert_eq!(
                atomic_expression().parse(stream(" x", "")).unwrap().0,
                Variable::new("x", SourceInformation::dummy()).into()
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
                    Boolean::new(true, SourceInformation::dummy()),
                    Number::new(42.0, SourceInformation::dummy()),
                    Number::new(13.0, SourceInformation::dummy()),
                    SourceInformation::dummy(),
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
                        Boolean::new(true, SourceInformation::dummy()),
                        Boolean::new(false, SourceInformation::dummy()),
                        Boolean::new(true, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    ),
                    Number::new(42.0, SourceInformation::dummy()),
                    Number::new(13.0, SourceInformation::dummy()),
                    SourceInformation::dummy(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream("if True then if False then 1 else 2 else 3", ""))
                    .unwrap()
                    .0,
                If::new(
                    Boolean::new(true, SourceInformation::dummy()),
                    If::new(
                        Boolean::new(false, SourceInformation::dummy()),
                        Number::new(1.0, SourceInformation::dummy()),
                        Number::new(2.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    ),
                    Number::new(3.0, SourceInformation::dummy()),
                    SourceInformation::dummy(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream("if True then 1 else if False then 2 else 3", ""))
                    .unwrap()
                    .0,
                If::new(
                    Boolean::new(true, SourceInformation::dummy()),
                    Number::new(1.0, SourceInformation::dummy()),
                    If::new(
                        Boolean::new(false, SourceInformation::dummy()),
                        Number::new(2.0, SourceInformation::dummy()),
                        Number::new(3.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    ),
                    SourceInformation::dummy(),
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
                        Variable::new("x", SourceInformation::dummy()),
                        Number::new(0.0, SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    Number::new(42.0, SourceInformation::dummy()),
                    Number::new(13.0, SourceInformation::dummy()),
                    SourceInformation::dummy(),
                )
            );
        }

        #[test]
        fn parse_case() {
            assert_eq!(
                case()
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
                    Boolean::new(true, SourceInformation::dummy()),
                    vec![Alternative::new(
                        types::Boolean::new(SourceInformation::dummy()),
                        Variable::new("foo", SourceInformation::dummy())
                    )],
                    SourceInformation::dummy(),
                )
            );
            assert_eq!(
                case()
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
                    Boolean::new(true, SourceInformation::dummy()),
                    vec![
                        Alternative::new(
                            types::Boolean::new(SourceInformation::dummy()),
                            Boolean::new(true, SourceInformation::dummy())
                        ),
                        Alternative::new(
                            types::None::new(SourceInformation::dummy()),
                            Boolean::new(false, SourceInformation::dummy())
                        )
                    ],
                    SourceInformation::dummy()
                )
            );
        }

        #[test]
        fn parse_list_case() {
            assert_eq!(
                list_case()
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
                    Variable::new("xs", SourceInformation::dummy()),
                    types::Unknown::new(SourceInformation::dummy()),
                    "x",
                    "xs",
                    None::new(SourceInformation::dummy()),
                    None::new(SourceInformation::dummy()),
                    SourceInformation::dummy(),
                )
            );
        }

        #[test]
        fn parse_let() {
            assert!(let_().parse(stream("let in 0", "")).is_err());
            assert_eq!(
                let_()
                    .parse(stream("let x : Number\nx = 42 in x", ""))
                    .unwrap()
                    .0,
                Let::new(
                    vec![VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_().parse(stream("let x = 42 in x", "")).unwrap().0,
                Let::new(
                    vec![VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Unknown::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_().parse(stream("let\nx = 42 in x", "")).unwrap().0,
                Let::new(
                    vec![VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Unknown::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_().parse(stream("let\n x = 42 in x", "")).unwrap().0,
                Let::new(
                    vec![VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Unknown::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_()
                    .parse(stream("let\nx = 42\ny = 42\nin x", ""))
                    .unwrap()
                    .0,
                Let::new(
                    vec![
                        VariableDefinition::new(
                            "x",
                            Number::new(42.0, SourceInformation::dummy()),
                            types::Unknown::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into(),
                        VariableDefinition::new(
                            "y",
                            Number::new(42.0, SourceInformation::dummy()),
                            types::Unknown::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into(),
                    ],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
        }

        #[test]
        fn parse_let_error() {
            assert!(let_error().parse(stream("let in 0", "")).is_err());
            assert_eq!(
                let_error()
                    .parse(stream("let x : Number\nx ?= 42 in x", ""))
                    .unwrap()
                    .0,
                LetError::new(
                    vec![VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_error().parse(stream("let x ?= 42 in x", "")).unwrap().0,
                LetError::new(
                    vec![VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Unknown::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_error()
                    .parse(stream("let\nx ?= 42 in x", ""))
                    .unwrap()
                    .0,
                LetError::new(
                    vec![VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Unknown::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_error()
                    .parse(stream("let\n x ?= 42 in x", ""))
                    .unwrap()
                    .0,
                LetError::new(
                    vec![VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Unknown::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_error()
                    .parse(stream("let\nx ?= 42\ny ?= 42\nin x", ""))
                    .unwrap()
                    .0,
                LetError::new(
                    vec![
                        VariableDefinition::new(
                            "x",
                            Number::new(42.0, SourceInformation::dummy()),
                            types::Unknown::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        ),
                        VariableDefinition::new(
                            "y",
                            Number::new(42.0, SourceInformation::dummy()),
                            types::Unknown::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                    ],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
        }

        #[test]
        fn parse_let_with_function_definitions() {
            assert!(let_().parse(stream("let in 0", "")).is_err());
            assert_eq!(
                let_().parse(stream("let f x = x in f", "")).unwrap().0,
                Let::new(
                    vec![FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        Variable::new("x", SourceInformation::dummy()),
                        types::Unknown::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("f", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_()
                    .parse(stream(
                        indoc!(
                            "
                        let
                            f x = x
                            g x = (
                                f x
                            )
                        in
                            g
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Let::new(
                    vec![
                        FunctionDefinition::new(
                            "f",
                            vec!["x".into()],
                            Variable::new("x", SourceInformation::dummy()),
                            types::Unknown::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into(),
                        FunctionDefinition::new(
                            "g",
                            vec!["x".into()],
                            Application::new(
                                Variable::new("f", SourceInformation::dummy()),
                                Variable::new("x", SourceInformation::dummy()),
                                SourceInformation::dummy()
                            ),
                            types::Unknown::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into(),
                    ],
                    Variable::new("g", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_()
                    .parse(stream(
                        indoc!(
                            "
                        let
                            f x = g x
                        in
                            f
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Let::new(
                    vec![FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        Application::new(
                            Variable::new("g", SourceInformation::dummy()),
                            Variable::new("x", SourceInformation::dummy()),
                            SourceInformation::dummy()
                        ),
                        types::Unknown::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("f", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
            assert_eq!(
                let_()
                    .parse(stream(
                        indoc!(
                            "
                        let
                            f x = g x
                            h x = i x
                        in
                            f
                        "
                        ),
                        ""
                    ))
                    .unwrap()
                    .0,
                Let::new(
                    vec![
                        FunctionDefinition::new(
                            "f",
                            vec!["x".into()],
                            Application::new(
                                Variable::new("g", SourceInformation::dummy()),
                                Variable::new("x", SourceInformation::dummy()),
                                SourceInformation::dummy()
                            ),
                            types::Unknown::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into(),
                        FunctionDefinition::new(
                            "h",
                            vec!["x".into()],
                            Application::new(
                                Variable::new("i", SourceInformation::dummy()),
                                Variable::new("x", SourceInformation::dummy()),
                                SourceInformation::dummy()
                            ),
                            types::Unknown::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into(),
                    ],
                    Variable::new("f", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
            );
        }

        #[test]
        fn parse_application() {
            assert_eq!(
                expression().parse(stream("f 1", "")).unwrap().0,
                Application::new(
                    Variable::new("f", SourceInformation::dummy()),
                    Number::new(1.0, SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("f x", "")).unwrap().0,
                Application::new(
                    Variable::new("f", SourceInformation::dummy()),
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("f 1 2", "")).unwrap().0,
                Application::new(
                    Application::new(
                        Variable::new("f", SourceInformation::dummy()),
                        Number::new(1.0, SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    Number::new(2.0, SourceInformation::dummy()),
                    SourceInformation::dummy()
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
                Application::new(
                    Variable::new("f", SourceInformation::dummy()),
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
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
                Application::new(
                    Variable::new("f", SourceInformation::dummy()),
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
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
                Application::new(
                    Variable::new("f", SourceInformation::dummy()),
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
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
                Application::new(
                    Variable::new("f", SourceInformation::dummy()),
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into()
            );
        }

        #[test]
        fn parse_application_terminator() {
            for source in &[
                "", "\n", " \n", "\n\n", "+", ")", "\n)", "\n )", "}", "then",
            ] {
                assert!(application_terminator().parse(stream(source, "")).is_ok());
            }
        }

        #[test]
        fn parse_operation() {
            for (source, target) in vec![
                (
                    "1 + 1",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(1.0, SourceInformation::dummy()),
                        Number::new(1.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 + 1 then",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(1.0, SourceInformation::dummy()),
                        Number::new(1.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 + 1 + 1",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        ArithmeticOperation::new(
                            ArithmeticOperator::Add,
                            Number::new(1.0, SourceInformation::dummy()),
                            Number::new(1.0, SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        Number::new(1.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 + (1 + 1)",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(1.0, SourceInformation::dummy()),
                        ArithmeticOperation::new(
                            ArithmeticOperator::Add,
                            Number::new(1.0, SourceInformation::dummy()),
                            Number::new(1.0, SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 * 2 - 3",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Subtract,
                        ArithmeticOperation::new(
                            ArithmeticOperator::Multiply,
                            Number::new(1.0, SourceInformation::dummy()),
                            Number::new(2.0, SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        Number::new(3.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 + 2 * 3",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(1.0, SourceInformation::dummy()),
                        ArithmeticOperation::new(
                            ArithmeticOperator::Multiply,
                            Number::new(2.0, SourceInformation::dummy()),
                            Number::new(3.0, SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 * 2 - 3 / 4",
                    ArithmeticOperation::new(
                        ArithmeticOperator::Subtract,
                        ArithmeticOperation::new(
                            ArithmeticOperator::Multiply,
                            Number::new(1.0, SourceInformation::dummy()),
                            Number::new(2.0, SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        ArithmeticOperation::new(
                            ArithmeticOperator::Divide,
                            Number::new(3.0, SourceInformation::dummy()),
                            Number::new(4.0, SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "1 == 1",
                    EqualityOperation::new(
                        EqualityOperator::Equal,
                        Number::new(1.0, SourceInformation::dummy()),
                        Number::new(1.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "True && True",
                    BooleanOperation::new(
                        BooleanOperator::And,
                        Boolean::new(true, SourceInformation::dummy()),
                        Boolean::new(true, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "True || True",
                    BooleanOperation::new(
                        BooleanOperator::Or,
                        Boolean::new(true, SourceInformation::dummy()),
                        Boolean::new(true, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "True && 1 < 2",
                    BooleanOperation::new(
                        BooleanOperator::And,
                        Boolean::new(true, SourceInformation::dummy()),
                        OrderOperation::new(
                            OrderOperator::LessThan,
                            Number::new(1.0, SourceInformation::dummy()),
                            Number::new(2.0, SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "True || True && True",
                    BooleanOperation::new(
                        BooleanOperator::Or,
                        Boolean::new(true, SourceInformation::dummy()),
                        BooleanOperation::new(
                            BooleanOperator::And,
                            Boolean::new(true, SourceInformation::dummy()),
                            Boolean::new(true, SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "42 |> f",
                    PipeOperation::new(
                        Number::new(42.0, SourceInformation::dummy()),
                        Variable::new("f", SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                (
                    "42 |> f |> g",
                    PipeOperation::new(
                        PipeOperation::new(
                            Number::new(42.0, SourceInformation::dummy()),
                            Variable::new("f", SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        Variable::new("g", SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
            ] {
                assert_eq!(expression().parse(stream(source, "")).unwrap().0, target);
            }
        }

        #[test]
        fn parse_record_construction() {
            assert!(record_construction().parse(stream("Foo", "")).is_err());

            assert!(record_construction().parse(stream("Foo{}", "")).is_err());

            assert_eq!(
                expression()
                    .parse(stream("Foo { foo = 42 }", ""))
                    .unwrap()
                    .0,
                Variable::new("Foo", SourceInformation::dummy()).into()
            );

            assert_eq!(
                record_construction()
                    .parse(stream("Foo{ foo = 42 }", ""))
                    .unwrap()
                    .0,
                RecordConstruction::new(
                    types::Reference::new("Foo", SourceInformation::dummy()),
                    vec![(
                        "foo".into(),
                        Number::new(42.0, SourceInformation::dummy()).into()
                    )]
                    .into_iter()
                    .collect(),
                    SourceInformation::dummy()
                )
            );

            assert_eq!(
                record_construction()
                    .parse(stream("Foo{ foo = 42, bar = 42 }", ""))
                    .unwrap()
                    .0,
                RecordConstruction::new(
                    types::Reference::new("Foo", SourceInformation::dummy()),
                    vec![
                        (
                            "foo".into(),
                            Number::new(42.0, SourceInformation::dummy()).into()
                        ),
                        (
                            "bar".into(),
                            Number::new(42.0, SourceInformation::dummy()).into()
                        )
                    ]
                    .into_iter()
                    .collect(),
                    SourceInformation::dummy()
                )
            );

            assert!(record_construction()
                .parse(stream("Foo{ foo = 42, foo = 42 }", ""))
                .is_err());

            assert_eq!(
                expression()
                    .parse(stream("foo Foo{ foo = 42 }", ""))
                    .unwrap()
                    .0,
                Application::new(
                    Variable::new("foo", SourceInformation::dummy()),
                    RecordConstruction::new(
                        types::Reference::new("Foo", SourceInformation::dummy()),
                        vec![(
                            "foo".into(),
                            Number::new(42.0, SourceInformation::dummy()).into()
                        )]
                        .into_iter()
                        .collect(),
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
                )
                .into()
            );

            assert_eq!(
                record_construction()
                    .parse(stream("Foo{ foo = bar\n42, }", ""))
                    .unwrap()
                    .0,
                RecordConstruction::new(
                    types::Reference::new("Foo", SourceInformation::dummy()),
                    vec![(
                        "foo".into(),
                        Application::new(
                            Variable::new("bar", SourceInformation::dummy()),
                            Number::new(42.0, SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into()
                    )]
                    .into_iter()
                    .collect(),
                    SourceInformation::dummy()
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
                    types::Reference::new("Foo", SourceInformation::dummy()),
                    Variable::new("foo", SourceInformation::dummy()),
                    vec![(
                        "bar".into(),
                        Number::new(42.0, SourceInformation::dummy()).into()
                    )]
                    .into_iter()
                    .collect(),
                    SourceInformation::dummy()
                )
            );

            assert_eq!(
                record_update()
                    .parse(stream("Foo{ ...foo, bar = 42, }", ""))
                    .unwrap()
                    .0,
                RecordUpdate::new(
                    types::Reference::new("Foo", SourceInformation::dummy()),
                    Variable::new("foo", SourceInformation::dummy()),
                    vec![(
                        "bar".into(),
                        Number::new(42.0, SourceInformation::dummy()).into()
                    )]
                    .into_iter()
                    .collect(),
                    SourceInformation::dummy()
                )
            );

            assert_eq!(
                expression()
                    .parse(stream("Foo { ...foo, bar = 42 }", ""))
                    .unwrap()
                    .0,
                Variable::new("Foo", SourceInformation::dummy()).into(),
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
            assert!(operator().parse(stream("", "")).is_err());
            assert!(operator().parse(stream("++", "")).is_err());

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
                assert_eq!(operator().parse(stream(source, "")).unwrap().0, *expected);
            }
        }

        #[test]
        fn parse_variable() {
            assert!(variable().parse(stream("Foo. x", "")).is_err());
            assert_eq!(
                variable().parse(stream("x", "")).unwrap().0,
                Variable::new("x", SourceInformation::dummy()),
            );
            assert_eq!(
                variable().parse(stream("Foo.x", "")).unwrap().0,
                Variable::new("Foo.x", SourceInformation::dummy()),
            );
            assert_eq!(
                variable().parse(stream("Foo .x", "")).unwrap().0,
                Variable::new("Foo", SourceInformation::dummy()),
            );
        }

        #[test]
        fn parse_boolean_literal() {
            assert!(boolean_literal().parse(stream("", "")).is_err());
            assert_eq!(
                boolean_literal().parse(stream("False", "")).unwrap().0,
                Boolean::new(false, SourceInformation::dummy())
            );
            assert_eq!(
                boolean_literal().parse(stream("True", "")).unwrap().0,
                Boolean::new(true, SourceInformation::dummy())
            );
        }

        #[test]
        fn parse_none_literal() {
            assert!(none_literal().parse(stream("", "")).is_err());
            assert_eq!(
                none_literal().parse(stream("None", "")).unwrap().0,
                None::new(SourceInformation::dummy())
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
                    Number::new(*value, SourceInformation::dummy())
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
                    ByteString::new(*value, SourceInformation::dummy())
                );
            }
        }

        #[test]
        fn parse_list() {
            for (source, target) in vec![
                ("[]", List::new(vec![], SourceInformation::dummy())),
                (
                    "[42]",
                    List::new(
                        vec![ListElement::Single(
                            Number::new(42.0, SourceInformation::dummy()).into(),
                        )],
                        SourceInformation::dummy(),
                    ),
                ),
                (
                    "[42,]",
                    List::new(
                        vec![ListElement::Single(
                            Number::new(42.0, SourceInformation::dummy()).into(),
                        )],
                        SourceInformation::dummy(),
                    ),
                ),
                (
                    "[42,42]",
                    List::new(
                        vec![
                            ListElement::Single(
                                Number::new(42.0, SourceInformation::dummy()).into(),
                            ),
                            ListElement::Single(
                                Number::new(42.0, SourceInformation::dummy()).into(),
                            ),
                        ],
                        SourceInformation::dummy(),
                    ),
                ),
                (
                    "[42,42,]",
                    List::new(
                        vec![
                            ListElement::Single(
                                Number::new(42.0, SourceInformation::dummy()).into(),
                            ),
                            ListElement::Single(
                                Number::new(42.0, SourceInformation::dummy()).into(),
                            ),
                        ],
                        SourceInformation::dummy(),
                    ),
                ),
                (
                    "[...foo]",
                    List::new(
                        vec![ListElement::Multiple(
                            Variable::new("foo", SourceInformation::dummy()).into(),
                        )],
                        SourceInformation::dummy(),
                    ),
                ),
                (
                    "[...foo,]",
                    List::new(
                        vec![ListElement::Multiple(
                            Variable::new("foo", SourceInformation::dummy()).into(),
                        )],
                        SourceInformation::dummy(),
                    ),
                ),
                (
                    "[...foo,...bar]",
                    List::new(
                        vec![
                            ListElement::Multiple(
                                Variable::new("foo", SourceInformation::dummy()).into(),
                            ),
                            ListElement::Multiple(
                                Variable::new("bar", SourceInformation::dummy()).into(),
                            ),
                        ],
                        SourceInformation::dummy(),
                    ),
                ),
                (
                    "[...foo,...bar,]",
                    List::new(
                        vec![
                            ListElement::Multiple(
                                Variable::new("foo", SourceInformation::dummy()).into(),
                            ),
                            ListElement::Multiple(
                                Variable::new("bar", SourceInformation::dummy()).into(),
                            ),
                        ],
                        SourceInformation::dummy(),
                    ),
                ),
                (
                    "[foo,...bar]",
                    List::new(
                        vec![
                            ListElement::Single(
                                Variable::new("foo", SourceInformation::dummy()).into(),
                            ),
                            ListElement::Multiple(
                                Variable::new("bar", SourceInformation::dummy()).into(),
                            ),
                        ],
                        SourceInformation::dummy(),
                    ),
                ),
                (
                    "[...foo,bar]",
                    List::new(
                        vec![
                            ListElement::Multiple(
                                Variable::new("foo", SourceInformation::dummy()).into(),
                            ),
                            ListElement::Single(
                                Variable::new("bar", SourceInformation::dummy()).into(),
                            ),
                        ],
                        SourceInformation::dummy(),
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
    fn parse_source_information() {
        assert!(source_information()
            .with(combine::eof())
            .parse(stream(" \n \n \n", ""))
            .is_ok());
    }

    #[test]
    fn parse_blank() {
        for source in &[
            "",
            " ",
            "  ",
            "\n",
            "\n\n",
            " \n",
            "\n ",
            " \n \n \n",
            "\n \n \n ",
        ] {
            assert!(blank().parse(stream(source, "")).is_ok());
        }
    }

    #[test]
    fn parse_spaces1() {
        assert!(spaces1()
            .with(combine::eof())
            .parse(stream("", ""))
            .is_err());

        for source in &[" ", "  ", "\t", "\r"] {
            assert!(spaces1()
                .with(combine::eof())
                .parse(stream(source, ""))
                .is_ok());
        }
    }

    #[test]
    fn parse_newlines1() {
        for source in &["", "\n", " \n", "\n\n", "#\n", " #\n"] {
            assert!(newlines1()
                .with(combine::eof())
                .parse(stream(source, ""))
                .is_ok());
        }
    }

    #[test]
    fn parse_comment() {
        assert!(comment().parse(stream("#\n", "")).is_ok());
        assert!(comment().parse(stream("#x\n", "")).is_ok());
    }
}
