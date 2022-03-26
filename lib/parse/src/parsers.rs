use super::operations::*;
use crate::{comment::Comment, stream::Stream};
use ast::{
    types::{self, Type},
    *,
};
use combine::{
    attempt, choice, look_ahead, many, many1, none_of, one_of, optional,
    parser::{
        char::{alpha_num, char as character, digit, letter, space, string},
        combinator::{lazy, no_partial, not_followed_by},
        regex::find,
        sequence::between,
    },
    sep_end_by, sep_end_by1, unexpected_any, value, Parser, Positioned,
};
use fnv::FnvHashSet;
use once_cell::sync::Lazy;
use position::Position;

const BUILT_IN_LITERALS: &[&str] = &["false", "none", "true"];
const BUILT_IN_TYPES: &[&str] = &["any", "boolean", "none", "number", "string"];
static KEYWORDS: Lazy<Vec<&str>> = Lazy::new(|| {
    [
        "as", "else", "export", "for", "foreign", "go", "if", "in", "import", "type",
    ]
    .iter()
    .chain(BUILT_IN_LITERALS)
    .chain(BUILT_IN_TYPES)
    .copied()
    .collect()
});
const OPERATOR_CHARACTERS: &str = "+-*/=<>&|!?";

static BINARY_REGEX: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"^0b[01]+").unwrap());
static HEXADECIMAL_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^0x[0-9a-fA-F]+").unwrap());
static DECIMAL_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^-?([1-9][0-9]*|0)(\.[0-9]+)?").unwrap());
static STRING_CHARACTER_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r#"^[^\\"]"#).unwrap());
static BYTE_CHARACTER_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^[0-9a-fA-F]{2}").unwrap());

pub fn module<'a>() -> impl Parser<Stream<'a>, Output = Module> {
    (
        position(),
        blank(),
        many(import()),
        many(foreign_import()),
        many(choice((
            type_alias().map(TypeDefinition::from),
            record_definition().map(TypeDefinition::from),
        ))),
        many(definition()),
    )
        .skip(eof())
        .map(
            |(position, _, imports, foreign_imports, type_definitions, definitions)| {
                Module::new(
                    imports,
                    foreign_imports,
                    type_definitions,
                    definitions,
                    position,
                )
            },
        )
}

fn import<'a>() -> impl Parser<Stream<'a>, Output = Import> {
    (
        attempt(keyword("import").with(not_followed_by(keyword("foreign").with(value("foreign"))))),
        module_path(),
        optional(keyword("as").with(identifier())),
        optional(between(
            sign("{"),
            sign("}"),
            sep_end_by1(identifier(), sign(",")),
        )),
    )
        .map(|(_, path, prefix, names)| Import::new(path, prefix, names.unwrap_or_default()))
        .expected("import statement")
}

fn module_path<'a>() -> impl Parser<Stream<'a>, Output = ModulePath> {
    token(choice((
        internal_module_path().map(ModulePath::from),
        external_module_path().map(ModulePath::from),
    )))
    .expected("module path")
}

fn internal_module_path<'a>() -> impl Parser<Stream<'a>, Output = InternalModulePath> {
    module_path_components(identifier()).map(InternalModulePath::new)
}

fn external_module_path<'a>() -> impl Parser<Stream<'a>, Output = ExternalModulePath> {
    (
        identifier(),
        module_path_components(public_module_path_component()),
    )
        .map(|(package, components)| ExternalModulePath::new(package, components))
}

fn module_path_components<'a>(
    component: impl Parser<Stream<'a>, Output = String>,
) -> impl Parser<Stream<'a>, Output = Vec<String>> {
    many1(string(IDENTIFIER_SEPARATOR).with(component))
}

fn public_module_path_component<'a>() -> impl Parser<Stream<'a>, Output = String> {
    look_ahead(identifier().expected("public module path"))
        .then(|name| {
            if ast::analysis::is_name_public(&name) {
                value(()).left()
            } else {
                unexpected_any("private module path").right()
            }
        })
        .with(identifier())
}

fn foreign_import<'a>() -> impl Parser<Stream<'a>, Output = ForeignImport> {
    (
        attempt(position().skip((keyword("import"), keyword("foreign")))),
        optional(calling_convention()),
        identifier(),
        type_(),
    )
        .map(|(position, calling_convention, name, type_)| {
            ForeignImport::new(
                &name,
                calling_convention.unwrap_or_default(),
                type_,
                position,
            )
        })
        .expected("foreign import statement")
}

fn calling_convention<'a>() -> impl Parser<Stream<'a>, Output = CallingConvention> {
    string_literal()
        .expected("calling convention")
        .then(|string| {
            if string.value() == "c" {
                value(CallingConvention::C).left()
            } else {
                unexpected_any("unknown calling convention").right()
            }
        })
}

fn definition<'a>() -> impl Parser<Stream<'a>, Output = Definition> {
    (
        optional(foreign_export()),
        position(),
        identifier(),
        sign("="),
        lambda(),
    )
        .map(|(foreign_export, position, name, _, lambda)| {
            Definition::new(name, lambda, foreign_export, position)
        })
        .expected("definition")
}

fn foreign_export<'a>() -> impl Parser<Stream<'a>, Output = ForeignExport> {
    keyword("foreign")
        .with(optional(calling_convention()))
        .map(|calling_convention| ForeignExport::new(calling_convention.unwrap_or_default()))
}

fn record_definition<'a>() -> impl Parser<Stream<'a>, Output = RecordDefinition> {
    (
        attempt((position(), keyword("type"))),
        identifier(),
        sign("{"),
        many((identifier(), type_())),
        sign("}"),
    )
        .map(
            |((position, _), name, _, fields, _): (_, _, _, Vec<_>, _)| {
                RecordDefinition::new(
                    name,
                    fields
                        .into_iter()
                        .map(|(name, type_)| types::RecordField::new(name, type_))
                        .collect(),
                    position,
                )
            },
        )
        .expected("record definition")
}

fn type_alias<'a>() -> impl Parser<Stream<'a>, Output = TypeAlias> {
    (
        attempt((position(), keyword("type"), identifier(), sign("="))),
        type_(),
    )
        .map(|((position, _, name, _), type_)| TypeAlias::new(name, type_, position))
        .expected("type alias")
}

fn type_<'a>() -> impl Parser<Stream<'a>, Output = Type> {
    lazy(|| no_partial(choice((function_type().map(Type::from), union_type()))))
        .boxed()
        .expected("type")
}

fn function_type<'a>() -> impl Parser<Stream<'a>, Output = types::Function> {
    (
        attempt(position().skip(sign("\\("))),
        sep_end_by(type_(), sign(",")),
        sign(")"),
        type_(),
    )
        .map(|(position, arguments, _, result)| types::Function::new(arguments, result, position))
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
    (attempt(position().skip(sign("["))), type_(), sign("]"))
        .map(|(position, element, _)| types::List::new(element, position))
        .expected("list type")
}

fn map_type<'a>() -> impl Parser<Stream<'a>, Output = types::Map> {
    (
        attempt(position().skip(sign("{"))),
        type_(),
        sign(":"),
        type_(),
        sign("}"),
    )
        .map(|(position, key, _, value, _)| types::Map::new(key, value, position))
        .expected("map type")
}

fn atomic_type<'a>() -> impl Parser<Stream<'a>, Output = Type> {
    choice((
        boolean_type().map(Type::from),
        none_type().map(Type::from),
        number_type().map(Type::from),
        string_type().map(Type::from),
        any_type().map(Type::from),
        reference_type().map(Type::from),
        list_type().map(Type::from),
        map_type().map(Type::from),
        between(sign("("), sign(")"), type_()),
    ))
}

fn boolean_type<'a>() -> impl Parser<Stream<'a>, Output = types::Boolean> {
    attempt(position().skip(keyword("boolean")))
        .map(types::Boolean::new)
        .expected("boolean type")
}

fn none_type<'a>() -> impl Parser<Stream<'a>, Output = types::None> {
    attempt(position().skip(keyword("none")))
        .map(types::None::new)
        .expected("none type")
}

fn number_type<'a>() -> impl Parser<Stream<'a>, Output = types::Number> {
    attempt(position().skip(keyword("number")))
        .map(types::Number::new)
        .expected("number type")
}

