use nom_locate::LocatedSpan;
use position::Position;
use std::str;

pub type Input<'a> = LocatedSpan<&'a str, &'a str>;

pub fn input<'a>(source: &'a str, path: &'a str) -> Input<'a> {
    LocatedSpan::new_extra(source, path)
}

pub fn position(input: Input) -> Position {
    Position::new(
        input.extra,
        input.location_line() as usize,
        input.get_column(),
        str::from_utf8(input.get_line_beginning()).unwrap(),
    )
}
