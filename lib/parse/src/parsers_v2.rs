use ast::*;
use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, none_of},
    multi::many0,
    IResult,
};
use nom_locate::LocatedSpan;
use position::Position;

type Input<'a> = LocatedSpan<&'a str, &'a str>;

fn comment(input: Input) -> IResult<Input, Comment> {
    let position = position(input);

    let (input, _) = tag("#")(input)?;
    let (input, comment) = many0(none_of("\n\r"))(input)?;

    Ok((input, Comment::new(String::from_iter(comment), position)))
}

fn position(input: Input) -> Position {
    Position::new(
        input.extra,
        input.location_line() as usize,
        input.get_column(),
        String::from_utf8_lossy(input.get_line_beginning()),
    )
}
