use crate::{
    combinator::{separated_or_terminated_list0, separated_or_terminated_list1},
    error::NomError,
    input::{self, Input},
    operations::{reduce_operations, SuffixOperator},
};
use ast::{types::Type, *};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1, alphanumeric1, char, digit1, multispace0, multispace1, none_of, one_of,
    },
    combinator::{
        all_consuming, cut, into, map, not, opt, peek, recognize, success, value, verify,
    },
    error::context,
    multi::{count, many0, many0_count, many1, separated_list1},
    number::complete::recognize_float,
    sequence::{delimited, pair, preceded, terminated, tuple},
    Parser,
};
use position::Position;
use std::{collections::HashSet, str};

const KEYWORDS: &[&str] = &[
    "as", "else", "export", "for", "foreign", "if", "in", "import", "type",
];
const OPERATOR_CHARACTERS: &str = "+-*/=<>&|!?";
const OPERATOR_MODIFIERS: &str = "=";

type IResult<'a, T> = nom::IResult<Input<'a>, T, NomError<'a>>;

pub fn module(input: Input) -> IResult<Module> {
    map(
        all_consuming(tuple((
            position,
            many0(import),
            many0(foreign_import),
            many0(alt((into(type_alias), into(record_definition)))),
            many0(function_definition),
            blank,
        ))),
        |(position, imports, foreign_imports, type_definitions, definitions, _)| {
            Module::new(
                imports,
                foreign_imports,
                type_definitions,
                definitions,
                position(),
            )
        },
    )(input)
}

pub fn comments(input: Input) -> IResult<Vec<Comment>> {
    map(
        all_consuming(many0(tuple((
            multispace0,
            alt((
                map(comment, Some),
                map(raw_string_literal, |_| None),
                map(none_of("\"#"), |_| None),
            )),
            multispace0,
        )))),
        |comments| {
            comments
                .into_iter()
                .flat_map(|(_, comment, _)| comment)
                .collect()
        },
    )(input)
}

fn import(input: Input) -> IResult<Import> {
    context(
        "import",
        map(
            tuple((
                position,
                keyword("import"),
                module_path,
                cut(tuple((
                    opt(preceded(keyword("as"), identifier)),
                    opt(delimited(
                        sign("{"),
                        separated_or_terminated_list1(sign(","), unqualified_name),
                        sign("}"),
                    )),
                ))),
            )),
            |(position, _, path, (prefix, names))| {
                Import::new(path, prefix, names.unwrap_or_default(), position())
            },
        ),
    )(input)
}

fn unqualified_name(input: Input) -> IResult<UnqualifiedName> {
    map(
        token(tuple((position, identifier))),
        |(position, identifier)| UnqualifiedName::new(identifier, position()),
    )(input)
}

fn module_path(input: Input) -> IResult<ModulePath> {
    context(
        "module path",
        token(alt((
            into(external_module_path),
            into(internal_module_path),
        ))),
    )(input)
}

fn internal_module_path(input: Input) -> IResult<InternalModulePath> {
    context(
        "internal module path",
        map(module_path_components(identifier), InternalModulePath::new),
    )(input)
}

fn external_module_path(input: Input) -> IResult<ExternalModulePath> {
    context(
        "external module path",
        map(
            tuple((
                identifier,
                cut(module_path_components(public_module_path_component)),
            )),
            |(package, components)| ExternalModulePath::new(package, components),
        ),
    )(input)
}