fn string_type<'a>() -> impl Parser<Stream<'a>, Output = types::ByteString> {
    attempt(position().skip(keyword("string")))
        .map(types::ByteString::new)
        .expected("string type")
}

fn any_type<'a>() -> impl Parser<Stream<'a>, Output = types::Any> {
    attempt(position().skip(keyword("any")))
        .map(types::Any::new)
        .expected("any type")
}

fn reference_type<'a>() -> impl Parser<Stream<'a>, Output = types::Reference> {
    token(attempt((position(), qualified_identifier())))
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
    choice((statement_with_result(), statement_without_result())).expected("statement")
}

fn statement_with_result<'a>() -> impl Parser<Stream<'a>, Output = Statement> {
    (attempt((position(), identifier(), sign("="))), expression())
        .map(|((position, name, _), expression)| Statement::new(Some(name), expression, position))
}

fn statement_without_result<'a>() -> impl Parser<Stream<'a>, Output = Statement> {
    // Get positions from parsed expressions to avoid use of the attempt combinator.
    expression()
        .map(|expression| Statement::new(None, expression.clone(), expression.position().clone()))
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
            (
                attempt((position(), binary_operator())),
                prefix_operation_like(),
            )
                .map(|((position, operator), expression)| (operator, expression, position)),
        ),
    )
        .map(|(expression, pairs): (_, Vec<_>)| reduce_operations(expression, &pairs))
}

fn binary_operator<'a>() -> impl Parser<Stream<'a>, Output = BinaryOperator> {
    choice((
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
    ))
    .expected("binary operator")
}

fn concrete_binary_operator<'a>(
    literal: &'static str,
    operator: BinaryOperator,
) -> impl Parser<Stream<'a>, Output = BinaryOperator> {
    attempt(token(many1(one_of(OPERATOR_CHARACTERS.chars())).then(
        move |parsed_literal: String| {
            if parsed_literal == literal {
                value(operator).left()
            } else {
                unexpected_any("unknown binary operator").right()
            }
        },
    )))
}

fn prefix_operation_like<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    lazy(|| {
        no_partial(choice((
            prefix_operation().map(Expression::from),
            suffix_operation_like().map(Expression::from),
        )))
    })
    .boxed()
}

fn prefix_operation<'a>() -> impl Parser<Stream<'a>, Output = UnaryOperation> {
    (
        attempt((position(), prefix_operator())),
        prefix_operation_like(),
    )
        .map(|((position, operator), expression)| {
            UnaryOperation::new(operator, expression, position)
        })
}

fn prefix_operator<'a>() -> impl Parser<Stream<'a>, Output = UnaryOperator> {
    choice((concrete_prefix_operator("!", UnaryOperator::Not),)).expected("unary operator")
}

fn spawn_operation<'a>() -> impl Parser<Stream<'a>, Output = SpawnOperation> {
    (attempt(position().skip(keyword("go"))), lambda())
        .map(|(position, lambda)| SpawnOperation::new(lambda, position))
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
    (atomic_expression(), many(suffix_operator())).map(
        |(expression, suffix_operators): (_, Vec<_>)| {
            suffix_operators
                .into_iter()
                .fold(expression, |expression, operator| match operator {
                    SuffixOperator::Call(arguments, position) => {
                        Call::new(expression, arguments, position).into()
                    }
                    SuffixOperator::RecordField(name, position) => {
                        RecordDeconstruction::new(expression, name, position).into()
                    }
                    SuffixOperator::Try(position) => {
                        UnaryOperation::new(UnaryOperator::Try, expression, position).into()
                    }
                })
        },
    )
}

fn suffix_operator<'a>() -> impl Parser<Stream<'a>, Output = SuffixOperator> {
    choice((call_operator(), record_field_operator(), try_operator()))
}

fn call_operator<'a>() -> impl Parser<Stream<'a>, Output = SuffixOperator> {
    (
        attempt(position().skip(sign("("))),
        sep_end_by(expression(), sign(",")).skip(sign(")")),
    )
        .map(|(position, arguments)| SuffixOperator::Call(arguments, position))
}

fn record_field_operator<'a>() -> impl Parser<Stream<'a>, Output = SuffixOperator> {
    (attempt(position().skip(sign("."))), identifier())
        .map(|(position, identifier)| SuffixOperator::RecordField(identifier, position))
}

fn try_operator<'a>() -> impl Parser<Stream<'a>, Output = SuffixOperator> {
    position().skip(sign("?")).map(SuffixOperator::Try)
}

fn atomic_expression<'a>() -> impl Parser<Stream<'a>, Output = Expression> {
    lazy(|| {
        no_partial(choice((
            spawn_operation().map(Expression::from),
            if_list().map(Expression::from),
            if_map().map(Expression::from),
            if_type().map(Expression::from),
            if_().map(Expression::from),
            lambda().map(Expression::from),
            record().map(Expression::from),
            list_comprehension().map(Expression::from),
            list_literal().map(Expression::from),
            map_literal().map(Expression::from),
            boolean_literal().map(Expression::from),
            none_literal().map(Expression::from),
            number_literal().map(Expression::from),
            string_literal().map(Expression::from),
            variable().map(Expression::from),
            between(sign("("), sign(")"), expression()),
        )))
    })
    .boxed()
}

fn lambda<'a>() -> impl Parser<Stream<'a>, Output = Lambda> {
    (
        attempt(position().skip(sign("\\("))),
        sep_end_by(argument(), sign(",")),
        sign(")"),
        type_(),
        block(),
    )
        .map(|(position, arguments, _, result_type, body)| {
            Lambda::new(arguments, result_type, body, position)
        })
        .expected("function expression")
}

fn argument<'a>() -> impl Parser<Stream<'a>, Output = Argument> {
    (identifier(), type_()).map(|(name, type_)| Argument::new(name, type_))
}

