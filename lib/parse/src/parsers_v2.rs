use ast::*;
use nom::IResult;

struct Input<'a> {
    string: &'a str,
}

fn comment(input: Input) -> IResult<Input, Comment> {
    let (input, foo) = input;
}