fn module_path_components<'a>(
    component: impl Parser<Input<'a>, String, NomError<'a>>,
) -> impl FnMut(Input<'a>) -> IResult<'a, Vec<String>> {
    many1(preceded(tag(IDENTIFIER_SEPARATOR), component))
}

fn public_module_path_component(input: Input) -> IResult<String> {
    context(
        "public module path component",
        verify(identifier, ast::analysis::is_name_public),
    )(input)
}

fn foreign_import(input: Input) -> IResult<ForeignImport> {
    context(
        "foreign import",
        map(
            tuple((
                position,
                keyword("import"),
                keyword("foreign"),
                cut(tuple((opt(calling_convention), identifier, type_))),
            )),
            |(position, _, _, (calling_convention, name, type_))| {
                ForeignImport::new(
                    &name,
                    calling_convention.unwrap_or_default(),
                    type_,
                    position(),
                )
            },
        ),
    )(input)
}

fn calling_convention(input: Input) -> IResult<CallingConvention> {
    context(
        "calling convention",
        value(
            CallingConvention::C,
            verify(string_literal, |string| string.value() == "c"),
        ),
    )(input)
}

fn function_definition(input: Input) -> IResult<FunctionDefinition> {
    context(
        "function definition",
        map(
            tuple((
                position,
                opt(foreign_export),
                identifier,
                sign("="),
                cut(lambda),
            )),
            |(position, foreign_export, name, _, lambda)| {
                FunctionDefinition::new(name, lambda, foreign_export, position())
            },
        ),
    )(input)
}

fn foreign_export(input: Input) -> IResult<ForeignExport> {
    context(
        "foreign export",
        map(
            preceded(keyword("foreign"), opt(calling_convention)),
            |calling_convention| ForeignExport::new(calling_convention.unwrap_or_default()),
        ),
    )(input)
}

fn record_definition(input: Input) -> IResult<RecordDefinition> {
    context(
        "record definition",
        map(
            tuple((
                position,
                keyword("type"),
                identifier,
                sign("{"),
                cut(tuple((many0(record_field_definition), sign("}")))),
            )),
            |(position, _, name, _, (fields, _))| RecordDefinition::new(name, fields, position()),
        ),
    )(input)
}

fn record_field_definition(input: Input) -> IResult<types::RecordField> {
    context(
        "record field",
        map(
            tuple((position, identifier, type_)),
            |(position, name, type_)| types::RecordField::new(name, type_, position()),
        ),
    )(input)
}

fn type_alias(input: Input) -> IResult<TypeAlias> {
    context(
        "type alias",
        map(
            tuple((position, keyword("type"), identifier, sign("="), cut(type_))),
            |(position, _, name, _, type_)| TypeAlias::new(name, type_, position()),
        ),
    )(input)
}

fn type_(input: Input) -> IResult<Type> {
    context("type", alt((into(function_type), union_type)))(input)
}

fn function_type(input: Input) -> IResult<types::Function> {
    context(
        "function type",
        map(
            tuple((
                position,
                sign("\\("),
                cut(tuple((
                    separated_or_terminated_list0(sign(","), type_),
                    sign(")"),
                    type_,
                ))),
            )),
            |(position, _, (arguments, _, result))| {
                types::Function::new(arguments, result, position())
            },
        ),
    )(input)
}

fn union_type(input: Input) -> IResult<Type> {
    map(separated_list1(sign("|"), atomic_type), |types| {
        types
            .into_iter()
            .reduce(|lhs, rhs| types::Union::new(lhs.clone(), rhs, lhs.position().clone()).into())
            .unwrap()
    })(input)
}

fn list_type(input: Input) -> IResult<types::List> {
    context(
        "list type",
        map(
            tuple((position, sign("["), cut(terminated(type_, sign("]"))))),
            |(position, _, element)| types::List::new(element, position()),
        ),
    )(input)
}

fn map_type(input: Input) -> IResult<types::Map> {
    context(
        "map type",
        map(
            tuple((
                position,
                sign("{"),
                cut(tuple((type_, sign(":"), type_, sign("}")))),
            )),
            |(position, _, (key, _, value, _))| types::Map::new(key, value, position()),
        ),
    )(input)
}

fn atomic_type(input: Input) -> IResult<Type> {
    alt((
        into(reference_type),
        into(list_type),
        into(map_type),
        preceded(sign("("), cut(terminated(type_, sign(")")))),
    ))(input)
}

fn reference_type(input: Input) -> IResult<types::Reference> {
    context(
        "reference type",
        map(
            tuple((position, token(qualified_identifier))),
            |(position, identifier)| types::Reference::new(identifier, position()),
        ),
    )(input)
}

fn block(input: Input) -> IResult<Block> {
    context(
        "block",
        map(
            tuple((
                position,
                sign("{"),
                cut(terminated(
                    verify(many1(statement), |statements: &[_]| {
                        statements
                            .last()
                            .map(|statement| statement.name().is_none())
                            .unwrap_or_default()
                    }),
                    sign("}"),
                )),
            )),
            |(position, _, statements)| {
                Block::new(
                    statements[..statements.len() - 1].to_vec(),
                    statements.last().unwrap().expression().clone(),
                    position(),
                )
            },
        ),
    )(input)
}

fn statement(input: Input) -> IResult<Statement> {
    context(
        "statement",
        map(
            tuple((position, opt(terminated(identifier, sign("="))), expression)),
            |(position, name, expression)| Statement::new(name, expression, position()),
        ),
    )(input)
}

fn expression(input: Input) -> IResult<Expression> {
    context(
        "expression",
        map(
            tuple((
                prefix_operation_like,
                many0(map(
                    tuple((position, binary_operator, cut(prefix_operation_like))),
                    |(position, operator, expression)| (operator, expression, position()),
                )),
            )),
            |(expression, pairs)| reduce_operations(expression, &pairs),
        ),
    )(input)
}

fn binary_operator(input: Input) -> IResult<BinaryOperator> {
    context(
        "binary operator",
        alt((
            value(BinaryOperator::Add, sign("+")),
            value(BinaryOperator::Subtract, sign("-")),
            value(BinaryOperator::Multiply, sign("*")),
            value(BinaryOperator::Divide, sign("/")),
            value(BinaryOperator::Equal, sign("==")),
            value(BinaryOperator::NotEqual, sign("!=")),
            value(BinaryOperator::LessThanOrEqual, sign("<=")),
            value(BinaryOperator::LessThan, sign("<")),
            value(BinaryOperator::GreaterThanOrEqual, sign(">=")),
            value(BinaryOperator::GreaterThan, sign(">")),
            value(BinaryOperator::And, sign("&")),
            value(BinaryOperator::Or, sign("|")),
        )),
    )(input)
}

fn prefix_operation_like(input: Input) -> IResult<Expression> {
    alt((into(prefix_operation), into(suffix_operation_like)))(input)
}

fn prefix_operation(input: Input) -> IResult<UnaryOperation> {
    context(
        "prefix operation",
        map(
            tuple((position, prefix_operator, cut(prefix_operation_like))),
            |(position, operator, expression)| {
                UnaryOperation::new(operator, expression, position())
            },
        ),
    )(input)
}

fn prefix_operator(input: Input) -> IResult<UnaryOperator> {
    context("prefix operator", value(UnaryOperator::Not, sign("!")))(input)
}

fn suffix_operation_like(input: Input) -> IResult<Expression> {
    map(
        tuple((atomic_expression, many0(suffix_operator))),
        |(expression, suffix_operators)| {
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
    )(input)
}

fn suffix_operator(input: Input) -> IResult<SuffixOperator> {
    alt((call_operator, record_field_operator, try_operator))(input)
}

fn call_operator(input: Input) -> IResult<SuffixOperator> {
    // Do not allow any space before parentheses.
    context(
        "call",
        map(
            tuple((
                peek(position),
                tag("("),
                cut(terminated(
                    separated_or_terminated_list0(sign(","), expression),
                    sign(")"),
                )),
            )),
            |(position, _, arguments)| SuffixOperator::Call(arguments, position()),
        ),
    )(input)
}

fn record_field_operator(input: Input) -> IResult<SuffixOperator> {
    context(
        "record field",
        map(
            tuple((position, sign("."), cut(identifier))),
            |(position, _, identifier)| SuffixOperator::RecordField(identifier, position()),
        ),
    )(input)
}

fn try_operator(input: Input) -> IResult<SuffixOperator> {
    context(
        "try operator",
        map(tuple((position, sign("?"))), |(position, _)| {
            SuffixOperator::Try(position())
        }),
    )(input)
}

fn atomic_expression(input: Input) -> IResult<Expression> {
    alt((
        into(lambda),
        into(if_type),
        into(if_list),
        into(if_map),
        into(if_),
        into(record),
        into(list_comprehension),
        into(list_literal),
        into(map_literal),
        into(number_literal),
        into(string_literal),
        into(variable),
        delimited(sign("("), expression, sign(")")),
    ))(input)
}

fn lambda(input: Input) -> IResult<Lambda> {
    context(
        "function",
        map(
            tuple((
                position,
                sign("\\("),
                cut(tuple((
                    separated_or_terminated_list0(sign(","), argument),
                    sign(")"),
                    type_,
                    block,
                ))),
            )),
            |(position, _, (arguments, _, result_type, body))| {
                Lambda::new(arguments, result_type, body, position())
            },
        ),
    )(input)
}

fn argument(input: Input) -> IResult<Argument> {
    context(
        "argument",
        map(
            tuple((position, identifier, cut(type_))),
            |(position, name, type_)| Argument::new(name, type_, position()),
        ),
    )(input)
}

fn if_(input: Input) -> IResult<If> {
    context(
        "if",
        map(
            tuple((
                position,
                keyword("if"),
                cut(tuple((
                    if_branch,
                    many0(preceded(
                        tuple((keyword("else"), keyword("if"))),
                        cut(if_branch),
                    )),
                    keyword("else"),
                    block,
                ))),
            )),
            |(position, _, (first_branch, branches, _, else_block))| {
                If::new(
                    [first_branch].into_iter().chain(branches).collect(),
                    else_block,
                    position(),
                )
            },
        ),
    )(input)
}

fn if_branch(input: Input) -> IResult<IfBranch> {
    map(tuple((expression, block)), |(expression, block)| {
        IfBranch::new(expression, block)
    })(input)
}

fn if_list(input: Input) -> IResult<IfList> {
    context(
        "if list",
        map(
            tuple((
                position,
                keyword("if"),
                sign("["),
                cut(tuple((
                    identifier,
                    sign(","),
                    sign("..."),
                    identifier,
                    sign("]"),
                    sign("="),
                    expression,
                    block,
                    keyword("else"),
                    block,
                ))),
            )),
            |(position, _, _, (first_name, _, _, rest_name, _, _, argument, then, _, else_))| {
                IfList::new(argument, first_name, rest_name, then, else_, position())
            },
        ),
    )(input)
}

fn if_map(input: Input) -> IResult<IfMap> {
    context(
        "if map",
        map(
            tuple((
                position,
                keyword("if"),
                identifier,
                sign("="),
                expression,
                sign("["),
                cut(tuple((
                    expression,
                    sign("]"),
                    block,
                    keyword("else"),
                    block,
                ))),
            )),
            |(position, _, name, _, map, _, (key, _, then, _, else_))| {
                IfMap::new(name, map, key, then, else_, position())
            },
        ),
    )(input)
}

fn if_type(input: Input) -> IResult<IfType> {
    context(
        "if type",
        map(
            tuple((
                position,
                keyword("if"),
                identifier,
                sign("="),
                expression,
                keyword("as"),
                cut(tuple((
                    if_type_branch,
                    many0(preceded(
                        tuple((keyword("else"), keyword("if"))),
                        cut(if_type_branch),
                    )),
                    opt(preceded(keyword("else"), block)),
                ))),
            )),
            |(position, _, identifier, _, argument, _, (first_branch, branches, else_))| {
                IfType::new(
                    identifier,
                    argument,
                    [first_branch].into_iter().chain(branches).collect(),
                    else_,
                    position(),
                )
            },
        ),
    )(input)
}

fn if_type_branch(input: Input) -> IResult<IfTypeBranch> {
    map(tuple((type_, block)), |(type_, block)| {
        IfTypeBranch::new(type_, block)
    })(input)
}

fn record(input: Input) -> IResult<Record> {
    // TODO Disallow spaces before `{` for disambiguation?
    context(
        "record",
        map(
            tuple((
                position,
                qualified_identifier,
                sign("{"),
                verify(
                    alt((
                        preceded(
                            sign("..."),
                            cut(tuple((
                                map(terminated(expression, sign(",")), Some),
                                separated_or_terminated_list1(sign(","), record_field),
                            ))),
                        ),
                        tuple((
                            success(None),
                            separated_or_terminated_list0(sign(","), record_field),
                        )),
                    )),
                    |(_, fields)| {
                        fields.len()
                            == HashSet::<&str>::from_iter(fields.iter().map(|field| field.name()))
                                .len()
                    },
                ),
                sign("}"),
            )),
            |(position, name, _, (record, fields), _)| {
                Record::new(name, record, fields, position())
            },
        ),
    )(input)
}

fn record_field(input: Input) -> IResult<RecordField> {
    context(
        "record field",
        map(
            tuple((position, identifier, sign(":"), cut(expression))),
            |(position, name, _, expression)| RecordField::new(name, expression, position()),
        ),
    )(input)
}

fn number_literal(input: Input) -> IResult<Number> {
    context(
        "number",
        map(
            token(tuple((
                position,
                alt((binary_literal, hexadecimal_literal, decimal_literal)),
                peek(not(digit1)),
            ))),
            |(position, number, _)| Number::new(number, position()),
        ),
    )(input)
}

fn binary_literal(input: Input) -> IResult<NumberRepresentation> {
    context(
        "binary literal",
        map(
            preceded(tag("0b"), cut(many1(one_of("01")))),
            |characters| NumberRepresentation::Binary(String::from_iter(characters)),
        ),
    )(input)
}

fn hexadecimal_literal(input: Input) -> IResult<NumberRepresentation> {
    context(
        "hexadecimal literal",
        map(
            preceded(tag("0x"), cut(many1(hexadecimal_digit))),
            |characters| {
                NumberRepresentation::Hexadecimal(String::from_iter(characters).to_lowercase())
            },
        ),
    )(input)
}

fn hexadecimal_digit(input: Input) -> IResult<char> {
    one_of("0123456789abcdefABCDEF")(input)
}

fn decimal_literal(input: Input) -> IResult<NumberRepresentation> {
    context(
        "decimal literal",
        map(recognize_float, |characters: Input| {
            NumberRepresentation::FloatingPoint(
                str::from_utf8(characters.as_bytes()).unwrap().into(),
            )
        }),
    )(input)
}

fn string_literal(input: Input) -> IResult<ByteString> {
    context("string", token(raw_string_literal))(input)
}

fn raw_string_literal(input: Input) -> IResult<ByteString> {
    map(
        tuple((
            position,
            preceded(
                char('"'),
                cut(terminated(
                    many0(alt((
                        recognize(none_of("\\\"")),
                        tag("\\\\"),
                        tag("\\\""),
                        tag("\\n"),
                        tag("\\r"),
                        tag("\\t"),
                        recognize(tuple((tag("\\x"), count(hexadecimal_digit, 2)))),
                    ))),
                    char('"'),
                )),
            ),
        )),
        |(position, spans)| {
            ByteString::new(
                spans
                    .iter()
                    .map(|span| str::from_utf8(span.as_bytes()).unwrap())
                    .collect::<Vec<_>>()
                    .concat(),
                position(),
            )
        },
    )(input)
}

fn list_literal(input: Input) -> IResult<List> {
    context(
        "list",
        map(
            tuple((
                position,
                sign("["),
                cut(tuple((
                    type_,
                    separated_or_terminated_list0(sign(","), list_element),
                    sign("]"),
                ))),
            )),
            |(position, _, (type_, elements, _))| List::new(type_, elements, position()),
        ),
    )(input)
}

fn list_element(input: Input) -> IResult<ListElement> {
    alt((
        map(
            preceded(sign("..."), cut(expression)),
            ListElement::Multiple,
        ),
        map(expression, ListElement::Single),
    ))(input)
}

fn list_comprehension(input: Input) -> IResult<Expression> {
    context(
        "list comprehension",
        map(
            tuple((
                position,
                sign("["),
                type_,
                expression,
                many1(list_comprehension_branch),
                sign("]"),
            )),
            |(position, _, type_, element, branches, _)| {
                ListComprehension::new(type_, element, branches, position()).into()
            },
        ),
    )(input)
}

fn list_comprehension_branch(input: Input) -> IResult<ListComprehensionBranch> {
    context(
        "list comprehension branch",
        map(
            tuple((
                position,
                keyword("for"),
                cut(tuple((
                    identifier,
                    opt(preceded(sign(","), identifier)),
                    keyword("in"),
                    expression,
                ))),
            )),
            |(position, _, (element_name, value_name, _, iterator))| {
                ListComprehensionBranch::new(element_name, value_name, iterator, position())
            },
        ),
    )(input)
}

fn map_literal(input: Input) -> IResult<Map> {
    context(
        "map",
        map(
            tuple((
                position,
                sign("{"),
                cut(tuple((
                    type_,
                    sign(":"),
                    type_,
                    separated_or_terminated_list0(sign(","), map_element),
                    sign("}"),
                ))),
            )),
            |(position, _, (key_type, _, value_type, elements, _))| {
                Map::new(key_type, value_type, elements, position())
            },
        ),
    )(input)
}

fn map_element(input: Input) -> IResult<MapElement> {
    alt((
        map(
            tuple((position, expression, sign(":"), cut(expression))),
            |(position, key, _, value)| MapEntry::new(key, value, position()).into(),
        ),
        map(preceded(sign("..."), cut(expression)), MapElement::Map),
    ))(input)
}

fn variable(input: Input) -> IResult<Variable> {
    context(
        "variable",
        map(
            tuple((position, token(qualified_identifier))),
            |(position, identifier)| Variable::new(identifier, position()),
        ),
    )(input)
}

fn qualified_identifier(input: Input) -> IResult<String> {
    map(
        recognize(tuple((
            raw_identifier,
            opt(tuple((tag(IDENTIFIER_SEPARATOR), cut(raw_identifier)))),
        ))),
        |span| str::from_utf8(span.as_bytes()).unwrap().into(),
    )(input)
}

fn identifier(input: Input) -> IResult<String> {
    context("identifier", token(raw_identifier))(input)
}

fn raw_identifier(input: Input) -> IResult<String> {
    verify(unchecked_identifier, |identifier: &str| {
        !KEYWORDS.contains(&identifier)
    })(input)
}

fn unchecked_identifier(input: Input) -> IResult<String> {
    map(
        recognize(tuple((
            alt((value((), alpha1::<Input, _>), value((), char('_')))),
            many0_count(alt((value((), alphanumeric1), value((), char('_'))))),
        ))),
        |span| str::from_utf8(span.as_bytes()).unwrap().into(),
    )(input)
}

fn keyword(name: &'static str) -> impl FnMut(Input) -> IResult<()> {
    if !KEYWORDS.contains(&name) {
        unreachable!("undefined keyword");
    }

    move |input| {
        context(
            "keyword",
            value(
                (),
                token(tuple((
                    tag(name),
                    peek(not(alt((value((), alphanumeric1), value((), char('_')))))),
                ))),
            ),
        )(input)
    }
}

fn sign(sign: &'static str) -> impl Fn(Input) -> IResult<()> + Clone {
    move |input| {
        let parser = context("sign", token(tag(sign)));

        if sign
            .chars()
            .any(|character| OPERATOR_CHARACTERS.contains(character))
        {
            value((), tuple((parser, peek(not(one_of(OPERATOR_MODIFIERS))))))(input)
        } else {
            value((), parser)(input)
        }
    }
}

fn token<'a, O>(
    mut parser: impl Parser<Input<'a>, O, NomError<'a>>,
) -> impl FnMut(Input<'a>) -> IResult<'a, O> {
    move |input| {
        let (input, _) = blank(input)?;

        parser.parse(input)
    }
}

fn blank(input: Input) -> IResult<()> {
    value(
        (),
        many0_count(alt((value((), multispace1), skipped_comment))),
    )(input)
}

fn comment(input: Input) -> IResult<Comment> {
    context(
        "comment",
        map(
            tuple((comment_position, tag("#"), many0(none_of("\n\r")))),
            |(position, _, characters)| Comment::new(String::from_iter(characters), position),
        ),
    )(input)
}

// Optimize comment parsing by skipping contents.
fn skipped_comment(input: Input) -> IResult<()> {
    value((), pair(tag("#"), many0_count(none_of("\n\r"))))(input)
}

fn comment_position(input: Input) -> IResult<Position> {
    let (input, _) = multispace0(input)?;

    Ok((input, input::position(input)))
}

// Allocate position objects lazily.
fn position(input: Input) -> IResult<impl Fn() -> Position + '_> {
    let (input, _) = blank(input)?;

    Ok((input, move || input::position(input)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::input;
    use indoc::indoc;
    use position::test::PositionFake;
    use pretty_assertions::assert_eq;

    mod module {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_module() {
            assert_eq!(
                module(input("", "")).unwrap().1,
                Module::new(vec![], vec![], vec![], vec![], Position::fake())
            );
            assert_eq!(
                module(input(" ", "")).unwrap().1,
                Module::new(vec![], vec![], vec![], vec![], Position::fake())
            );
            assert_eq!(
                module(input("\n", "")).unwrap().1,
                Module::new(vec![], vec![], vec![], vec![], Position::fake())
            );
            assert_eq!(
                module(input("import Foo'Bar", "")).unwrap().1,
                Module::new(
                    vec![Import::new(
                        ExternalModulePath::new("Foo", vec!["Bar".into()]),
                        None,
                        vec![],
                        Position::fake()
                    )],
                    vec![],
                    vec![],
                    vec![],
                    Position::fake()
                )
            );
            assert_eq!(
                module(input("type foo = number", "")).unwrap().1,
                Module::new(
                    vec![],
                    vec![],
                    vec![TypeAlias::new(
                        "foo",
                        types::Reference::new("number", Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    vec![],
                    Position::fake()
                )
            );
            assert_eq!(
                module(input("x=\\(x number)number{42}", "")).unwrap().1,
                Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![FunctionDefinition::new(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Reference::new("number", Position::fake()),
                                Position::fake()
                            )],
                            types::Reference::new("number", Position::fake()),
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
                module(input(
                    "x=\\(x number)number{42}y=\\(y number)number{42}",
                    ""
                ))
                .unwrap()
                .1,
                Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![
                        FunctionDefinition::new(
                            "x",
                            Lambda::new(
                                vec![Argument::new(
                                    "x",
                                    types::Reference::new("number", Position::fake()),
                                    Position::fake()
                                )],
                                types::Reference::new("number", Position::fake()),
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
                        FunctionDefinition::new(
                            "y",
                            Lambda::new(
                                vec![Argument::new(
                                    "y",
                                    types::Reference::new("number", Position::fake()),
                                    Position::fake()
                                )],
                                types::Reference::new("number", Position::fake()),
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
                module(input("import Foo'Bar import foreign foo \\() number", ""))
                    .unwrap()
                    .1,
                Module::new(
                    vec![Import::new(
                        ExternalModulePath::new("Foo", vec!["Bar".into()]),
                        None,
                        vec![],
                        Position::fake()
                    )],
                    vec![ForeignImport::new(
                        "foo",
                        CallingConvention::Native,
                        types::Function::new(
                            vec![],
                            types::Reference::new("number", Position::fake()),
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
                module(input("type foo = number type bar {}", ""))
                    .unwrap()
                    .1,
                Module::new(
                    vec![],
                    vec![],
                    vec![
                        TypeAlias::new(
                            "foo",
                            types::Reference::new("number", Position::fake()),
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
        use crate::ParseError;

        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_import() {
            assert_eq!(
                import(input("import 'Foo", "")).unwrap().1,
                Import::new(
                    InternalModulePath::new(vec!["Foo".into()]),
                    None,
                    vec![],
                    Position::fake()
                ),
            );
            assert_eq!(
                import(input("import Foo'Bar", "")).unwrap().1,
                Import::new(
                    ExternalModulePath::new("Foo", vec!["Bar".into()]),
                    None,
                    vec![],
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_import_with_custom_prefix() {
            assert_eq!(
                import(input("import 'Foo as foo", "")).unwrap().1,
                Import::new(
                    InternalModulePath::new(vec!["Foo".into()]),
                    Some("foo".into()),
                    vec![],
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_unqualified_import() {
            assert_eq!(
                import(input("import 'Foo { Foo }", "")).unwrap().1,
                Import::new(
                    InternalModulePath::new(vec!["Foo".into()]),
                    None,
                    vec![UnqualifiedName::new("Foo", Position::fake())],
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_unqualified_import_with_multiple_identifiers() {
            assert_eq!(
                import(input("import 'Foo { Foo, Bar }", "")).unwrap().1,
                Import::new(
                    InternalModulePath::new(vec!["Foo".into()]),
                    None,
                    vec![
                        UnqualifiedName::new("Foo", Position::fake()),
                        UnqualifiedName::new("Bar", Position::fake())
                    ],
                    Position::fake()
                ),
            );
        }

        #[test]
        fn parse_module_path() {
            assert!(module_path(input("", "")).is_err());
            assert_eq!(
                module_path(input("'Foo", "")).unwrap().1,
                InternalModulePath::new(vec!["Foo".into()]).into(),
            );
            assert_eq!(
                module_path(input("Foo'Bar", "")).unwrap().1,
                ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
            );
        }

        #[test]
        fn parse_internal_module_path() {
            assert!(internal_module_path(input("", "")).is_err());
            assert_eq!(
                internal_module_path(input("'Foo", "")).unwrap().1,
                InternalModulePath::new(vec!["Foo".into()]),
            );
            assert_eq!(
                internal_module_path(input("'Foo'Bar", "")).unwrap().1,
                InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
            );
        }

        #[test]
        fn parse_external_module_path() {
            assert!(external_module_path(input("", "")).is_err());
            assert_eq!(
                external_module_path(input("Foo'Bar", "")).unwrap().1,
                ExternalModulePath::new("Foo", vec!["Bar".into()]),
            );
        }

        #[test]
        fn fail_to_parse_private_external_module_file() {
            let source = "Foo'bar";

            insta::assert_debug_snapshot!(external_module_path(input(source, ""))
                .map_err(|error| ParseError::new(source, "", error))
                .unwrap_err());
        }

        #[test]
        fn fail_to_parse_private_external_module_directory() {
            let source = "Foo'bar'Baz";

            insta::assert_debug_snapshot!(external_module_path(input(source, ""))
                .map_err(|error| ParseError::new(source, "", error))
                .unwrap_err());
        }
    }

    #[test]
    fn parse_foreign_import() {
        assert_eq!(
            foreign_import(input("import foreign foo \\(number) number", ""))
                .unwrap()
                .1,
            ForeignImport::new(
                "foo",
                CallingConvention::Native,
                types::Function::new(
                    vec![types::Reference::new("number", Position::fake()).into()],
                    types::Reference::new("number", Position::fake()),
                    Position::fake()
                ),
                Position::fake()
            ),
        );

        assert_eq!(
            foreign_import(input("import foreign \"c\" foo \\(number) number", ""))
                .unwrap()
                .1,
            ForeignImport::new(
                "foo",
                CallingConvention::C,
                types::Function::new(
                    vec![types::Reference::new("number", Position::fake()).into()],
                    types::Reference::new("number", Position::fake()),
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
                function_definition(input("x=\\(x number)number{42}", ""))
                    .unwrap()
                    .1,
                FunctionDefinition::new(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Reference::new("number", Position::fake()),
                            Position::fake()
                        )],
                        types::Reference::new("number", Position::fake()),
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
                function_definition(input("foreign x=\\(x number)number{42}", ""))
                    .unwrap()
                    .1,
                FunctionDefinition::new(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Reference::new("number", Position::fake()),
                            Position::fake()
                        )],
                        types::Reference::new("number", Position::fake()),
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
                function_definition(input("foreign \"c\" x=\\(x number)number{42}", ""))
                    .unwrap()
                    .1,
                FunctionDefinition::new(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Reference::new("number", Position::fake()),
                            Position::fake()
                        )],
                        types::Reference::new("number", Position::fake()),
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
                function_definition(input("importA = \\() number { 42 }", ""))
                    .unwrap()
                    .1,
                FunctionDefinition::new(
                    "importA",
                    Lambda::new(
                        vec![],
                        types::Reference::new("number", Position::fake()),
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
                        types::Reference::new("number", Position::fake()),
                        Position::fake(),
                    )],
                    Position::fake(),
                ),
            ),
            (
                "type Foo {foo number bar number}",
                RecordDefinition::new(
                    "Foo",
                    vec![
                        types::RecordField::new(
                            "foo",
                            types::Reference::new("number", Position::fake()),
                            Position::fake(),
                        ),
                        types::RecordField::new(
                            "bar",
                            types::Reference::new("number", Position::fake()),
                            Position::fake(),
                        ),
                    ],
                    Position::fake(),
                ),
            ),
        ] {
            assert_eq!(&record_definition(input(source, "")).unwrap().1, expected);
        }
    }

    #[test]
    fn parse_type_alias() {
        for (source, expected) in &[
            (
                "type foo=number",
                TypeAlias::new(
                    "foo",
                    types::Reference::new("number", Position::fake()),
                    Position::fake(),
                ),
            ),
            (
                "type foo = number",
                TypeAlias::new(
                    "foo",
                    types::Reference::new("number", Position::fake()),
                    Position::fake(),
                ),
            ),
            (
                "type foo=number|none",
                TypeAlias::new(
                    "foo",
                    types::Union::new(
                        types::Reference::new("number", Position::fake()),
                        types::Reference::new("none", Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
            ),
        ] {
            assert_eq!(&type_alias(input(source, "")).unwrap().1, expected);
        }
    }

    mod type_ {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_type() {
            assert!(type_(input("", "")).is_err());
            assert_eq!(
                type_(input("boolean", "")).unwrap().1,
                types::Reference::new("boolean", Position::fake()).into()
            );
            assert_eq!(
                type_(input("none", "")).unwrap().1,
                types::Reference::new("none", Position::fake()).into()
            );
            assert_eq!(
                type_(input("number", "")).unwrap().1,
                types::Reference::new("number", Position::fake()).into()
            );
            assert_eq!(
                type_(input("Foo", "")).unwrap().1,
                types::Reference::new("Foo", Position::fake()).into()
            );
            assert_eq!(
                type_(input("Foo'Bar", "")).unwrap().1,
                types::Reference::new("Foo'Bar", Position::fake()).into()
            );
            assert_eq!(
                type_(input("\\(number)number", "")).unwrap().1,
                types::Function::new(
                    vec![types::Reference::new("number", Position::fake()).into()],
                    types::Reference::new("number", Position::fake()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                type_(input("\\(number,number)number", "")).unwrap().1,
                types::Function::new(
                    vec![
                        types::Reference::new("number", Position::fake()).into(),
                        types::Reference::new("number", Position::fake()).into(),
                    ],
                    types::Reference::new("number", Position::fake()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                type_(input("\\(\\(number)number)number", "")).unwrap().1,
                types::Function::new(
                    vec![types::Function::new(
                        vec![types::Reference::new("number", Position::fake()).into()],
                        types::Reference::new("number", Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    types::Reference::new("number", Position::fake()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                type_(input("number|none", "")).unwrap().1,
                types::Union::new(
                    types::Reference::new("number", Position::fake()),
                    types::Reference::new("none", Position::fake()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                type_(input("boolean|number|none", "")).unwrap().1,
                types::Union::new(
                    types::Union::new(
                        types::Reference::new("boolean", Position::fake()),
                        types::Reference::new("number", Position::fake()),
                        Position::fake()
                    ),
                    types::Reference::new("none", Position::fake()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                type_(input("\\(number)number|none", "")).unwrap().1,
                types::Function::new(
                    vec![types::Reference::new("number", Position::fake()).into()],
                    types::Union::new(
                        types::Reference::new("number", Position::fake()),
                        types::Reference::new("none", Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                type_(input("(\\(number)number)|none", "")).unwrap().1,
                types::Union::new(
                    types::Function::new(
                        vec![types::Reference::new("number", Position::fake()).into()],
                        types::Reference::new("number", Position::fake()),
                        Position::fake()
                    ),
                    types::Reference::new("none", Position::fake()),
                    Position::fake()
                )
                .into()
            );
        }

        #[test]
        fn parse_reference_type() {
            assert!(type_(input("", "")).is_err());
            assert_eq!(
                type_(input("Foo", "")).unwrap().1,
                types::Reference::new("Foo", Position::fake()).into()
            );
            assert_eq!(
                type_(input("Foo'Bar", "")).unwrap().1,
                types::Reference::new("Foo'Bar", Position::fake()).into()
            );
        }

        #[test]
        fn parse_list_type() {
            assert_eq!(
                type_(input("[number]", "")).unwrap().1,
                types::List::new(
                    types::Reference::new("number", Position::fake()),
                    Position::fake()
                )
                .into()
            );

            assert_eq!(
                type_(input("[[number]]", "")).unwrap().1,
                types::List::new(
                    types::List::new(
                        types::Reference::new("number", Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                )
                .into()
            );

            assert_eq!(
                type_(input("[number]|[none]", "")).unwrap().1,
                types::Union::new(
                    types::List::new(
                        types::Reference::new("number", Position::fake()),
                        Position::fake()
                    ),
                    types::List::new(
                        types::Reference::new("none", Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                )
                .into()
            );

            assert_eq!(
                type_(input("\\([number])[none]", "")).unwrap().1,
                types::Function::new(
                    vec![types::List::new(
                        types::Reference::new("number", Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    types::List::new(
                        types::Reference::new("none", Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                )
                .into()
            );
        }

        #[test]
        fn parse_map_type() {
            assert_eq!(
                type_(input("{number:none}", "")).unwrap().1,
                types::Map::new(
                    types::Reference::new("number", Position::fake()),
                    types::Reference::new("none", Position::fake()),
                    Position::fake()
                )
                .into()
            );
        }
    }

    mod expression {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_expression() {
            assert!(expression(input("", "")).is_err());
            assert_eq!(
                expression(input("1", "")).unwrap().1,
                Number::new(
                    NumberRepresentation::FloatingPoint("1".into()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                expression(input("x", "")).unwrap().1,
                Variable::new("x", Position::fake()).into()
            );
            assert_eq!(
                expression(input("x + 1", "")).unwrap().1,
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
                expression(input("x + y(z)", "")).unwrap().1,
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
                expression(input("(x + y)(z)", "")).unwrap().1,
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
                expression(input("(((((42)))))", "")).unwrap().1,
                Number::new(
                    NumberRepresentation::FloatingPoint("42".into()),
                    Position::fake()
                )
                .into()
            )
        }

        #[test]
        fn parse_atomic_expression() {
            assert!(atomic_expression(input("", "")).is_err());
            assert_eq!(
                atomic_expression(input("1", "")).unwrap().1,
                Number::new(
                    NumberRepresentation::FloatingPoint("1".into()),
                    Position::fake()
                )
                .into()
            );
            assert_eq!(
                atomic_expression(input("x", "")).unwrap().1,
                Variable::new("x", Position::fake()).into()
            );
            assert_eq!(
                atomic_expression(input("(x)", "")).unwrap().1,
                Variable::new("x", Position::fake()).into()
            );
        }

        #[test]
        fn parse_lambda() {
            assert_eq!(
                lambda(input("\\(x number)number{42}", "")).unwrap().1,
                Lambda::new(
                    vec![Argument::new(
                        "x",
                        types::Reference::new("number", Position::fake()),
                        Position::fake()
                    )],
                    types::Reference::new("number", Position::fake()),
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
                lambda(input("\\(x number,y number)number{42}", ""))
                    .unwrap()
                    .1,
                Lambda::new(
                    vec![
                        Argument::new(
                            "x",
                            types::Reference::new("number", Position::fake()),
                            Position::fake()
                        ),
                        Argument::new(
                            "y",
                            types::Reference::new("number", Position::fake()),
                            Position::fake()
                        )
                    ],
                    types::Reference::new("number", Position::fake()),
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
                lambda(input("\\() Foo { 42 }", "")).unwrap().1,
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
                block(input("{none}", "")).unwrap().1,
                Block::new(
                    vec![],
                    Variable::new("none", Position::fake()),
                    Position::fake()
                ),
            );
            assert_eq!(
                block(input("{none none}", "")).unwrap().1,
                Block::new(
                    vec![Statement::new(
                        None,
                        Variable::new("none", Position::fake()),
                        Position::fake()
                    )],
                    Variable::new("none", Position::fake()),
                    Position::fake()
                ),
            );
            assert_eq!(
                block(input("{none none none}", "")).unwrap().1,
                Block::new(
                    vec![
                        Statement::new(
                            None,
                            Variable::new("none", Position::fake()),
                            Position::fake()
                        ),
                        Statement::new(
                            None,
                            Variable::new("none", Position::fake()),
                            Position::fake()
                        )
                    ],
                    Variable::new("none", Position::fake()),
                    Position::fake()
                ),
            );
            assert_eq!(
                block(input("{x=none none}", "")).unwrap().1,
                Block::new(
                    vec![Statement::new(
                        Some("x".into()),
                        Variable::new("none", Position::fake()),
                        Position::fake()
                    )],
                    Variable::new("none", Position::fake()),
                    Position::fake()
                ),
            );
            assert_eq!(
                block(input("{x==x}", "")).unwrap().1,
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
                statement(input("x==x", "")).unwrap().1,
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
                if_(input("if true { 42 } else { 13 }", "")).unwrap().1,
                If::new(
                    vec![IfBranch::new(
                        Variable::new("true", Position::fake()),
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
                if_(input("if if true {true}else{true}{42}else{13}", ""))
                    .unwrap()
                    .1,
                If::new(
                    vec![IfBranch::new(
                        If::new(
                            vec![IfBranch::new(
                                Variable::new("true", Position::fake()),
                                Block::new(
                                    vec![],
                                    Variable::new("true", Position::fake()),
                                    Position::fake()
                                ),
                            )],
                            Block::new(
                                vec![],
                                Variable::new("true", Position::fake()),
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
                if_(input("if true {1}else if true {2}else{3}", ""))
                    .unwrap()
                    .1,
                If::new(
                    vec![
                        IfBranch::new(
                            Variable::new("true", Position::fake()),
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
                            Variable::new("true", Position::fake()),
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
                expression(input("if x==y {none}else{none}", "")).unwrap().1,
                If::new(
                    vec![IfBranch::new(
                        BinaryOperation::new(
                            BinaryOperator::Equal,
                            Variable::new("x", Position::fake()),
                            Variable::new("y", Position::fake()),
                            Position::fake()
                        ),
                        Block::new(
                            vec![],
                            Variable::new("none", Position::fake()),
                            Position::fake()
                        ),
                    )],
                    Block::new(
                        vec![],
                        Variable::new("none", Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                )
                .into()
            );
        }

        #[test]
        fn parse_if_type() {
            assert_eq!(
                if_type(input("if x=y as boolean {none}else{none}", ""))
                    .unwrap()
                    .1,
                IfType::new(
                    "x",
                    Variable::new("y", Position::fake()),
                    vec![IfTypeBranch::new(
                        types::Reference::new("boolean", Position::fake()),
                        Block::new(
                            vec![],
                            Variable::new("none", Position::fake()),
                            Position::fake()
                        ),
                    )],
                    Some(Block::new(
                        vec![],
                        Variable::new("none", Position::fake()),
                        Position::fake()
                    )),
                    Position::fake(),
                )
            );

            assert_eq!(
                if_type(input(
                    "if x=y as boolean{none}else if none{none}else{none}",
                    ""
                ))
                .unwrap()
                .1,
                IfType::new(
                    "x",
                    Variable::new("y", Position::fake()),
                    vec![
                        IfTypeBranch::new(
                            types::Reference::new("boolean", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                        ),
                        IfTypeBranch::new(
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                        )
                    ],
                    Some(Block::new(
                        vec![],
                        Variable::new("none", Position::fake()),
                        Position::fake()
                    )),
                    Position::fake()
                )
            );

            assert_eq!(
                if_type(input("if x=y as boolean{none}else if none{none}", ""))
                    .unwrap()
                    .1,
                IfType::new(
                    "x",
                    Variable::new("y", Position::fake()),
                    vec![
                        IfTypeBranch::new(
                            types::Reference::new("boolean", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
                        ),
                        IfTypeBranch::new(
                            types::Reference::new("none", Position::fake()),
                            Block::new(
                                vec![],
                                Variable::new("none", Position::fake()),
                                Position::fake()
                            ),
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
                if_list(input("if[x,...xs]=xs {none}else{none}", ""))
                    .unwrap()
                    .1,
                IfList::new(
                    Variable::new("xs", Position::fake()),
                    "x",
                    "xs",
                    Block::new(
                        vec![],
                        Variable::new("none", Position::fake()),
                        Position::fake()
                    ),
                    Block::new(
                        vec![],
                        Variable::new("none", Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                )
            );
        }

        #[test]
        fn parse_if_map() {
            assert_eq!(
                if_map(input("if x=xs[42]{none}else{none}", "")).unwrap().1,
                IfMap::new(
                    "x",
                    Variable::new("xs", Position::fake()),
                    Number::new(
                        NumberRepresentation::FloatingPoint("42".into()),
                        Position::fake()
                    ),
                    Block::new(
                        vec![],
                        Variable::new("none", Position::fake()),
                        Position::fake()
                    ),
                    Block::new(
                        vec![],
                        Variable::new("none", Position::fake()),
                        Position::fake()
                    ),
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
                    expression(input("f ()", "")).unwrap().1,
                    Variable::new("f", Position::fake()).into()
                );

                assert_eq!(
                    expression(input("f()", "")).unwrap().1,
                    Call::new(
                        Variable::new("f", Position::fake()),
                        vec![],
                        Position::fake()
                    )
                    .into()
                );

                assert_eq!(
                    expression(input("f()()", "")).unwrap().1,
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
                    expression(input("f(1)", "")).unwrap().1,
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
                    expression(input("f(1,)", "")).unwrap().1,
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
                    expression(input("f(1, 2)", "")).unwrap().1,
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
                    expression(input("f(1, 2,)", "")).unwrap().1,
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
                assert!(expression(input("f(1+)", "")).is_err());
            }
        }

        #[test]
        fn parse_try_operation() {
            assert_eq!(
                expression(input("x?", "")).unwrap().1,
                UnaryOperation::new(
                    UnaryOperator::Try,
                    Variable::new("x", Position::fake()),
                    Position::fake()
                )
                .into()
            );
        }

        #[test]
        fn parse_prefix_operation() {
            assert!(prefix_operation(input("", "")).is_err());

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
                                Variable::new("true", Position::fake()),
                                Block::new(
                                    vec![],
                                    Variable::new("true", Position::fake()),
                                    Position::fake(),
                                ),
                            )],
                            Block::new(
                                vec![],
                                Variable::new("true", Position::fake()),
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
                assert_eq!(prefix_operation(input(source, "")).unwrap().1, *expected);
            }
        }

        #[test]
        fn parse_prefix_operator() {
            assert!(prefix_operator(input("", "")).is_err());

            assert_eq!(
                prefix_operator(input("!", "")).unwrap().1,
                UnaryOperator::Not
            );
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
                        Variable::new("true", Position::fake()),
                        Variable::new("true", Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "true|true",
                    BinaryOperation::new(
                        BinaryOperator::Or,
                        Variable::new("true", Position::fake()),
                        Variable::new("true", Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "true&1<2",
                    BinaryOperation::new(
                        BinaryOperator::And,
                        Variable::new("true", Position::fake()),
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
                        Variable::new("true", Position::fake()),
                        BinaryOperation::new(
                            BinaryOperator::And,
                            Variable::new("true", Position::fake()),
                            Variable::new("true", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "true|true&true|true",
                    BinaryOperation::new(
                        BinaryOperator::Or,
                        BinaryOperation::new(
                            BinaryOperator::Or,
                            Variable::new("true", Position::fake()),
                            BinaryOperation::new(
                                BinaryOperator::And,
                                Variable::new("true", Position::fake()),
                                Variable::new("true", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Variable::new("true", Position::fake()),
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
                assert_eq!(expression(input(source, "")).unwrap().1, target);
            }
        }

        #[test]
        fn parse_binary_operator() {
            assert!(binary_operator(input("", "")).is_err());
            assert!(binary_operator(input("+=", "")).is_err());

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
                assert_eq!(binary_operator(input(source, "")).unwrap().1, *expected);
            }
        }

        #[test]
        fn parse_record() {
            assert!(record(input("Foo", "")).is_err());

            assert_eq!(
                record(input("Foo{}", "")).unwrap().1,
                Record::new("Foo", None, vec![], Position::fake())
            );

            assert_eq!(
                expression(input("Foo{foo:42}", "")).unwrap().1,
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
                record(input("Foo{foo:42}", "")).unwrap().1,
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
                record(input("Foo{foo:42,bar:42}", "")).unwrap().1,
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

            assert_eq!(
                expression(input("foo(Foo{foo:42})", "")).unwrap().1,
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
                record(input("Foo{foo:bar(42)}", "")).unwrap().1,
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

            assert!(record(input("Foo{...foo,}", "")).is_err());

            assert_eq!(
                record(input("Foo{...foo,bar:42}", "")).unwrap().1,
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
                record(input("Foo{...foo,bar:42,}", "")).unwrap().1,
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
                expression(input("Foo{...foo,bar:42}", "")).unwrap().1,
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

            assert!(record(input("Foo{...foo}", "")).is_err());
            assert!(record(input("Foo{...(foo),bar:42}", "")).is_ok());
            assert!(record(input("Foo{...foo(bar),bar:42}", "")).is_ok());
            assert!(record(input("Foo{...if true { none } else { none },bar:42}", "")).is_ok());
        }

        #[test]
        fn parse_variable() {
            assert!(variable(input("", "")).is_err());

            assert_eq!(
                variable(input("x", "")).unwrap().1,
                Variable::new("x", Position::fake()),
            );

            assert_eq!(
                variable(input("Foo.x", "")).unwrap().1,
                Variable::new("Foo", Position::fake()),
            );
        }

        #[test]
        fn parse_number_literal() {
            assert!(number_literal(input("", "")).is_err());
            assert!(number_literal(input("foo", "")).is_err());

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
                    number_literal(input(source, "")).unwrap().1,
                    Number::new(value, Position::fake())
                );
            }
        }

        #[test]
        fn parse_string_literal() {
            assert!(string_literal(input("", "")).is_err());
            assert!(string_literal(input("foo", "")).is_err());
            assert!(string_literal(input("\\a", "")).is_err());

            for (source, value) in &[
                (r#""""#, ""),
                (r#""foo""#, "foo"),
                (r#" "foo""#, "foo"),
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
                    string_literal(input(source, "")).unwrap().1,
                    ByteString::new(*value, Position::fake())
                );
            }
        }

        #[test]
        fn parse_list() {
            for (source, target) in vec![
                (
                    "[none]",
                    List::new(
                        types::Reference::new("none", Position::fake()),
                        vec![],
                        Position::fake(),
                    ),
                ),
                (
                    "[none none]",
                    List::new(
                        types::Reference::new("none", Position::fake()),
                        vec![ListElement::Single(
                            Variable::new("none", Position::fake()).into(),
                        )],
                        Position::fake(),
                    ),
                ),
                (
                    "[none none,]",
                    List::new(
                        types::Reference::new("none", Position::fake()),
                        vec![ListElement::Single(
                            Variable::new("none", Position::fake()).into(),
                        )],
                        Position::fake(),
                    ),
                ),
                (
                    "[none none,none]",
                    List::new(
                        types::Reference::new("none", Position::fake()),
                        vec![
                            ListElement::Single(Variable::new("none", Position::fake()).into()),
                            ListElement::Single(Variable::new("none", Position::fake()).into()),
                        ],
                        Position::fake(),
                    ),
                ),
                (
                    "[none none,none,]",
                    List::new(
                        types::Reference::new("none", Position::fake()),
                        vec![
                            ListElement::Single(Variable::new("none", Position::fake()).into()),
                            ListElement::Single(Variable::new("none", Position::fake()).into()),
                        ],
                        Position::fake(),
                    ),
                ),
                (
                    "[none ...foo]",
                    List::new(
                        types::Reference::new("none", Position::fake()),
                        vec![ListElement::Multiple(
                            Variable::new("foo", Position::fake()).into(),
                        )],
                        Position::fake(),
                    ),
                ),
                (
                    "[none ...foo,]",
                    List::new(
                        types::Reference::new("none", Position::fake()),
                        vec![ListElement::Multiple(
                            Variable::new("foo", Position::fake()).into(),
                        )],
                        Position::fake(),
                    ),
                ),
                (
                    "[none ...foo,...bar]",
                    List::new(
                        types::Reference::new("none", Position::fake()),
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
                        types::Reference::new("none", Position::fake()),
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
                        types::Reference::new("none", Position::fake()),
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
                        types::Reference::new("none", Position::fake()),
                        vec![
                            ListElement::Multiple(Variable::new("foo", Position::fake()).into()),
                            ListElement::Single(Variable::new("bar", Position::fake()).into()),
                        ],
                        Position::fake(),
                    ),
                ),
            ] {
                assert_eq!(expression(input(source, "")).unwrap().1, target.into());
            }
        }

        #[test]
        fn parse_list_comprehension() {
            for (source, target) in vec![
                (
                    "[none x for x in xs]",
                    ListComprehension::new(
                        types::Reference::new("none", Position::fake()),
                        Variable::new("x", Position::fake()),
                        vec![ListComprehensionBranch::new(
                            "x",
                            None,
                            Variable::new("xs", Position::fake()),
                            Position::fake(),
                        )],
                        Position::fake(),
                    ),
                ),
                (
                    "[number x + 42 for x in xs]",
                    ListComprehension::new(
                        types::Reference::new("number", Position::fake()),
                        BinaryOperation::new(
                            BinaryOperator::Add,
                            Variable::new("x", Position::fake()),
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        vec![ListComprehensionBranch::new(
                            "x",
                            None,
                            Variable::new("xs", Position::fake()),
                            Position::fake(),
                        )],
                        Position::fake(),
                    ),
                ),
                (
                    "[number x + 42 for y in x for x in xs]",
                    ListComprehension::new(
                        types::Reference::new("number", Position::fake()),
                        BinaryOperation::new(
                            BinaryOperator::Add,
                            Variable::new("x", Position::fake()),
                            Number::new(
                                NumberRepresentation::FloatingPoint("42".into()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        vec![
                            ListComprehensionBranch::new(
                                "y",
                                None,
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            ListComprehensionBranch::new(
                                "x",
                                None,
                                Variable::new("xs", Position::fake()),
                                Position::fake(),
                            ),
                        ],
                        Position::fake(),
                    ),
                ),
            ] {
                assert_eq!(
                    list_comprehension(input(source, "")).unwrap().1,
                    target.into()
                );
            }
        }

        #[test]
        fn parse_map() {
            for (source, target) in vec![
                (
                    "{none:none}",
                    Map::new(
                        types::Reference::new("none", Position::fake()),
                        types::Reference::new("none", Position::fake()),
                        vec![],
                        Position::fake(),
                    )
                    .into(),
                ),
                (
                    "{none:none none:none}",
                    Map::new(
                        types::Reference::new("none", Position::fake()),
                        types::Reference::new("none", Position::fake()),
                        vec![MapEntry::new(
                            Variable::new("none", Position::fake()),
                            Variable::new("none", Position::fake()),
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
                        types::Reference::new("number", Position::fake()),
                        types::Reference::new("none", Position::fake()),
                        vec![
                            MapEntry::new(
                                Number::new(
                                    NumberRepresentation::FloatingPoint("1".into()),
                                    Position::fake(),
                                ),
                                Variable::new("none", Position::fake()),
                                Position::fake(),
                            )
                            .into(),
                            MapEntry::new(
                                Number::new(
                                    NumberRepresentation::FloatingPoint("2".into()),
                                    Position::fake(),
                                ),
                                Variable::new("none", Position::fake()),
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
                        types::Reference::new("none", Position::fake()),
                        types::Reference::new("none", Position::fake()),
                        vec![MapElement::Map(
                            Variable::new("none", Position::fake()).into(),
                        )],
                        Position::fake(),
                    )
                    .into(),
                ),
            ] {
                assert_eq!(expression(input(source, "")).unwrap().1, target);
            }
        }

        #[test]
        fn parse_map_iteration_comprehension() {
            assert_eq!(
                list_comprehension(input("[none v for k, v in xs]", ""))
                    .unwrap()
                    .1,
                ListComprehension::new(
                    types::Reference::new("none", Position::fake()),
                    Variable::new("v", Position::fake()),
                    vec![ListComprehensionBranch::new(
                        "k",
                        Some("v".into()),
                        Variable::new("xs", Position::fake()),
                        Position::fake(),
                    )],
                    Position::fake(),
                )
                .into()
            );
        }
    }

    #[test]
    fn parse_identifier() {
        assert!(identifier(input("if", "")).is_err());
        assert!(identifier(input("1foo", "")).is_err());
        assert_eq!(identifier(input("foo", "")).unwrap().1, "foo".to_string());
        assert_eq!(
            identifier(input("foo42", "")).unwrap().1,
            "foo42".to_string()
        );
    }

    #[test]
    fn parse_keyword() {
        assert!(keyword("type").parse(input("bar", "")).is_err());
        // spell-checker: disable-next-line
        assert!(keyword("type").parse(input("typer", "")).is_err());
        assert!(keyword("type").parse(input("type_", "")).is_err());
        assert!(keyword("type").parse(input("type", "")).is_ok());
    }

    #[test]
    fn parse_sign() {
        assert!(sign("+")(input("", "")).is_err());
        assert!(sign("+")(input("-", "")).is_err());
        assert!(sign("+")(input("+", "")).is_ok());
        assert!(sign("++")(input("++", "")).is_ok());
        assert!(sign("+")(input("++", "")).is_ok());
        assert!(sign("+")(input("+=", "")).is_err());
        assert!(sign("\\")(input("\\", "")).is_ok());
    }

    #[test]
    fn parse_blank() {
        assert!(all_consuming(blank)(input(" ", "")).is_ok());
        assert!(all_consuming(blank)(input("\n", "")).is_ok());
        assert!(all_consuming(blank)(input(" \n", "")).is_ok());
        assert!(all_consuming(blank)(input("\t", "")).is_ok());
        assert!(all_consuming(blank)(input("# foo", "")).is_ok());
    }

    #[test]
    fn parse_comment() {
        assert!(comment(input("#", "")).is_ok());
        assert!(comment(input("#\n", "")).is_ok());
        assert!(comment(input("#x\n", "")).is_ok());
    }

    #[test]
    fn parse_skipped_comment() {
        assert!(skipped_comment(input("#", "")).is_ok());
        assert!(skipped_comment(input("#\n", "")).is_ok());
        assert!(skipped_comment(input("#x\n", "")).is_ok());
    }

    mod comments {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parse_comment() {
            assert_eq!(
                comments(input("#foo", "")).unwrap().1,
                vec![Comment::new("foo", Position::fake())]
            );
        }

        #[test]
        fn parse_comment_after_space() {
            assert_eq!(
                comments(input(" #foo", "")).unwrap().1,
                vec![Comment::new("foo", Position::fake())]
            );
        }

        #[test]
        fn parse_comment_before_space() {
            assert_eq!(
                comments(input("#foo\n #bar", "")).unwrap().1,
                vec![
                    Comment::new("foo", Position::fake()),
                    Comment::new("bar", Position::fake())
                ]
            );
        }

        #[test]
        fn parse_comment_before_newlines() {
            assert_eq!(
                comments(input("#foo\n\n", "")).unwrap().1,
                vec![Comment::new("foo", Position::fake())]
            );
        }

        #[test]
        fn parse_two_line_comments() {
            assert_eq!(
                comments(input(
                    indoc!(
                        "
                            #foo
                            #bar
                            "
                    ),
                    ""
                ))
                .unwrap()
                .1,
                vec![
                    Comment::new("foo", Position::fake()),
                    Comment::new("bar", Position::fake())
                ]
            );
        }

        #[test]
        fn parse_comment_after_identifier() {
            assert_eq!(
                comments(input("foo#foo", "")).unwrap().1,
                vec![Comment::new("foo", Position::fake())]
            );
        }

        #[test]
        fn parse_comment_before_identifier() {
            assert_eq!(
                comments(input("#foo\nfoo#bar", "")).unwrap().1,
                vec![
                    Comment::new("foo", Position::fake()),
                    Comment::new("bar", Position::fake())
                ]
            );
        }

        #[test]
        fn parse_comment_after_keyword() {
            assert_eq!(
                comments(input("if#foo", "")).unwrap().1,
                vec![Comment::new("foo", Position::fake())]
            );
        }

        #[test]
        fn parse_comment_before_keyword() {
            assert_eq!(
                comments(input("#foo\nif#bar", "")).unwrap().1,
                vec![
                    Comment::new("foo", Position::fake()),
                    Comment::new("bar", Position::fake())
                ]
            );
        }

        #[test]
        fn parse_comment_after_sign() {
            assert_eq!(
                comments(input("+#foo", "")).unwrap().1,
                vec![Comment::new("foo", Position::fake())]
            );
        }

        #[test]
        fn parse_comment_before_sign() {
            assert_eq!(
                comments(input("#foo\n+#bar", "")).unwrap().1,
                vec![
                    Comment::new("foo", Position::fake()),
                    Comment::new("bar", Position::fake())
                ]
            );
        }

        #[test]
        fn parse_comment_after_string() {
            assert_eq!(
                comments(input("\"string\"#foo", "")).unwrap().1,
                vec![Comment::new("foo", Position::fake())]
            );
        }

        #[test]
        fn parse_comment_before_string() {
            assert_eq!(
                comments(input("#foo\n\"string\"#bar", "")).unwrap().1,
                vec![
                    Comment::new("foo", Position::fake()),
                    Comment::new("bar", Position::fake())
                ]
            );
        }
    }
}
