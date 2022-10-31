use ast::{types::Type, *};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1, alphanumeric0, alphanumeric1, char, digit1, hex_digit1, multispace0, multispace1,
        none_of, one_of,
    },
    combinator::{all_consuming, into, map, not, opt, peek, recognize, success, value, verify},
    error::ParseError,
    multi::{many0, many1, separated_list0, separated_list1},
    number::complete::recognize_float,
    sequence::{delimited, preceded, terminated, tuple},
    IResult, InputLength, Parser,
};
use nom_locate::LocatedSpan;
use position::Position;

use crate::{
    combinator::{separated_or_terminated_list0, separated_or_terminated_list1},
    operations::{reduce_operations, SuffixOperator},
};

const KEYWORDS: &[&str] = &[
    "as", "else", "export", "for", "foreign", "if", "in", "import", "type",
];
const OPERATOR_CHARACTERS: &str = "+-*/=<>&|!?";
const OPERATOR_MODIFIERS: &str = "=";

type Input<'a> = LocatedSpan<&'a str, &'a str>;

fn input<'a>(source: &'a str, path: &'a str) -> Input<'a> {
    LocatedSpan::new_extra(source, path)
}

pub fn comments(input: Input) -> IResult<Input, Vec<Comment>> {
    let (input, comments) = all_consuming(many0(tuple((
        multispace0,
        alt((
            map(comment, Some),
            map(raw_string_literal, |_| None),
            map(none_of("\"#"), |_| None),
        )),
        multispace0,
    ))))(input)?;

    Ok((
        input,
        comments
            .into_iter()
            .flat_map(|(_, comment, _)| comment)
            .collect(),
    ))
}

fn type_(input: Input) -> IResult<Input, Type> {
    alt((into(function_type), union_type))(input)
}

fn function_type(input: Input) -> IResult<Input, types::Function> {
    let position = position(input);

    let (input, (_, arguments, _, result)) = tuple((
        sign("\\("),
        separated_or_terminated_list0(sign(","), type_),
        sign(")"),
        type_,
    ))(input)?;

    Ok((input, types::Function::new(arguments, result, position)))
}

fn union_type(input: Input) -> IResult<Input, Type> {
    let (input, types) = separated_list1(sign("|"), atomic_type)(input)?;

    Ok((
        input,
        types
            .into_iter()
            .reduce(|lhs, rhs| types::Union::new(lhs.clone(), rhs, lhs.position().clone()).into())
            .unwrap(),
    ))
}

fn list_type(input: Input) -> IResult<Input, types::List> {
    let position = position(input);

    let (input, element) = delimited(sign("["), type_, sign("]"))(input)?;

    Ok((input, types::List::new(element, position)))
}

fn map_type(input: Input) -> IResult<Input, types::Map> {
    let position = position(input);

    let (input, (_, key, _, value, _)) =
        tuple((sign("{"), type_, sign(":"), type_, sign("}")))(input)?;

    Ok((input, types::Map::new(key, value, position)))
}

fn atomic_type(input: Input) -> IResult<Input, Type> {
    alt((
        into(reference_type),
        into(list_type),
        into(map_type),
        delimited(sign("("), type_, sign(")")),
    ))(input)
}

fn reference_type(input: Input) -> IResult<Input, types::Reference> {
    let position = position(input);
    let (input, identifier) = token(qualified_identifier)(input)?;

    Ok((input, types::Reference::new(identifier, position)))
}

fn block(input: Input) -> IResult<Input, Block> {
    let (input, (position, statements)) = tuple((
        position_parser,
        delimited(sign("{"), many1(statement), sign("}")),
    ))(input)?;

    // TODO Validate the last expressions.
    Ok((
        input,
        Block::new(
            statements[..statements.len() - 1].to_vec(),
            statements.last().unwrap().expression().clone(),
            position,
        ),
    ))
}

fn statement(input: Input) -> IResult<Input, Statement> {
    let (input, (position, name, expression)) = tuple((
        position_parser,
        opt(terminated(identifier, sign("="))),
        expression,
    ))(input)?;

    Ok((input, Statement::new(name, expression, position)))
}

fn expression(input: Input) -> IResult<Input, Expression> {
    binary_operation_like(input)
}

fn binary_operation_like(input: Input) -> IResult<Input, Expression> {
    let (input, (expression, pairs)): (_, (_, Vec<_>)) = tuple((
        prefix_operation_like,
        many0(map(
            tuple((position_parser, binary_operator, prefix_operation_like)),
            |(position, operator, expression)| (operator, expression, position),
        )),
    ))(input)?;

    Ok((input, reduce_operations(expression, &pairs)))
}

fn binary_operator(input: Input) -> IResult<Input, BinaryOperator> {
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
    ))(input)
}

fn prefix_operation_like(input: Input) -> IResult<Input, Expression> {
    alt((into(prefix_operation), into(suffix_operation_like)))(input)
}

fn prefix_operation(input: Input) -> IResult<Input, UnaryOperation> {
    let (input, (position, operator, expression)) =
        tuple((position_parser, prefix_operator, prefix_operation_like))(input)?;

    Ok((input, UnaryOperation::new(operator, expression, position)))
}