fn if_<'a>() -> impl Parser<Stream<'a>, Output = If> {
    (
        attempt(position().skip(keyword("if"))),
        if_branch(),
        many(attempt((keyword("else"), keyword("if"))).with(if_branch())),
        keyword("else"),
        block(),
    )
        .map(
            |(position, first_branch, branches, _, else_block): (_, _, Vec<_>, _, _)| {
                If::new(
                    [first_branch].into_iter().chain(branches).collect(),
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
        attempt(position().skip((keyword("if"), sign("[")))),
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
            |(position, first_name, _, _, rest_name, _, _, argument, then, _, else_)| {
                IfList::new(argument, first_name, rest_name, then, else_, position)
            },
        )
        .expected("if-list expression")
}

fn if_map<'a>() -> impl Parser<Stream<'a>, Output = IfMap> {
    (
        attempt((
            position(),
            keyword("if"),
            identifier(),
            sign("="),
            expression(),
            sign("["),
        )),
        expression(),
        sign("]"),
        block(),
        keyword("else"),
        block(),
    )
        .map(|((position, _, name, _, map, _), key, _, then, _, else_)| {
            IfMap::new(name, map, key, then, else_, position)
        })
        .expected("if-map expression")
}

fn if_type<'a>() -> impl Parser<Stream<'a>, Output = IfType> {
    (
        attempt((position(), keyword("if"), identifier(), sign("="))),
        expression(),
        keyword("as"),
        if_type_branch(),
        many(attempt((keyword("else"), keyword("if"))).with(if_type_branch())),
        optional(keyword("else").with(block())),
    )
        .map(
            |((position, _, identifier, _), argument, _, first_branch, branches, else_): (
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
                    [first_branch].into_iter().chain(branches).collect(),
                    else_,
                    position,
                )
            },
        )
        .expected("if-type expression")
}

fn if_type_branch<'a>() -> impl Parser<Stream<'a>, Output = IfTypeBranch> {
    (type_(), block()).map(|(type_, block)| IfTypeBranch::new(type_, block))
}

fn record<'a>() -> impl Parser<Stream<'a>, Output = Record> {
    (
        attempt((position(), qualified_identifier(), sign("{"))),
        choice((
            (
                between(sign("..."), sign(","), expression()).map(Some),
                sep_end_by1(record_field(), sign(",")),
            ),
            (value(None), sep_end_by(record_field(), sign(","))),
        )),
        sign("}"),
    )
        .then(|((position, name, _), (record, fields), _)| {
            let fields: Vec<_> = fields;

            if fields
                .iter()
                .map(|field| field.name())
                .collect::<FnvHashSet<_>>()
                .len()
                == fields.len()
            {
                value(Record::new(name, record, fields, position)).left()
            } else {
                unexpected_any("duplicate keys in record literal").right()
            }
        })
        .expected("record literal")
}

fn record_field<'a>() -> impl Parser<Stream<'a>, Output = RecordField> {
    (attempt((position(), identifier())), sign(":"), expression())
        .map(|((position, name), _, expression)| RecordField::new(name, expression, position))
}

fn boolean_literal<'a>() -> impl Parser<Stream<'a>, Output = Boolean> {
    token(choice((
        attempt(position().skip(keyword("false"))).map(|position| Boolean::new(false, position)),
        attempt(position().skip(keyword("true"))).map(|position| Boolean::new(true, position)),
    )))
    .expected("boolean literal")
}

fn none_literal<'a>() -> impl Parser<Stream<'a>, Output = None> {
    token(attempt(position().skip(keyword("none"))))
        .map(None::new)
        .expected("none literal")
}

fn number_literal<'a>() -> impl Parser<Stream<'a>, Output = Number> {
    token(
        attempt((
            position(),
            choice((binary_literal(), hexadecimal_literal(), decimal_literal())),
        ))
        .skip(not_followed_by(digit())),
    )
    .map(|(position, number)| Number::new(number, position))
    .silent()
    .expected("number literal")
}

fn binary_literal<'a>() -> impl Parser<Stream<'a>, Output = NumberRepresentation> {
    let regex: &'static regex::Regex = &BINARY_REGEX;

    find(regex).map(|string: &str| NumberRepresentation::Binary(string[2..].into()))
}

fn hexadecimal_literal<'a>() -> impl Parser<Stream<'a>, Output = NumberRepresentation> {
    let regex: &'static regex::Regex = &HEXADECIMAL_REGEX;

    find(regex).map(|string: &str| NumberRepresentation::Hexadecimal(string[2..].to_lowercase()))
}

fn decimal_literal<'a>() -> impl Parser<Stream<'a>, Output = NumberRepresentation> {
    let regex: &'static regex::Regex = &DECIMAL_REGEX;

    find(regex).map(|string: &str| NumberRepresentation::FloatingPoint(string.into()))
}

fn string_literal<'a>() -> impl Parser<Stream<'a>, Output = ByteString> {
    let string_regex: &'static regex::Regex = &STRING_CHARACTER_REGEX;
    let byte_regex: &'static regex::Regex = &BYTE_CHARACTER_REGEX;

    token((
        attempt(position().skip(character('"'))),
        many(choice((
            find(string_regex).map(String::from),
            special_string_character("\\\\"),
            special_string_character("\\\""),
            special_string_character("\\n"),
            special_string_character("\\r"),
            special_string_character("\\t"),
            (attempt(string("\\x")), find(byte_regex))
                .map(|(prefix, byte)| prefix.to_owned() + byte),
        ))),
        character('"'),
    ))
    .map(|(position, strings, _): (_, Vec<String>, _)| ByteString::new(strings.concat(), position))
    .expected("string literal")
}

fn special_string_character<'a>(escape: &'static str) -> impl Parser<Stream<'a>, Output = String> {
    attempt(string(escape)).map(String::from)
}

fn list_literal<'a>() -> impl Parser<Stream<'a>, Output = List> {
    (
        attempt(position().skip(sign("["))),
        type_(),
        sep_end_by(list_element(), sign(",")),
        sign("]"),
    )
        .map(|(position, type_, elements, _)| List::new(type_, elements, position))
        .expected("list literal")
}

fn list_element<'a>() -> impl Parser<Stream<'a>, Output = ListElement> {
    choice((
        expression().map(ListElement::Single),
        sign("...").with(expression()).map(ListElement::Multiple),
    ))
}

fn list_comprehension<'a>() -> impl Parser<Stream<'a>, Output = ListComprehension> {
    (
        attempt((position(), sign("["), type_(), expression(), keyword("for"))),
        identifier(),
        keyword("in"),
        expression(),
        sign("]"),
    )
        .map(
            |((position, _, type_, element, _), element_name, _, list, _)| {
                ListComprehension::new(type_, element, element_name, list, position)
            },
        )
        .expected("list literal")
}

fn map_literal<'a>() -> impl Parser<Stream<'a>, Output = Map> {
    (
        attempt(position().skip(sign("{"))),
        type_(),
        sign(":"),
        type_(),
        sep_end_by(map_element(), sign(",")),
        sign("}"),
    )
        .map(|(position, key_type, _, value_type, elements, _)| {
            Map::new(key_type, value_type, elements, position)
        })
        .expected("map literal")
}

fn map_element<'a>() -> impl Parser<Stream<'a>, Output = MapElement> {
    choice((
        (
            attempt((position(), expression()).skip(sign(":"))),
            expression(),
        )
            .map(|((position, key), value)| MapEntry::new(key, value, position).into()),
        sign("...").with(expression()).map(MapElement::Map),
        expression().map(MapElement::Removal),
    ))
}

fn variable<'a>() -> impl Parser<Stream<'a>, Output = Variable> {
    token(attempt((position(), qualified_identifier())))
        .map(|(position, identifier)| Variable::new(identifier, position))
        .expected("variable")
}

fn qualified_identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    (
        raw_identifier(),
        optional(string(IDENTIFIER_SEPARATOR).with(raw_identifier())),
    )
        .map(|(former, latter)| {
            latter
                .map(|latter| [&former, IDENTIFIER_SEPARATOR, &latter].concat())
                .unwrap_or(former)
        })
}

fn identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    token(raw_identifier()).expected("identifier")
}

fn raw_identifier<'a>() -> impl Parser<Stream<'a>, Output = String> {
    (
        choice((letter(), combine::parser::char::char('_'))),
        many(choice((alpha_num(), combine::parser::char::char('_')))),
    )
        .map(|(head, tail): (char, String)| [head.into(), tail].concat())
        .then(|identifier| {
            if KEYWORDS.contains(&identifier.as_str()) {
                // TODO Fix those misuse of `unexpected_any` combinators.
                // These lead to wrong positions in error messages.
                unexpected_any("keyword").left()
            } else {
                value(identifier).right()
            }
        })
        .silent()
}

fn keyword<'a>(name: &'static str) -> impl Parser<Stream<'a>, Output = ()> {
    if !KEYWORDS.contains(&name) {
        unreachable!("undefined keyword");
    }

    token(attempt(string(name)).skip(not_followed_by(choice((
        alpha_num(),
        combine::parser::char::char('_'),
    )))))
    .with(value(()))
    .expected(name)
}

fn sign<'a>(sign: &'static str) -> impl Parser<Stream<'a>, Output = ()> {
    let parser = string(sign);

    token(
        if sign
            .chars()
            .any(|character| OPERATOR_CHARACTERS.contains(character))
        {
            parser
                .skip(not_followed_by(one_of(OPERATOR_CHARACTERS.chars())))
                .left()
        } else {
            parser.right()
        },
    )
    .with(value(()))
    .expected(sign)
}

fn token<'a, O, P: Parser<Stream<'a>, Output = O>>(p: P) -> impl Parser<Stream<'a>, Output = O> {
    p.skip(blank())
}

fn position<'a>() -> impl Parser<Stream<'a>, Output = Position> {
    value(()).map_input(|_, stream: &mut Stream<'a>| {
        let position = stream.position();

        Position::new(
            &stream.0.state.path,
            position.line as usize,
            position.column as usize,
            stream.0.state.lines[position.line as usize - 1],
        )
    })
}

fn eof<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    combine::eof().expected("end of file")
}

fn blank<'a>() -> impl Parser<Stream<'a>, Output = ()> {
    many::<Vec<_>, _, _>(choice((space().with(value(())), comment().with(value(())))))
        .with(value(()))
}