fn prefix_operator(input: Input) -> IResult<Input, UnaryOperator> {
    value(UnaryOperator::Not, sign("!"))(input)
}

fn suffix_operation_like(input: Input) -> IResult<Input, Expression> {
    let (input, (expression, suffix_operators)) =
        tuple((atomic_expression, many0(suffix_operator)))(input)?;

    Ok((
        input,
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
            }),
    ))
}

fn suffix_operator(input: Input) -> IResult<Input, SuffixOperator> {
    alt((call_operator, record_field_operator, try_operator))(input)
}

fn call_operator(input: Input) -> IResult<Input, SuffixOperator> {
    let position = position(input);

    let (input, arguments) = delimited(
        sign("("),
        separated_or_terminated_list0(sign(","), expression),
        sign(")"),
    )(input)?;

    Ok((input, SuffixOperator::Call(arguments, position)))
}

fn record_field_operator(input: Input) -> IResult<Input, SuffixOperator> {
    let position = position(input);

    let (input, identifier) = preceded(sign("."), identifier)(input)?;

    Ok((input, SuffixOperator::RecordField(identifier, position)))
}

fn try_operator(input: Input) -> IResult<Input, SuffixOperator> {
    let position = position(input);

    let (input, _) = sign("?")(input)?;

    Ok((input, SuffixOperator::Try(position)))
}