fn comment<'a>() -> impl Parser<Stream<'a>, Output = Comment> {
    (
        attempt((position(), string("#"))),
        many::<Vec<_>, _, _>(none_of("\n".chars())),
    )
        .map(|((position, _), string)| {
            Comment::new(string.into_iter().collect::<String>().trim_end(), position)
        })
        .skip(choice((
            combine::parser::char::newline().with(value(())),
            eof(),
        )))
        .expected("comment")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::ParseError, stream::stream};
    use position::test::PositionFake;
    use pretty_assertions::assert_eq;

    mod module {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_module() {
            assert_eq!(
                module().parse(stream("", "")).unwrap().0,
                Module::new(vec![], vec![], vec![], vec![], Position::fake())
            );
            assert_eq!(
                module().parse(stream(" ", "")).unwrap().0,
                Module::new(vec![], vec![], vec![], vec![], Position::fake())
            );
            assert_eq!(
                module().parse(stream("\n", "")).unwrap().0,
                Module::new(vec![], vec![], vec![], vec![], Position::fake())
            );
            assert_eq!(
                module().parse(stream("import Foo'Bar", "")).unwrap().0,
                Module::new(
                    vec![Import::new(
                        ExternalModulePath::new("Foo", vec!["Bar".into()]),
                        None,
                        vec![]
                    )],
                    vec![],
                    vec![],
                    vec![],
                    Position::fake()
                )
            );
            assert_eq!(
                module().parse(stream("type foo = number", "")).unwrap().0,
                Module::new(
                    vec![],
                    vec![],
                    vec![TypeAlias::new(
                        "foo",
                        types::Number::new(Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    vec![],
                    Position::fake()
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
                    vec![Definition::new(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", types::Number::new(Position::fake()))],
                            types::Number::new(Position::fake()),
                            Block::new(
                                vec![],
                                Number::new(
                                    NumberRepresentation::FloatingPoint("42".into()),
                                    Position::fake()
                                ),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        None,
                        Position::fake()
                    )],
                    Position::fake()
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
                    vec![
                        Definition::new(
                            "x",
                            Lambda::new(
                                vec![Argument::new("x", types::Number::new(Position::fake()))],
                                types::Number::new(Position::fake()),
                                Block::new(
                                    vec![],
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("42".into()),
                                        Position::fake()
                                    ),
                                    Position::fake()
                                ),
                                Position::fake()
                            ),
                            None,
                            Position::fake()
                        ),
                        Definition::new(
                            "y",
                            Lambda::new(
                                vec![Argument::new("y", types::Number::new(Position::fake()))],
                                types::Number::new(Position::fake()),
                                Block::new(
                                    vec![],
                                    Number::new(
                                        NumberRepresentation::FloatingPoint("42".into()),
                                        Position::fake()
                                    ),
                                    Position::fake()
                                ),
                                Position::fake()
                            ),
                            None,
                            Position::fake()
                        )
                    ],
                    Position::fake()
                )
            );
        }

        #[test]
        fn parse_import_foreign_after_import() {
            assert_eq!(
                module()
                    .parse(stream("import Foo'Bar import foreign foo \\() number", ""))
                    .unwrap()
                    .0,
                Module::new(
                    vec![Import::new(
                        ExternalModulePath::new("Foo", vec!["Bar".into()]),
                        None,
                        vec![]
                    )],
                    vec![ForeignImport::new(
                        "foo",
                        CallingConvention::Native,
                        types::Function::new(
                            vec![],
                            types::Number::new(Position::fake()),
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    vec![],
                    vec![],
                    Position::fake()
                )
            );
        }

        #[test]
        fn parse_record_definition_after_type_alias() {
            assert_eq!(
                module()
                    .parse(stream("type foo = number type bar {}", ""))
                    .unwrap()
                    .0,
                Module::new(
                    vec![],
                    vec![],
                    vec![
                        TypeAlias::new(
                            "foo",
                            types::Number::new(Position::fake()),
                            Position::fake()
                        )
                        .into(),
                        RecordDefinition::new("bar", vec![], Position::fake()).into(),
                    ],
                    vec![],
                    Position::fake()
                )
            );
        }
    }

    mod import {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_import() {
            assert_eq!(
                import().parse(stream("import 'Foo", "")).unwrap().0,
                Import::new(InternalModulePath::new(vec!["Foo".into()]), None, vec![]),
            );
            assert_eq!(
                import().parse(stream("import Foo'Bar", "")).unwrap().0,
                Import::new(
                    ExternalModulePath::new("Foo", vec!["Bar".into()]),
                    None,
                    vec![]
                ),
            );
        }

        #[test]
        fn parse_import_with_custom_prefix() {
            assert_eq!(
                import().parse(stream("import 'Foo as foo", "")).unwrap().0,
                Import::new(
                    InternalModulePath::new(vec!["Foo".into()]),
                    Some("foo".into()),
                    vec![]
                ),
            );
        }

        #[test]
        fn parse_unqualified_import() {
            assert_eq!(
                import().parse(stream("import 'Foo { Foo }", "")).unwrap().0,
                Import::new(
                    InternalModulePath::new(vec!["Foo".into()]),
                    None,
                    vec!["Foo".into()]
                ),
            );
        }

        #[test]
        fn parse_unqualified_import_with_multiple_identifiers() {
            assert_eq!(
                import()
                    .parse(stream("import 'Foo { Foo, Bar }", ""))
                    .unwrap()
                    .0,
                Import::new(
                    InternalModulePath::new(vec!["Foo".into()]),
                    None,
                    vec!["Foo".into(), "Bar".into()]
                ),
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
        fn fail_to_parse_private_external_module_file() {
            let source = "Foo'bar";

            insta::assert_debug_snapshot!(external_module_path()
                .parse(stream(source, ""))
                .map_err(|error| ParseError::new(source, "", error))
                .err());
        }

        #[test]
        fn fail_to_parse_private_external_module_directory() {
            let source = "Foo'bar'Baz";

            insta::assert_debug_snapshot!(external_module_path()
                .parse(stream(source, ""))
                .map_err(|error| ParseError::new(source, "", error))
                .err());
        }
    }

    #[test]
    fn parse_foreign_import() {
        assert_eq!(
            foreign_import()
                .parse(stream("import foreign foo \\(number) number", ""))
                .unwrap()
                .0,
            ForeignImport::new(
                "foo",
                CallingConvention::Native,
                types::Function::new(
                    vec![types::Number::new(Position::fake()).into()],
                    types::Number::new(Position::fake()),
                    Position::fake()
                ),
                Position::fake()
            ),
        );

        assert_eq!(
            foreign_import()
                .parse(stream("import foreign \"c\" foo \\(number) number", ""))
                .unwrap()
                .0,
            ForeignImport::new(
                "foo",
                CallingConvention::C,
                types::Function::new(
                    vec![types::Number::new(Position::fake()).into()],
                    types::Number::new(Position::fake()),
                    Position::fake()
                ),
                Position::fake()
            ),
        );
    }

    mod definition {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse() {
            assert_eq!(
                definition()
                    .parse(stream("x=\\(x number)number{42}", ""))
                    .unwrap()
                    .0,
                Definition::new(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", types::Number::new(Position::fake()))],
                        types::Number::new(Position::fake()),
                        Block::new(
                            vec![],
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    None,
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_foreign_definition() {
            assert_eq!(
                definition()
                    .parse(stream("foreign x=\\(x number)number{42}", ""))
                    .unwrap()
                    .0,
                Definition::new(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", types::Number::new(Position::fake()))],
                        types::Number::new(Position::fake()),
                        Block::new(
                            vec![],
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    ForeignExport::new(CallingConvention::Native).into(),
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_foreign_definition_with_c_calling_convention() {
            assert_eq!(
                definition()
                    .parse(stream("foreign \"c\" x=\\(x number)number{42}", ""))
                    .unwrap()
                    .0,
                Definition::new(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", types::Number::new(Position::fake()))],
                        types::Number::new(Position::fake()),
                        Block::new(
                            vec![],
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    ForeignExport::new(CallingConvention::C).into(),
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_keyword_like_name() {
            assert_eq!(
                definition()
                    .parse(stream("importA = \\() number { 42 }", ""))
                    .unwrap()
                    .0,
                Definition::new(
                    "importA",
                    Lambda::new(
                        vec![],
                        types::Number::new(Position::fake()),
                        Block::new(
                            vec![],
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    None,
                    Position::fake()
                ),
            );
        }
    }

    #[test]
    fn parse_record_definition() {
        for (source, expected) in &[
            (
                "type Foo {}",
                RecordDefinition::new("Foo", vec![], Position::fake()),
            ),
            (
                "type Foo {foo number}",
                RecordDefinition::new(
                    "Foo",
                    vec![types::RecordField::new(
                        "foo",
                        types::Number::new(Position::fake()),
                    )],
                    Position::fake(),
                ),
            ),
            (
                "type Foo {foo number bar number}",
                RecordDefinition::new(
                    "Foo",
                    vec![
                        types::RecordField::new("foo", types::Number::new(Position::fake())),
                        types::RecordField::new("bar", types::Number::new(Position::fake())),
                    ],
                    Position::fake(),
                ),
            ),
        ] {
            assert_eq!(
                &record_definition().parse(stream(source, "")).unwrap().0,
                expected
            );
        }
    }

    #[test]
    fn parse_type_alias() {
        for (source, expected) in &[
            (
                "type foo=number",
                TypeAlias::new(
                    "foo",
                    types::Number::new(Position::fake()),
                    Position::fake(),
                ),
            ),
            (
                "type foo = number",
                TypeAlias::new(
                    "foo",
                    types::Number::new(Position::fake()),
                    Position::fake(),
                ),
            ),
            (
                "type foo=number|none",
                TypeAlias::new(
                    "foo",
                    types::Union::new(
                        types::Number::new(Position::fake()),
                        types::None::new(Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
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
                types::Boolean::new(Position::fake()).into()
            );
            assert_eq!(
                type_().parse(stream("none", "")).unwrap().0,
                types::None::new(Position::fake()).into()
            );
            assert_eq!(
                type_().parse(stream("number", "")).unwrap().0,
                types::Number::new(Position::fake()).into()
            );
            assert_eq!(
                type_().parse(stream("Foo", "")).unwrap().0,
                types::Reference::new("Foo", Position::fake()).into()
            );
            assert_eq!(
                type_().parse(stream("Foo'Bar", "")).unwrap().0,
                types::Reference::new("Foo'Bar", Position::fake()).into()
            );
            assert_eq!(
                type_().parse(stream("\\(number)number", "")).unwrap().0,
                types::Function::new(
                    vec![types::Number::new(Position::fake()).into()],
                    types::Number::new(Position::fake()),
                    Position::fake()
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
                        types::Number::new(Position::fake()).into(),
                        types::Number::new(Position::fake()).into(),
                    ],
                    types::Number::new(Position::fake()),
                    Position::fake()
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
                        vec![types::Number::new(Position::fake()).into()],
                        types::Number::new(Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    types::Number::new(Position::fake()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                type_().parse(stream("number|none", "")).unwrap().0,
                types::Union::new(
                    types::Number::new(Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                type_().parse(stream("boolean|number|none", "")).unwrap().0,
                types::Union::new(
                    types::Union::new(
                        types::Boolean::new(Position::fake()),
                        types::Number::new(Position::fake()),
                        Position::fake()
                    ),
                    types::None::new(Position::fake()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                type_()
                    .parse(stream("\\(number)number|none", ""))
                    .unwrap()
                    .0,
                types::Function::new(
                    vec![types::Number::new(Position::fake()).into()],
                    types::Union::new(
                        types::Number::new(Position::fake()),
                        types::None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
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
                        vec![types::Number::new(Position::fake()).into()],
                        types::Number::new(Position::fake()),
                        Position::fake()
                    ),
                    types::None::new(Position::fake()),
                    Position::fake()
                )
                .into()
            );
        }

        #[test]
        fn parse_any_type() {
            assert_eq!(
                any_type().parse(stream("any", "")).unwrap().0,
                types::Any::new(Position::fake())
            );
        }

        #[test]
        fn parse_reference_type() {
            assert!(type_().parse(stream("", "")).is_err());
            assert_eq!(
                type_().parse(stream("Foo", "")).unwrap().0,
                types::Reference::new("Foo", Position::fake()).into()
            );
            assert_eq!(
                type_().parse(stream("Foo'Bar", "")).unwrap().0,
                types::Reference::new("Foo'Bar", Position::fake()).into()
            );
        }

        #[test]
        fn parse_list_type() {
            assert_eq!(
                type_().parse(stream("[number]", "")).unwrap().0,
                types::List::new(types::Number::new(Position::fake()), Position::fake()).into()
            );

            assert_eq!(
                type_().parse(stream("[[number]]", "")).unwrap().0,
                types::List::new(
                    types::List::new(types::Number::new(Position::fake()), Position::fake()),
                    Position::fake()
                )
                .into()
            );

            assert_eq!(
                type_().parse(stream("[number]|[none]", "")).unwrap().0,
                types::Union::new(
                    types::List::new(types::Number::new(Position::fake()), Position::fake()),
                    types::List::new(types::None::new(Position::fake()), Position::fake()),
                    Position::fake()
                )
                .into()
            );

            assert_eq!(
                type_().parse(stream("\\([number])[none]", "")).unwrap().0,
                types::Function::new(
                    vec![
                        types::List::new(types::Number::new(Position::fake()), Position::fake())
                            .into()
                    ],
                    types::List::new(types::None::new(Position::fake()), Position::fake()),
                    Position::fake()
                )
                .into()
            );
        }

        #[test]
        fn parse_map_type() {
            assert_eq!(
                type_().parse(stream("{number:none}", "")).unwrap().0,
                types::Map::new(
                    types::Number::new(Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake()
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
                Number::new(
                    NumberRepresentation::FloatingPoint("1".into()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("x", "")).unwrap().0,
                Variable::new("x", Position::fake()).into()
            );
            assert_eq!(
                expression().parse(stream("x + 1", "")).unwrap().0,
                BinaryOperation::new(
                    BinaryOperator::Add,
                    Variable::new("x", Position::fake()),
                    Number::new(
                        NumberRepresentation::FloatingPoint("1".into()),
                        Position::fake()
                    ),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("x + y(z)", "")).unwrap().0,
                BinaryOperation::new(
                    BinaryOperator::Add,
                    Variable::new("x", Position::fake()),
                    Call::new(
                        Variable::new("y", Position::fake()),
                        vec![Variable::new("z", Position::fake()).into()],
                        Position::fake()
                    ),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                expression().parse(stream("(x + y)(z)", "")).unwrap().0,
                Call::new(
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        Variable::new("x", Position::fake()),
                        Variable::new("y", Position::fake()),
                        Position::fake()
                    ),
                    vec![Variable::new("z", Position::fake()).into()],
                    Position::fake()
                )
                .into()
            );
        }

        #[test]
        fn parse_deeply_nested_expression() {
            assert_eq!(
                expression().parse(stream("(((((42)))))", "")).unwrap().0,
                Number::new(
                    NumberRepresentation::FloatingPoint("42".into()),
                    Position::fake()
                )
                .into()
            )
        }

        #[test]
        fn parse_atomic_expression() {
            assert!(atomic_expression().parse(stream("", "")).is_err());
            assert_eq!(
                atomic_expression().parse(stream("1", "")).unwrap().0,
                Number::new(
                    NumberRepresentation::FloatingPoint("1".into()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                atomic_expression().parse(stream("x", "")).unwrap().0,
                Variable::new("x", Position::fake()).into()
            );
            assert_eq!(
                atomic_expression().parse(stream("(x)", "")).unwrap().0,
                Variable::new("x", Position::fake()).into()
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
                    vec![Argument::new("x", types::Number::new(Position::fake()))],
                    types::Number::new(Position::fake()),
                    Block::new(
                        vec![],
                        Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    Position::fake()
                ),
            );

            assert_eq!(
                lambda()
                    .parse(stream("\\(x number,y number)number{42}", ""))
                    .unwrap()
                    .0,
                Lambda::new(
                    vec![
                        Argument::new("x", types::Number::new(Position::fake())),
                        Argument::new("y", types::Number::new(Position::fake()))
                    ],
                    types::Number::new(Position::fake()),
                    Block::new(
                        vec![],
                        Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_lambda_with_reference_type() {
            assert_eq!(
                lambda().parse(stream("\\() Foo { 42 }", "")).unwrap().0,
                Lambda::new(
                    vec![],
                    types::Reference::new("Foo", Position::fake()),
                    Block::new(
                        vec![],
                        Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_block() {
            assert_eq!(
                block().parse(stream("{none}", "")).unwrap().0,
                Block::new(vec![], None::new(Position::fake()), Position::fake()),
            );
            assert_eq!(
                block().parse(stream("{none none}", "")).unwrap().0,
                Block::new(
                    vec![Statement::new(
                        None,
                        None::new(Position::fake()),
                        Position::fake()
                    )],
                    None::new(Position::fake()),
                    Position::fake()
                ),
            );
            assert_eq!(
                block().parse(stream("{none none none}", "")).unwrap().0,
                Block::new(
                    vec![
                        Statement::new(None, None::new(Position::fake()), Position::fake()),
                        Statement::new(None, None::new(Position::fake()), Position::fake())
                    ],
                    None::new(Position::fake()),
                    Position::fake()
                ),
            );
            assert_eq!(
                block().parse(stream("{x=none none}", "")).unwrap().0,
                Block::new(
                    vec![Statement::new(
                        Some("x".into()),
                        None::new(Position::fake()),
                        Position::fake()
                    )],
                    None::new(Position::fake()),
                    Position::fake()
                ),
            );
            assert_eq!(
                block().parse(stream("{x==x}", "")).unwrap().0,
                Block::new(
                    vec![],
                    BinaryOperation::new(
                        BinaryOperator::Equal,
                        Variable::new("x", Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_statement() {
            assert_eq!(
                statement().parse(stream("x==x", "")).unwrap().0,
                Statement::new(
                    None,
                    BinaryOperation::new(
                        BinaryOperator::Equal,
                        Variable::new("x", Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
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
                        Boolean::new(true, Position::fake()),
                        Block::new(
                            vec![],
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                    )],
                    Block::new(
                        vec![],
                        Number::new(
                            NumberRepresentation::FloatingPoint("13".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    Position::fake(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream("if if true {true}else{true}{42}else{13}", ""))
                    .unwrap()
                    .0,
                If::new(
                    vec![IfBranch::new(
                        If::new(
                            vec![IfBranch::new(
                                Boolean::new(true, Position::fake()),
                                Block::new(
                                    vec![],
                                    Boolean::new(true, Position::fake()),
                                    Position::fake()
                                ),
                            )],
                            Block::new(
                                vec![],
                                Boolean::new(true, Position::fake()),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        Block::new(
                            vec![],
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                    )],
                    Block::new(
                        vec![],
                        Number::new(
                            NumberRepresentation::FloatingPoint("13".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    Position::fake(),
                )
            );
            assert_eq!(
                if_()
                    .parse(stream("if true {1}else if true {2}else{3}", ""))
                    .unwrap()
                    .0,
                If::new(
                    vec![
                        IfBranch::new(
                            Boolean::new(true, Position::fake()),
                            Block::new(
                                vec![],
                                Number::new(
                                    NumberRepresentation::FloatingPoint("1".into()),
                                    Position::fake()
                                ),
                                Position::fake()
                            ),
                        ),
                        IfBranch::new(
                            Boolean::new(true, Position::fake()),
                            Block::new(
                                vec![],
                                Number::new(
                                    NumberRepresentation::FloatingPoint("2".into()),
                                    Position::fake()
                                ),
                                Position::fake()
                            ),
                        )
                    ],
                    Block::new(
                        vec![],
                        Number::new(
                            NumberRepresentation::FloatingPoint("3".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    Position::fake(),
                )
            );
        }

        #[test]
        fn parse_if_with_equal_operator() {
            assert_eq!(
                expression()
                    .parse(stream("if x==y {none}else{none}", ""))
                    .unwrap()
                    .0,
                If::new(
                    vec![IfBranch::new(
                        BinaryOperation::new(
                            BinaryOperator::Equal,
                            Variable::new("x", Position::fake()),
                            Variable::new("y", Position::fake()),
                            Position::fake()
                        ),
                        Block::new(vec![], None::new(Position::fake()), Position::fake()),
                    )],
                    Block::new(vec![], None::new(Position::fake()), Position::fake()),
                    Position::fake(),
                )
                .into()
            );
        }

        #[test]
        fn parse_if_type() {
            assert_eq!(
                if_type()
                    .parse(stream("if x=y as boolean {none}else{none}", ""))
                    .unwrap()
                    .0,
                IfType::new(
                    "x",
                    Variable::new("y", Position::fake()),
                    vec![IfTypeBranch::new(
                        types::Boolean::new(Position::fake()),
                        Block::new(vec![], None::new(Position::fake()), Position::fake()),
                    )],
                    Some(Block::new(
                        vec![],
                        None::new(Position::fake()),
                        Position::fake()
                    )),
                    Position::fake(),
                )
            );

            assert_eq!(
                if_type()
                    .parse(stream(
                        "if x=y as boolean{none}else if none{none}else{none}",
                        ""
                    ))
                    .unwrap()
                    .0,
                IfType::new(
                    "x",
                    Variable::new("y", Position::fake()),
                    vec![
                        IfTypeBranch::new(
                            types::Boolean::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
                        ),
                        IfTypeBranch::new(
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
                        )
                    ],
                    Some(Block::new(
                        vec![],
                        None::new(Position::fake()),
                        Position::fake()
                    )),
                    Position::fake()
                )
            );

            assert_eq!(
                if_type()
                    .parse(stream("if x=y as boolean{none}else if none{none}", ""))
                    .unwrap()
                    .0,
                IfType::new(
                    "x",
                    Variable::new("y", Position::fake()),
                    vec![
                        IfTypeBranch::new(
                            types::Boolean::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
                        ),
                        IfTypeBranch::new(
                            types::None::new(Position::fake()),
                            Block::new(vec![], None::new(Position::fake()), Position::fake()),
                        )
                    ],
                    None,
                    Position::fake()
                )
            );
        }

        #[test]
        fn parse_if_list() {
            assert_eq!(
                if_list()
                    .parse(stream("if[x,...xs]=xs {none}else{none}", ""))
                    .unwrap()
                    .0,
                IfList::new(
                    Variable::new("xs", Position::fake()),
                    "x",
                    "xs",
                    Block::new(vec![], None::new(Position::fake()), Position::fake()),
                    Block::new(vec![], None::new(Position::fake()), Position::fake()),
                    Position::fake(),
                )
            );
        }

        #[test]
        fn parse_if_map() {
            assert_eq!(
                if_map()
                    .parse(stream("if x=xs[42]{none}else{none}", ""))
                    .unwrap()
                    .0,
                IfMap::new(
                    "x",
                    Variable::new("xs", Position::fake()),
                    Number::new(
                        NumberRepresentation::FloatingPoint("42".into()),
                        Position::fake()
                    ),
                    Block::new(vec![], None::new(Position::fake()), Position::fake()),
                    Block::new(vec![], None::new(Position::fake()), Position::fake()),
                    Position::fake(),
                )
            );
        }

        mod call {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn parse_call() {
                assert_eq!(
                    expression().parse(stream("f()", "")).unwrap().0,
                    Call::new(
                        Variable::new("f", Position::fake()),
                        vec![],
                        Position::fake()
                    )
                    .into()
                );

                assert_eq!(
                    expression().parse(stream("f()()", "")).unwrap().0,
                    Call::new(
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        vec![],
                        Position::fake()
                    )
                    .into()
                );

                assert_eq!(
                    expression().parse(stream("f(1)", "")).unwrap().0,
                    Call::new(
                        Variable::new("f", Position::fake()),
                        vec![Number::new(
                            NumberRepresentation::FloatingPoint("1".into()),
                            Position::fake()
                        )
                        .into()],
                        Position::fake()
                    )
                    .into()
                );

                assert_eq!(
                    expression().parse(stream("f(1,)", "")).unwrap().0,
                    Call::new(
                        Variable::new("f", Position::fake()),
                        vec![Number::new(
                            NumberRepresentation::FloatingPoint("1".into()),
                            Position::fake()
                        )
                        .into()],
                        Position::fake()
                    )
                    .into()
                );

                assert_eq!(
                    expression().parse(stream("f(1, 2)", "")).unwrap().0,
                    Call::new(
                        Variable::new("f", Position::fake()),
                        vec![
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake()
                            )
                            .into(),
                            Number::new(
                                NumberRepresentation::FloatingPoint("2".into()),
                                Position::fake()
                            )
                            .into()
                        ],
                        Position::fake()
                    )
                    .into()
                );

                assert_eq!(
                    expression().parse(stream("f(1, 2,)", "")).unwrap().0,
                    Call::new(
                        Variable::new("f", Position::fake()),
                        vec![
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake()
                            )
                            .into(),
                            Number::new(
                                NumberRepresentation::FloatingPoint("2".into()),
                                Position::fake()
                            )
                            .into()
                        ],
                        Position::fake()
                    )
                    .into()
                );
            }

            #[test]
            fn fail_to_parse_call() {
                let source = "f(1+)";

                insta::assert_debug_snapshot!(expression()
                    .parse(stream(source, ""))
                    .map_err(|error| ParseError::new(source, "", error))
                    .err());
            }
        }

        #[test]
        fn parse_try_operation() {
            assert_eq!(
                expression().parse(stream("x?", "")).unwrap().0,
                UnaryOperation::new(
                    UnaryOperator::Try,
                    Variable::new("x", Position::fake()),
                    Position::fake()
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
                        Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                ),
                (
                    "!f(42)",
                    UnaryOperation::new(
                        UnaryOperator::Not,
                        Call::new(
                            Variable::new("f", Position::fake()),
                            vec![Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake(),
                            )
                            .into()],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                ),
                (
                    "!if true {true}else{true}",
                    UnaryOperation::new(
                        UnaryOperator::Not,
                        If::new(
                            vec![IfBranch::new(
                                Boolean::new(true, Position::fake()),
                                Block::new(
                                    vec![],
                                    Boolean::new(true, Position::fake()),
                                    Position::fake(),
                                ),
                            )],
                            Block::new(
                                vec![],
                                Boolean::new(true, Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                ),
                (
                    "!!42",
                    UnaryOperation::new(
                        UnaryOperator::Not,
                        UnaryOperation::new(
                            UnaryOperator::Not,
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Position::fake(),
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
        fn parse_spawn_operation() {
            assert_eq!(
                spawn_operation()
                    .parse(stream("go \\() number { 42 }", ""))
                    .unwrap()
                    .0,
                SpawnOperation::new(
                    Lambda::new(
                        vec![],
                        types::Number::new(Position::fake()),
                        Block::new(
                            vec![],
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        Position::fake()
                    ),
                    Position::fake()
                )
            );
        }

        #[test]
        fn parse_prefix_operator() {
            assert!(prefix_operator().parse(stream("", "")).is_err());

            for (source, expected) in &[("!", UnaryOperator::Not)] {
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
                        Number::new(
                            NumberRepresentation::FloatingPoint("1".into()),
                            Position::fake(),
                        ),
                        Number::new(
                            NumberRepresentation::FloatingPoint("1".into()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "1+1+1",
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        BinaryOperation::new(
                            BinaryOperator::Add,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake(),
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Number::new(
                            NumberRepresentation::FloatingPoint("1".into()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "1+(1+1)",
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        Number::new(
                            NumberRepresentation::FloatingPoint("1".into()),
                            Position::fake(),
                        ),
                        BinaryOperation::new(
                            BinaryOperator::Add,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake(),
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "1*2-3",
                    BinaryOperation::new(
                        BinaryOperator::Subtract,
                        BinaryOperation::new(
                            BinaryOperator::Multiply,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake(),
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("2".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Number::new(
                            NumberRepresentation::FloatingPoint("3".into()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "1+2*3",
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        Number::new(
                            NumberRepresentation::FloatingPoint("1".into()),
                            Position::fake(),
                        ),
                        BinaryOperation::new(
                            BinaryOperator::Multiply,
                            Number::new(
                                NumberRepresentation::FloatingPoint("2".into()),
                                Position::fake(),
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("3".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "1*2-3/4",
                    BinaryOperation::new(
                        BinaryOperator::Subtract,
                        BinaryOperation::new(
                            BinaryOperator::Multiply,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake(),
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("2".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        BinaryOperation::new(
                            BinaryOperator::Divide,
                            Number::new(
                                NumberRepresentation::FloatingPoint("3".into()),
                                Position::fake(),
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("4".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "1==1",
                    BinaryOperation::new(
                        BinaryOperator::Equal,
                        Number::new(
                            NumberRepresentation::FloatingPoint("1".into()),
                            Position::fake(),
                        ),
                        Number::new(
                            NumberRepresentation::FloatingPoint("1".into()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "true&true",
                    BinaryOperation::new(
                        BinaryOperator::And,
                        Boolean::new(true, Position::fake()),
                        Boolean::new(true, Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "true|true",
                    BinaryOperation::new(
                        BinaryOperator::Or,
                        Boolean::new(true, Position::fake()),
                        Boolean::new(true, Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "true&1<2",
                    BinaryOperation::new(
                        BinaryOperator::And,
                        Boolean::new(true, Position::fake()),
                        BinaryOperation::new(
                            BinaryOperator::LessThan,
                            Number::new(
                                NumberRepresentation::FloatingPoint("1".into()),
                                Position::fake(),
                            ),
                            Number::new(
                                NumberRepresentation::FloatingPoint("2".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "true|true&true",
                    BinaryOperation::new(
                        BinaryOperator::Or,
                        Boolean::new(true, Position::fake()),
                        BinaryOperation::new(
                            BinaryOperator::And,
                            Boolean::new(true, Position::fake()),
                            Boolean::new(true, Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "x+x",
                    BinaryOperation::new(
                        BinaryOperator::Add,
                        Variable::new("x", Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
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
                record().parse(stream("Foo{}", "")).unwrap().0,
                Record::new("Foo", None, vec![], Position::fake())
            );

            assert_eq!(
                expression().parse(stream("Foo{foo:42}", "")).unwrap().0,
                Record::new(
                    "Foo",
                    None,
                    vec![RecordField::new(
                        "foo",
                        Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    Position::fake()
                )
                .into()
            );

            assert_eq!(
                record().parse(stream("Foo{foo:42}", "")).unwrap().0,
                Record::new(
                    "Foo",
                    None,
                    vec![RecordField::new(
                        "foo",
                        Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    Position::fake()
                )
            );

            assert_eq!(
                record().parse(stream("Foo{foo:42,bar:42}", "")).unwrap().0,
                Record::new(
                    "Foo",
                    None,
                    vec![
                        RecordField::new(
                            "foo",
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        RecordField::new(
                            "bar",
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        )
                    ],
                    Position::fake()
                )
            );

            assert!(record().parse(stream("Foo{foo:42,foo:42}", "")).is_err());

            assert_eq!(
                expression()
                    .parse(stream("foo(Foo{foo:42})", ""))
                    .unwrap()
                    .0,
                Call::new(
                    Variable::new("foo", Position::fake()),
                    vec![Record::new(
                        "Foo",
                        None,
                        vec![RecordField::new(
                            "foo",
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            ),
                            Position::fake()
                        )],
                        Position::fake()
                    )
                    .into()],
                    Position::fake()
                )
                .into()
            );

            assert_eq!(
                record().parse(stream("Foo{foo:bar(42)}", "")).unwrap().0,
                Record::new(
                    "Foo",
                    None,
                    vec![RecordField::new(
                        "foo",
                        Call::new(
                            Variable::new("bar", Position::fake()),
                            vec![Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake()
                            )
                            .into()],
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    Position::fake()
                )
            );

            assert!(record().parse(stream("Foo{...foo,}", "")).is_err());

            assert_eq!(
                record().parse(stream("Foo{...foo,bar:42}", "")).unwrap().0,
                Record::new(
                    "Foo",
                    Some(Variable::new("foo", Position::fake()).into()),
                    vec![RecordField::new(
                        "bar",
                        Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    Position::fake()
                )
            );

            assert_eq!(
                record().parse(stream("Foo{...foo,bar:42,}", "")).unwrap().0,
                Record::new(
                    "Foo",
                    Some(Variable::new("foo", Position::fake()).into()),
                    vec![RecordField::new(
                        "bar",
                        Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    Position::fake()
                )
            );

            assert_eq!(
                expression()
                    .parse(stream("Foo{...foo,bar:42}", ""))
                    .unwrap()
                    .0,
                Record::new(
                    "Foo",
                    Some(Variable::new("foo", Position::fake()).into()),
                    vec![RecordField::new(
                        "bar",
                        Number::new(
                            NumberRepresentation::FloatingPoint("42".into()),
                            Position::fake()
                        ),
                        Position::fake()
                    )],
                    Position::fake()
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
                Variable::new("x", Position::fake()),
            );

            assert_eq!(
                variable().parse(stream("Foo.x", "")).unwrap().0,
                Variable::new("Foo", Position::fake()),
            );
        }

        #[test]
        fn parse_boolean_literal() {
            assert!(boolean_literal().parse(stream("", "")).is_err());
            assert_eq!(
                boolean_literal().parse(stream("false", "")).unwrap().0,
                Boolean::new(false, Position::fake())
            );
            assert_eq!(
                boolean_literal().parse(stream("true", "")).unwrap().0,
                Boolean::new(true, Position::fake())
            );
        }

        #[test]
        fn parse_none_literal() {
            assert!(none_literal().parse(stream("", "")).is_err());
            assert_eq!(
                none_literal().parse(stream("none", "")).unwrap().0,
                None::new(Position::fake())
            );
        }

        #[test]
        fn parse_number_literal() {
            assert!(number_literal().parse(stream("", "")).is_err());
            assert!(number_literal().parse(stream("foo", "")).is_err());
            assert!(number_literal().parse(stream("01", "")).is_err());

            for (source, value) in [
                ("0", NumberRepresentation::FloatingPoint("0".into())),
                ("1", NumberRepresentation::FloatingPoint("1".into())),
                (
                    "123456789",
                    NumberRepresentation::FloatingPoint("123456789".into()),
                ),
                ("-1", NumberRepresentation::FloatingPoint("-1".into())),
                ("0.1", NumberRepresentation::FloatingPoint("0.1".into())),
                ("0.01", NumberRepresentation::FloatingPoint("0.01".into())),
                ("0b1", NumberRepresentation::Binary("1".into())),
                ("0b10", NumberRepresentation::Binary("10".into())),
                ("0x1", NumberRepresentation::Hexadecimal("1".into())),
                ("0xFA", NumberRepresentation::Hexadecimal("fa".into())),
                ("0xfa", NumberRepresentation::Hexadecimal("fa".into())),
            ] {
                assert_eq!(
                    number_literal().parse(stream(source, "")).unwrap().0,
                    Number::new(value, Position::fake())
                );
            }
        }

        #[test]
        fn parse_string_literal() {
            assert!(string_literal().parse(stream("", "")).is_err());
            assert!(string_literal().parse(stream("foo", "")).is_err());

            for (source, value) in &[
                (r#""""#, ""),
                (r#""foo""#, "foo"),
                (r#""foo bar""#, "foo bar"),
                (r#""\"""#, "\\\""),
                (r#""\n""#, "\\n"),
                (r#""\r""#, "\\r"),
                (r#""\t""#, "\\t"),
                (r#""\\""#, "\\\\"),
                (r#""\x42""#, "\\x42"),
                (r#""\n\n""#, "\\n\\n"),
            ] {
                assert_eq!(
                    string_literal().parse(stream(source, "")).unwrap().0,
                    ByteString::new(*value, Position::fake())
                );
            }
        }

        #[test]
        fn parse_list() {
            for (source, target) in vec![
                (
                    "[none]",
                    List::new(types::None::new(Position::fake()), vec![], Position::fake()),
                ),
                (
                    "[none none]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![ListElement::Single(None::new(Position::fake()).into())],
                        Position::fake(),
                    ),
                ),
                (
                    "[none none,]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![ListElement::Single(None::new(Position::fake()).into())],
                        Position::fake(),
                    ),
                ),
                (
                    "[none none,none]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![
                            ListElement::Single(None::new(Position::fake()).into()),
                            ListElement::Single(None::new(Position::fake()).into()),
                        ],
                        Position::fake(),
                    ),
                ),
                (
                    "[none none,none,]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![
                            ListElement::Single(None::new(Position::fake()).into()),
                            ListElement::Single(None::new(Position::fake()).into()),
                        ],
                        Position::fake(),
                    ),
                ),
                (
                    "[none ...foo]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![ListElement::Multiple(
                            Variable::new("foo", Position::fake()).into(),
                        )],
                        Position::fake(),
                    ),
                ),
                (
                    "[none ...foo,]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![ListElement::Multiple(
                            Variable::new("foo", Position::fake()).into(),
                        )],
                        Position::fake(),
                    ),
                ),
                (
                    "[none ...foo,...bar]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![
                            ListElement::Multiple(Variable::new("foo", Position::fake()).into()),
                            ListElement::Multiple(Variable::new("bar", Position::fake()).into()),
                        ],
                        Position::fake(),
                    ),
                ),
                (
                    "[none ...foo,...bar,]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![
                            ListElement::Multiple(Variable::new("foo", Position::fake()).into()),
                            ListElement::Multiple(Variable::new("bar", Position::fake()).into()),
                        ],
                        Position::fake(),
                    ),
                ),
                (
                    "[none foo,...bar]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![
                            ListElement::Single(Variable::new("foo", Position::fake()).into()),
                            ListElement::Multiple(Variable::new("bar", Position::fake()).into()),
                        ],
                        Position::fake(),
                    ),
                ),
                (
                    "[none ...foo,bar]",
                    List::new(
                        types::None::new(Position::fake()),
                        vec![
                            ListElement::Multiple(Variable::new("foo", Position::fake()).into()),
                            ListElement::Single(Variable::new("bar", Position::fake()).into()),
                        ],
                        Position::fake(),
                    ),
                ),
            ] {
                assert_eq!(
                    expression().parse(stream(source, "")).unwrap().0,
                    target.into()
                );
            }
        }

        #[test]
        fn parse_list_comprehension() {
            for (source, target) in vec![
                (
                    "[none x for x in xs]",
                    ListComprehension::new(
                        types::None::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        "x",
                        Variable::new("xs", Position::fake()),
                        Position::fake(),
                    ),
                ),
                (
                    "[number x + 42 for x in xs]",
                    ListComprehension::new(
                        types::Number::new(Position::fake()),
                        BinaryOperation::new(
                            BinaryOperator::Add,
                            Variable::new("x", Position::fake()),
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        "x",
                        Variable::new("xs", Position::fake()),
                        Position::fake(),
                    ),
                ),
            ] {
                assert_eq!(
                    list_comprehension().parse(stream(source, "")).unwrap().0,
                    target
                );
            }
        }

        #[test]
        fn parse_map() {
            for (source, target) in vec![
                (
                    "{none:none}",
                    Map::new(
                        types::None::new(Position::fake()),
                        types::None::new(Position::fake()),
                        vec![],
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "{none:none none:none}",
                    Map::new(
                        types::None::new(Position::fake()),
                        types::None::new(Position::fake()),
                        vec![MapEntry::new(
                            None::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        )
                        .into()],
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "{number:none 1:none,2:none}",
                    Map::new(
                        types::Number::new(Position::fake()),
                        types::None::new(Position::fake()),
                        vec![
                            MapEntry::new(
                                Number::new(
                                    NumberRepresentation::FloatingPoint("1".into()),
                                    Position::fake(),
                                ),
                                None::new(Position::fake()),
                                Position::fake(),
                            )
                            .into(),
                            MapEntry::new(
                                Number::new(
                                    NumberRepresentation::FloatingPoint("2".into()),
                                    Position::fake(),
                                ),
                                None::new(Position::fake()),
                                Position::fake(),
                            )
                            .into(),
                        ],
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "{none:none ...none}",
                    Map::new(
                        types::None::new(Position::fake()),
                        types::None::new(Position::fake()),
                        vec![MapElement::Map(None::new(Position::fake()).into())],
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "{none:none none}",
                    Map::new(
                        types::None::new(Position::fake()),
                        types::None::new(Position::fake()),
                        vec![MapElement::Removal(None::new(Position::fake()).into())],
                        Position::fake(),
                    )
                    .into(),
                ),
            ] {
                assert_eq!(expression().parse(stream(source, "")).unwrap().0, target);
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
        assert!(keyword("type").parse(stream("bar", "")).is_err());
        // spell-checker: disable-next-line
        assert!(keyword("type").parse(stream("typer", "")).is_err());
        assert!(keyword("type").parse(stream("type_", "")).is_err());
        assert!(keyword("type").parse(stream("type", "")).is_ok());
    }

    #[test]
    fn parse_sign() {
        assert!(sign("+").parse(stream("", "")).is_err());
        assert!(sign("+").parse(stream("-", "")).is_err());
        assert!(sign("+").parse(stream("+", "")).is_ok());
        assert!(sign("++").parse(stream("++", "")).is_ok());
        assert!(sign("+").parse(stream("++", "")).is_err());
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