fn atomic_expression(input: Input) -> IResult<Input, Expression> {
    alt((
        // if_list().map(Expression::from),
        // if_map().map(Expression::from),
        // if_type().map(Expression::from),
        into(if_),
        // lambda().map(Expression::from),
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

fn if_(input: Input) -> IResult<Input, If> {
    let (input, (position, _, first_branch, branches, _, else_block)) = tuple((
        position_parser,
        keyword("if"),
        if_branch,
        many0(preceded(tuple((keyword("else"), keyword("if"))), if_branch)),
        keyword("else"),
        block,
    ))(input)?;

    Ok((
        input,
        If::new(
            [first_branch].into_iter().chain(branches).collect(),
            else_block,
            position,
        ),
    ))
}

fn if_branch(input: Input) -> IResult<Input, IfBranch> {
    let (input, (expression, block)) = tuple((expression, block))(input)?;

    Ok((input, IfBranch::new(expression, block)))
}

fn record(input: Input) -> IResult<Input, Record> {
    let (input, (position, name, _, (record, fields), _)) = tuple((
        position_parser,
        qualified_identifier,
        sign("{"),
        alt((
            tuple((
                map(delimited(sign("..."), expression, sign(",")), Some),
                separated_or_terminated_list1(sign(","), record_field),
            )),
            tuple((
                success(None),
                separated_or_terminated_list0(sign(","), record_field),
            )),
        )),
        sign("}"),
    ))(input)?;

    // TODO Validate duplicate fields.
    Ok((input, Record::new(name, record, fields, position)))
}

fn record_field(input: Input) -> IResult<Input, RecordField> {
    let (input, (position, name, _, expression)) =
        tuple((position_parser, identifier, sign(":"), expression))(input)?;

    Ok((input, RecordField::new(name, expression, position)))
}

fn number_literal(input: Input) -> IResult<Input, Number> {
    let position = position(input);
    let (input, number) = token(terminated(
        alt((binary_literal, hexadecimal_literal, decimal_literal)),
        peek(not(digit1)),
    ))(input)?;

    Ok((input, Number::new(number, position)))
}

fn binary_literal(input: Input) -> IResult<Input, NumberRepresentation> {
    let (input, characters) = preceded(tag("0b"), many1(one_of("01")))(input)?;

    Ok((
        input,
        NumberRepresentation::Binary(String::from_iter(characters)),
    ))
}

fn hexadecimal_literal(input: Input) -> IResult<Input, NumberRepresentation> {
    let (input, characters) = preceded(tag("0x"), many1(one_of("0123456789abcdefABCDEF")))(input)?;

    Ok((
        input,
        NumberRepresentation::Hexadecimal(String::from_iter(characters).to_lowercase()),
    ))
}

fn decimal_literal(input: Input) -> IResult<Input, NumberRepresentation> {
    let (input, characters) = recognize_float(input)?;

    Ok((
        input,
        NumberRepresentation::FloatingPoint(
            String::from_utf8_lossy(characters.as_bytes()).to_string(),
        ),
    ))
}

fn string_literal(input: Input) -> IResult<Input, ByteString> {
    token(raw_string_literal)(input)
}

fn raw_string_literal(input: Input) -> IResult<Input, ByteString> {
    let position = position(input);

    let (input, strings) = delimited(
        char('"'),
        many0(alt((
            recognize(none_of("\\\"")),
            tag("\\\\"),
            tag("\\\""),
            tag("\\n"),
            tag("\\r"),
            tag("\\t"),
            // TODO Limit a number of digits.
            recognize(tuple((tag("\\x"), hex_digit1))),
        ))),
        char('"'),
    )(input)?;

    Ok((
        input,
        ByteString::new(
            strings
                .iter()
                .map(|span| String::from_utf8_lossy(span.as_bytes()))
                .collect::<Vec<_>>()
                .concat(),
            position,
        ),
    ))
}

fn list_literal(input: Input) -> IResult<Input, List> {
    let (input, position) = position_parser(input)?;
    let (input, (_, type_, elements, _)) = tuple((
        sign("["),
        type_,
        separated_or_terminated_list0(sign(","), list_element),
        sign("]"),
    ))(input)?;

    Ok((input, List::new(type_, elements, position)))
}

fn list_element(input: Input) -> IResult<Input, ListElement> {
    alt((
        map(expression, ListElement::Single),
        map(preceded(sign("..."), expression), ListElement::Multiple),
    ))(input)
}

fn list_comprehension(input: Input) -> IResult<Input, Expression> {
    let (input, (position, _, type_, element, _, element_name, value_name, _, iterator, _)) =
        tuple((
            position_parser,
            sign("["),
            type_,
            expression,
            keyword("for"),
            identifier,
            opt(preceded(sign(","), identifier)),
            keyword("in"),
            expression,
            sign("]"),
        ))(input)?;

    Ok((
        input,
        if let Some(value_name) = value_name {
            MapIterationComprehension::new(
                type_,
                element,
                element_name,
                value_name,
                iterator,
                position,
            )
            .into()
        } else {
            ListComprehension::new(type_, element, element_name, iterator, position).into()
        },
    ))
}

fn map_literal(input: Input) -> IResult<Input, Map> {
    let position = position(input);

    let (input, (_, key_type, _, value_type, elements, _)) = tuple((
        sign("{"),
        type_,
        sign(":"),
        type_,
        separated_or_terminated_list0(sign(","), map_element),
        sign("}"),
    ))(input)?;

    Ok((input, Map::new(key_type, value_type, elements, position)))
}

fn map_element(input: Input) -> IResult<Input, MapElement> {
    alt((
        map(
            tuple((position_parser, expression, sign(":"), expression)),
            |(position, key, _, value)| MapEntry::new(key, value, position).into(),
        ),
        map(preceded(sign("..."), expression), MapElement::Map),
        map(expression, MapElement::Removal),
    ))(input)
}

fn variable(input: Input) -> IResult<Input, Variable> {
    let position = position(input);

    let (input, identifier) = token(qualified_identifier)(input)?;

    Ok((input, Variable::new(identifier, position)))
}

fn qualified_identifier(input: Input) -> IResult<Input, String> {
    let (input, (former, latter)) = tuple((
        raw_identifier,
        opt(tuple((tag(IDENTIFIER_SEPARATOR), raw_identifier))),
    ))(input)?;

    Ok((
        input,
        if let Some((_, latter)) = latter {
            [&former, IDENTIFIER_SEPARATOR, &latter].concat()
        } else {
            former
        },
    ))
}

fn identifier(input: Input) -> IResult<Input, String> {
    token(raw_identifier)(input)
}

fn raw_identifier(input: Input) -> IResult<Input, String> {
    verify(unchecked_identifier, |identifier: &str| {
        !KEYWORDS.contains(&identifier)
    })(input)
}

fn unchecked_identifier(input: Input) -> IResult<Input, String> {
    let (input, span) = recognize(tuple((
        alt((value((), alpha1), value((), char('_')))),
        many0(alt((value((), alphanumeric1), value((), char('_'))))),
    )))(input)?;

    Ok((input, String::from_utf8_lossy(span.as_bytes()).to_string()))
}

fn keyword(name: &'static str) -> impl FnMut(Input) -> IResult<Input, ()> {
    if !KEYWORDS.contains(&name) {
        unreachable!("undefined keyword");
    }

    move |input| {
        let (input, _) = value(
            (),
            token(tuple((
                tag(name),
                peek(not(alt((value((), alphanumeric1), value((), char('_')))))),
            ))),
        )(input)?;

        Ok((input, ()))
    }
}

fn sign(sign: &'static str) -> impl Fn(Input) -> IResult<Input, ()> + Clone {
    move |input| {
        let parser = token(tag(sign));

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
    mut parser: impl Parser<Input<'a>, O, nom::error::Error<Input<'a>>>,
) -> impl FnMut(Input<'a>) -> IResult<Input, O, nom::error::Error<Input<'a>>> {
    move |input| {
        let (input, _) = blank(input)?;

        parser.parse(input)
    }
}

fn blank(input: Input) -> IResult<Input, ()> {
    value((), many0(alt((value((), multispace1), value((), comment)))))(input)
}

fn comment(input: Input) -> IResult<Input, Comment> {
    let position = position(input);

    let (input, _) = tag("#")(input)?;
    let (input, comment) = many0(none_of("\n\r"))(input)?;

    Ok((input, Comment::new(String::from_iter(comment), position)))
}

// TODO Replace use of this with `position_parser`.
fn position(input: Input) -> Position {
    Position::new(
        input.extra,
        input.location_line() as usize,
        input.get_column(),
        String::from_utf8_lossy(input.get_line_beginning()),
    )
}

fn position_parser(input: Input) -> IResult<Input, Position> {
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        Position::new(
            input.extra,
            input.location_line() as usize,
            input.get_column(),
            String::from_utf8_lossy(input.get_line_beginning()),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::ParseError, stream::stream};
    use indoc::indoc;
    use position::test::PositionFake;
    use pretty_assertions::assert_eq;

    // mod module {
    //     use super::*;
    //     use pretty_assertions::assert_eq;

    //     #[test]
    //     fn parse_module() {
    //         assert_eq!(
    //             module().parse(input("", "")).unwrap().0,
    //             Module::new(vec![], vec![], vec![], vec![], Position::fake())
    //         );
    //         assert_eq!(
    //             module().parse(input(" ", "")).unwrap().0,
    //             Module::new(vec![], vec![], vec![], vec![], Position::fake())
    //         );
    //         assert_eq!(
    //             module().parse(input("\n", "")).unwrap().0,
    //             Module::new(vec![], vec![], vec![], vec![], Position::fake())
    //         );
    //         assert_eq!(
    //             module().parse(input("import Foo'Bar", "")).unwrap().0,
    //             Module::new(
    //                 vec![Import::new(
    //                     ExternalModulePath::new("Foo", vec!["Bar".into()]),
    //                     None,
    //                     vec![],
    //                     Position::fake()
    //                 )],
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 Position::fake()
    //             )
    //         );
    //         assert_eq!(
    //             module().parse(input("type foo = number", "")).unwrap().0,
    //             Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![TypeAlias::new(
    //                     "foo",
    //                     types::Reference::new("number", Position::fake()),
    //                     Position::fake()
    //                 )
    //                 .into()],
    //                 vec![],
    //                 Position::fake()
    //             )
    //         );
    //         assert_eq!(
    //             module()
    //                 .parse(input("x=\\(x number)number{42}", ""))
    //                 .unwrap()
    //                 .0,
    //             Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 vec![FunctionDefinition::new(
    //                     "x",
    //                     Lambda::new(
    //                         vec![Argument::new(
    //                             "x",
    //                             types::Reference::new("number", Position::fake())
    //                         )],
    //                         types::Reference::new("number", Position::fake()),
    //                         Block::new(
    //                             vec![],
    //                             Number::new(
    //                                 NumberRepresentation::FloatingPoint("42".into()),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     ),
    //                     None,
    //                     Position::fake()
    //                 )],
    //                 Position::fake()
    //             )
    //         );
    //         assert_eq!(
    //             module()
    //                 .parse(input(
    //                     "x=\\(x number)number{42}y=\\(y number)number{42}",
    //                     ""
    //                 ))
    //                 .unwrap()
    //                 .0,
    //             Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![],
    //                 vec![
    //                     FunctionDefinition::new(
    //                         "x",
    //                         Lambda::new(
    //                             vec![Argument::new(
    //                                 "x",
    //                                 types::Reference::new("number", Position::fake())
    //                             )],
    //                             types::Reference::new("number", Position::fake()),
    //                             Block::new(
    //                                 vec![],
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("42".into()),
    //                                     Position::fake()
    //                                 ),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake()
    //                         ),
    //                         None,
    //                         Position::fake()
    //                     ),
    //                     FunctionDefinition::new(
    //                         "y",
    //                         Lambda::new(
    //                             vec![Argument::new(
    //                                 "y",
    //                                 types::Reference::new("number", Position::fake())
    //                             )],
    //                             types::Reference::new("number", Position::fake()),
    //                             Block::new(
    //                                 vec![],
    //                                 Number::new(
    //                                     NumberRepresentation::FloatingPoint("42".into()),
    //                                     Position::fake()
    //                                 ),
    //                                 Position::fake()
    //                             ),
    //                             Position::fake()
    //                         ),
    //                         None,
    //                         Position::fake()
    //                     )
    //                 ],
    //                 Position::fake()
    //             )
    //         );
    //     }

    //     #[test]
    //     fn parse_import_foreign_after_import() {
    //         assert_eq!(
    //             module()
    //                 .parse(input("import Foo'Bar import foreign foo \\() number", ""))
    //                 .unwrap()
    //                 .0,
    //             Module::new(
    //                 vec![Import::new(
    //                     ExternalModulePath::new("Foo", vec!["Bar".into()]),
    //                     None,
    //                     vec![],
    //                     Position::fake()
    //                 )],
    //                 vec![ForeignImport::new(
    //                     "foo",
    //                     CallingConvention::Native,
    //                     types::Function::new(
    //                         vec![],
    //                         types::Reference::new("number", Position::fake()),
    //                         Position::fake()
    //                     ),
    //                     Position::fake()
    //                 )],
    //                 vec![],
    //                 vec![],
    //                 Position::fake()
    //             )
    //         );
    //     }

    //     #[test]
    //     fn parse_record_definition_after_type_alias() {
    //         assert_eq!(
    //             module()
    //                 .parse(input("type foo = number type bar {}", ""))
    //                 .unwrap()
    //                 .0,
    //             Module::new(
    //                 vec![],
    //                 vec![],
    //                 vec![
    //                     TypeAlias::new(
    //                         "foo",
    //                         types::Reference::new("number", Position::fake()),
    //                         Position::fake()
    //                     )
    //                     .into(),
    //                     RecordDefinition::new("bar", vec![], Position::fake()).into(),
    //                 ],
    //                 vec![],
    //                 Position::fake()
    //             )
    //         );
    //     }
    // }

    // mod import {
    //     use super::*;
    //     use pretty_assertions::assert_eq;

    //     #[test]
    //     fn parse_import() {
    //         assert_eq!(
    //             import().parse(input("import 'Foo", "")).unwrap().0,
    //             Import::new(
    //                 InternalModulePath::new(vec!["Foo".into()]),
    //                 None,
    //                 vec![],
    //                 Position::fake()
    //             ),
    //         );
    //         assert_eq!(
    //             import().parse(input("import Foo'Bar", "")).unwrap().0,
    //             Import::new(
    //                 ExternalModulePath::new("Foo", vec!["Bar".into()]),
    //                 None,
    //                 vec![],
    //                 Position::fake()
    //             ),
    //         );
    //     }

    //     #[test]
    //     fn parse_import_with_custom_prefix() {
    //         assert_eq!(
    //             import().parse(input("import 'Foo as foo", "")).unwrap().0,
    //             Import::new(
    //                 InternalModulePath::new(vec!["Foo".into()]),
    //                 Some("foo".into()),
    //                 vec![],
    //                 Position::fake()
    //             ),
    //         );
    //     }

    //     #[test]
    //     fn parse_unqualified_import() {
    //         assert_eq!(
    //             import().parse(input("import 'Foo { Foo }", "")).unwrap().0,
    //             Import::new(
    //                 InternalModulePath::new(vec!["Foo".into()]),
    //                 None,
    //                 vec![UnqualifiedName::new("Foo", Position::fake())],
    //                 Position::fake()
    //             ),
    //         );
    //     }

    //     #[test]
    //     fn parse_unqualified_import_with_multiple_identifiers() {
    //         assert_eq!(
    //             import()
    //                 .parse(input("import 'Foo { Foo, Bar }", ""))
    //                 .unwrap()
    //                 .0,
    //             Import::new(
    //                 InternalModulePath::new(vec!["Foo".into()]),
    //                 None,
    //                 vec![
    //                     UnqualifiedName::new("Foo", Position::fake()),
    //                     UnqualifiedName::new("Bar", Position::fake())
    //                 ],
    //                 Position::fake()
    //             ),
    //         );
    //     }

    //     #[test]
    //     fn parse_module_path() {
    //         assert!(module_path().parse(input("", "")).is_err());
    //         assert_eq!(
    //             module_path().parse(input("'Foo", "")).unwrap().0,
    //             InternalModulePath::new(vec!["Foo".into()]).into(),
    //         );
    //         assert_eq!(
    //             module_path().parse(input("Foo'Bar", "")).unwrap().0,
    //             ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
    //         );
    //     }

    //     #[test]
    //     fn parse_internal_module_path() {
    //         assert!(internal_module_path().parse(input("", "")).is_err());
    //         assert_eq!(
    //             internal_module_path().parse(input("'Foo", "")).unwrap().0,
    //             InternalModulePath::new(vec!["Foo".into()]),
    //         );
    //         assert_eq!(
    //             internal_module_path()
    //                 .parse(input("'Foo'Bar", ""))
    //                 .unwrap()
    //                 .0,
    //             InternalModulePath::new(vec!["Foo".into(), "Bar".into()]),
    //         );
    //     }

    //     #[test]
    //     fn parse_external_module_path() {
    //         assert!(external_module_path().parse(input("", "")).is_err());
    //         assert_eq!(
    //             external_module_path()
    //                 .parse(input("Foo'Bar", ""))
    //                 .unwrap()
    //                 .0,
    //             ExternalModulePath::new("Foo", vec!["Bar".into()]),
    //         );
    //     }

    //     #[test]
    //     fn fail_to_parse_private_external_module_file() {
    //         let source = "Foo'bar";

    //         insta::assert_debug_snapshot!(external_module_path()
    //             .parse(input(source, ""))
    //             .map_err(|error| ParseError::new(source, "", error))
    //             .err());
    //     }

    //     #[test]
    //     fn fail_to_parse_private_external_module_directory() {
    //         let source = "Foo'bar'Baz";

    //         insta::assert_debug_snapshot!(external_module_path()
    //             .parse(input(source, ""))
    //             .map_err(|error| ParseError::new(source, "", error))
    //             .err());
    //     }
    // }

    // #[test]
    // fn parse_foreign_import() {
    //     assert_eq!(
    //         foreign_import()
    //             .parse(input("import foreign foo \\(number) number", ""))
    //             .unwrap()
    //             .0,
    //         ForeignImport::new(
    //             "foo",
    //             CallingConvention::Native,
    //             types::Function::new(
    //                 vec![types::Reference::new("number", Position::fake()).into()],
    //                 types::Reference::new("number", Position::fake()),
    //                 Position::fake()
    //             ),
    //             Position::fake()
    //         ),
    //     );

    //     assert_eq!(
    //         foreign_import()
    //             .parse(input("import foreign \"c\" foo \\(number) number", ""))
    //             .unwrap()
    //             .0,
    //         ForeignImport::new(
    //             "foo",
    //             CallingConvention::C,
    //             types::Function::new(
    //                 vec![types::Reference::new("number", Position::fake()).into()],
    //                 types::Reference::new("number", Position::fake()),
    //                 Position::fake()
    //             ),
    //             Position::fake()
    //         ),
    //     );
    // }

    // mod definition {
    //     use super::*;
    //     use pretty_assertions::assert_eq;

    //     #[test]
    //     fn parse() {
    //         assert_eq!(
    //             definition()
    //                 .parse(input("x=\\(x number)number{42}", ""))
    //                 .unwrap()
    //                 .0,
    //             FunctionDefinition::new(
    //                 "x",
    //                 Lambda::new(
    //                     vec![Argument::new(
    //                         "x",
    //                         types::Reference::new("number", Position::fake())
    //                     )],
    //                     types::Reference::new("number", Position::fake()),
    //                     Block::new(
    //                         vec![],
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("42".into()),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     ),
    //                     Position::fake()
    //                 ),
    //                 None,
    //                 Position::fake()
    //             ),
    //         );
    //     }

    //     #[test]
    //     fn parse_foreign_definition() {
    //         assert_eq!(
    //             definition()
    //                 .parse(input("foreign x=\\(x number)number{42}", ""))
    //                 .unwrap()
    //                 .0,
    //             FunctionDefinition::new(
    //                 "x",
    //                 Lambda::new(
    //                     vec![Argument::new(
    //                         "x",
    //                         types::Reference::new("number", Position::fake())
    //                     )],
    //                     types::Reference::new("number", Position::fake()),
    //                     Block::new(
    //                         vec![],
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("42".into()),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     ),
    //                     Position::fake()
    //                 ),
    //                 ForeignExport::new(CallingConvention::Native).into(),
    //                 Position::fake()
    //             ),
    //         );
    //     }

    //     #[test]
    //     fn parse_foreign_definition_with_c_calling_convention() {
    //         assert_eq!(
    //             definition()
    //                 .parse(input("foreign \"c\" x=\\(x number)number{42}", ""))
    //                 .unwrap()
    //                 .0,
    //             FunctionDefinition::new(
    //                 "x",
    //                 Lambda::new(
    //                     vec![Argument::new(
    //                         "x",
    //                         types::Reference::new("number", Position::fake())
    //                     )],
    //                     types::Reference::new("number", Position::fake()),
    //                     Block::new(
    //                         vec![],
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("42".into()),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     ),
    //                     Position::fake()
    //                 ),
    //                 ForeignExport::new(CallingConvention::C).into(),
    //                 Position::fake()
    //             ),
    //         );
    //     }

    //     #[test]
    //     fn parse_keyword_like_name() {
    //         assert_eq!(
    //             definition()
    //                 .parse(input("importA = \\() number { 42 }", ""))
    //                 .unwrap()
    //                 .0,
    //             FunctionDefinition::new(
    //                 "importA",
    //                 Lambda::new(
    //                     vec![],
    //                     types::Reference::new("number", Position::fake()),
    //                     Block::new(
    //                         vec![],
    //                         Number::new(
    //                             NumberRepresentation::FloatingPoint("42".into()),
    //                             Position::fake()
    //                         ),
    //                         Position::fake()
    //                     ),
    //                     Position::fake()
    //                 ),
    //                 None,
    //                 Position::fake()
    //             ),
    //         );
    //     }
    // }

    // #[test]
    // fn parse_record_definition() {
    //     for (source, expected) in &[
    //         (
    //             "type Foo {}",
    //             RecordDefinition::new("Foo", vec![], Position::fake()),
    //         ),
    //         (
    //             "type Foo {foo number}",
    //             RecordDefinition::new(
    //                 "Foo",
    //                 vec![types::RecordField::new(
    //                     "foo",
    //                     types::Reference::new("number", Position::fake()),
    //                 )],
    //                 Position::fake(),
    //             ),
    //         ),
    //         (
    //             "type Foo {foo number bar number}",
    //             RecordDefinition::new(
    //                 "Foo",
    //                 vec![
    //                     types::RecordField::new(
    //                         "foo",
    //                         types::Reference::new("number", Position::fake()),
    //                     ),
    //                     types::RecordField::new(
    //                         "bar",
    //                         types::Reference::new("number", Position::fake()),
    //                     ),
    //                 ],
    //                 Position::fake(),
    //             ),
    //         ),
    //     ] {
    //         assert_eq!(
    //             &record_definition().parse(input(source, "")).unwrap().0,
    //             expected
    //         );
    //     }
    // }

    // #[test]
    // fn parse_type_alias() {
    //     for (source, expected) in &[
    //         (
    //             "type foo=number",
    //             TypeAlias::new(
    //                 "foo",
    //                 types::Reference::new("number", Position::fake()),
    //                 Position::fake(),
    //             ),
    //         ),
    //         (
    //             "type foo = number",
    //             TypeAlias::new(
    //                 "foo",
    //                 types::Reference::new("number", Position::fake()),
    //                 Position::fake(),
    //             ),
    //         ),
    //         (
    //             "type foo=number|none",
    //             TypeAlias::new(
    //                 "foo",
    //                 types::Union::new(
    //                     types::Reference::new("number", Position::fake()),
    //                     types::Reference::new("none", Position::fake()),
    //                     Position::fake(),
    //                 ),
    //                 Position::fake(),
    //             ),
    //         ),
    //     ] {
    //         assert_eq!(&type_alias().parse(input(source, "")).unwrap().0, expected);
    //     }
    // }

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

        //     #[test]
        //     fn parse_expression() {
        //         assert!(expression().parse(input("", "")).is_err());
        //         assert_eq!(
        //             expression().parse(input("1", "")).unwrap().0,
        //             Number::new(
        //                 NumberRepresentation::FloatingPoint("1".into()),
        //                 Position::fake()
        //             )
        //             .into()
        //         );
        //         assert_eq!(
        //             expression().parse(input("x", "")).unwrap().0,
        //             Variable::new("x", Position::fake()).into()
        //         );
        //         assert_eq!(
        //             expression().parse(input("x + 1", "")).unwrap().0,
        //             BinaryOperation::new(
        //                 BinaryOperator::Add,
        //                 Variable::new("x", Position::fake()),
        //                 Number::new(
        //                     NumberRepresentation::FloatingPoint("1".into()),
        //                     Position::fake()
        //                 ),
        //                 Position::fake()
        //             )
        //             .into()
        //         );
        //         assert_eq!(
        //             expression().parse(input("x + y(z)", "")).unwrap().0,
        //             BinaryOperation::new(
        //                 BinaryOperator::Add,
        //                 Variable::new("x", Position::fake()),
        //                 Call::new(
        //                     Variable::new("y", Position::fake()),
        //                     vec![Variable::new("z", Position::fake()).into()],
        //                     Position::fake()
        //                 ),
        //                 Position::fake()
        //             )
        //             .into()
        //         );
        //         assert_eq!(
        //             expression().parse(input("(x + y)(z)", "")).unwrap().0,
        //             Call::new(
        //                 BinaryOperation::new(
        //                     BinaryOperator::Add,
        //                     Variable::new("x", Position::fake()),
        //                     Variable::new("y", Position::fake()),
        //                     Position::fake()
        //                 ),
        //                 vec![Variable::new("z", Position::fake()).into()],
        //                 Position::fake()
        //             )
        //             .into()
        //         );
        //     }

        //     #[test]
        //     fn parse_deeply_nested_expression() {
        //         assert_eq!(
        //             expression().parse(input("(((((42)))))", "")).unwrap().0,
        //             Number::new(
        //                 NumberRepresentation::FloatingPoint("42".into()),
        //                 Position::fake()
        //             )
        //             .into()
        //         )
        //     }

        //     #[test]
        //     fn parse_atomic_expression() {
        //         assert!(atomic_expression().parse(input("", "")).is_err());
        //         assert_eq!(
        //             atomic_expression().parse(input("1", "")).unwrap().0,
        //             Number::new(
        //                 NumberRepresentation::FloatingPoint("1".into()),
        //                 Position::fake()
        //             )
        //             .into()
        //         );
        //         assert_eq!(
        //             atomic_expression().parse(input("x", "")).unwrap().0,
        //             Variable::new("x", Position::fake()).into()
        //         );
        //         assert_eq!(
        //             atomic_expression().parse(input("(x)", "")).unwrap().0,
        //             Variable::new("x", Position::fake()).into()
        //         );
        //     }

        //     #[test]
        //     fn parse_lambda() {
        //         assert_eq!(
        //             lambda()
        //                 .parse(input("\\(x number)number{42}", ""))
        //                 .unwrap()
        //                 .0,
        //             Lambda::new(
        //                 vec![Argument::new(
        //                     "x",
        //                     types::Reference::new("number", Position::fake())
        //                 )],
        //                 types::Reference::new("number", Position::fake()),
        //                 Block::new(
        //                     vec![],
        //                     Number::new(
        //                         NumberRepresentation::FloatingPoint("42".into()),
        //                         Position::fake()
        //                     ),
        //                     Position::fake()
        //                 ),
        //                 Position::fake()
        //             ),
        //         );

        //         assert_eq!(
        //             lambda()
        //                 .parse(input("\\(x number,y number)number{42}", ""))
        //                 .unwrap()
        //                 .0,
        //             Lambda::new(
        //                 vec![
        //                     Argument::new("x", types::Reference::new("number", Position::fake())),
        //                     Argument::new("y", types::Reference::new("number", Position::fake()))
        //                 ],
        //                 types::Reference::new("number", Position::fake()),
        //                 Block::new(
        //                     vec![],
        //                     Number::new(
        //                         NumberRepresentation::FloatingPoint("42".into()),
        //                         Position::fake()
        //                     ),
        //                     Position::fake()
        //                 ),
        //                 Position::fake()
        //             ),
        //         );
        //     }

        //     #[test]
        //     fn parse_lambda_with_reference_type() {
        //         assert_eq!(
        //             lambda().parse(input("\\() Foo { 42 }", "")).unwrap().0,
        //             Lambda::new(
        //                 vec![],
        //                 types::Reference::new("Foo", Position::fake()),
        //                 Block::new(
        //                     vec![],
        //                     Number::new(
        //                         NumberRepresentation::FloatingPoint("42".into()),
        //                         Position::fake()
        //                     ),
        //                     Position::fake()
        //                 ),
        //                 Position::fake()
        //             ),
        //         );
        //     }

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

        //     #[test]
        //     fn parse_if_type() {
        //         assert_eq!(
        //             if_type()
        //                 .parse(input("if x=y as boolean {none}else{none}", ""))
        //                 .unwrap()
        //                 .0,
        //             IfType::new(
        //                 "x",
        //                 Variable::new("y", Position::fake()),
        //                 vec![IfTypeBranch::new(
        //                     types::Reference::new("boolean", Position::fake()),
        //                     Block::new(
        //                         vec![],
        //                         Variable::new("none", Position::fake()),
        //                         Position::fake()
        //                     ),
        //                 )],
        //                 Some(Block::new(
        //                     vec![],
        //                     Variable::new("none", Position::fake()),
        //                     Position::fake()
        //                 )),
        //                 Position::fake(),
        //             )
        //         );

        //         assert_eq!(
        //             if_type()
        //                 .parse(input(
        //                     "if x=y as boolean{none}else if none{none}else{none}",
        //                     ""
        //                 ))
        //                 .unwrap()
        //                 .0,
        //             IfType::new(
        //                 "x",
        //                 Variable::new("y", Position::fake()),
        //                 vec![
        //                     IfTypeBranch::new(
        //                         types::Reference::new("boolean", Position::fake()),
        //                         Block::new(
        //                             vec![],
        //                             Variable::new("none", Position::fake()),
        //                             Position::fake()
        //                         ),
        //                     ),
        //                     IfTypeBranch::new(
        //                         types::Reference::new("none", Position::fake()),
        //                         Block::new(
        //                             vec![],
        //                             Variable::new("none", Position::fake()),
        //                             Position::fake()
        //                         ),
        //                     )
        //                 ],
        //                 Some(Block::new(
        //                     vec![],
        //                     Variable::new("none", Position::fake()),
        //                     Position::fake()
        //                 )),
        //                 Position::fake()
        //             )
        //         );

        //         assert_eq!(
        //             if_type()
        //                 .parse(input("if x=y as boolean{none}else if none{none}", ""))
        //                 .unwrap()
        //                 .0,
        //             IfType::new(
        //                 "x",
        //                 Variable::new("y", Position::fake()),
        //                 vec![
        //                     IfTypeBranch::new(
        //                         types::Reference::new("boolean", Position::fake()),
        //                         Block::new(
        //                             vec![],
        //                             Variable::new("none", Position::fake()),
        //                             Position::fake()
        //                         ),
        //                     ),
        //                     IfTypeBranch::new(
        //                         types::Reference::new("none", Position::fake()),
        //                         Block::new(
        //                             vec![],
        //                             Variable::new("none", Position::fake()),
        //                             Position::fake()
        //                         ),
        //                     )
        //                 ],
        //                 None,
        //                 Position::fake()
        //             )
        //         );
        //     }

        //     #[test]
        //     fn parse_if_list() {
        //         assert_eq!(
        //             if_list()
        //                 .parse(input("if[x,...xs]=xs {none}else{none}", ""))
        //                 .unwrap()
        //                 .0,
        //             IfList::new(
        //                 Variable::new("xs", Position::fake()),
        //                 "x",
        //                 "xs",
        //                 Block::new(
        //                     vec![],
        //                     Variable::new("none", Position::fake()),
        //                     Position::fake()
        //                 ),
        //                 Block::new(
        //                     vec![],
        //                     Variable::new("none", Position::fake()),
        //                     Position::fake()
        //                 ),
        //                 Position::fake(),
        //             )
        //         );
        //     }

        //     #[test]
        //     fn parse_if_map() {
        //         assert_eq!(
        //             if_map()
        //                 .parse(input("if x=xs[42]{none}else{none}", ""))
        //                 .unwrap()
        //                 .0,
        //             IfMap::new(
        //                 "x",
        //                 Variable::new("xs", Position::fake()),
        //                 Number::new(
        //                     NumberRepresentation::FloatingPoint("42".into()),
        //                     Position::fake()
        //                 ),
        //                 Block::new(
        //                     vec![],
        //                     Variable::new("none", Position::fake()),
        //                     Position::fake()
        //                 ),
        //                 Block::new(
        //                     vec![],
        //                     Variable::new("none", Position::fake()),
        //                     Position::fake()
        //                 ),
        //                 Position::fake(),
        //             )
        //         );
        //     }

        mod call {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn parse_call() {
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
                // TODO Should we rather commit to parse calls?
                assert_eq!(
                    expression(input("f(1+)", "")).unwrap().1,
                    Variable::new("f", Position::fake()).into()
                );
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
                        "x",
                        Variable::new("xs", Position::fake()),
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
                        "x",
                        Variable::new("xs", Position::fake()),
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
                (
                    "{none:none none}",
                    Map::new(
                        types::Reference::new("none", Position::fake()),
                        types::Reference::new("none", Position::fake()),
                        vec![MapElement::Removal(
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
                MapIterationComprehension::new(
                    types::Reference::new("none", Position::fake()),
                    Variable::new("v", Position::fake()),
                    "k",
                    "v",
                    Variable::new("xs", Position::fake()),
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
