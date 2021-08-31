use combine::{attempt, Parser};

#[macro_export]
macro_rules! choice {
    ($($x:expr),* $(,)?) => {
        combine::choice(($(combine::attempt($x),)*))
    }
}

pub fn many<F: Extend<P::Output> + Default, I: combine::Stream, P: Parser<I>>(
    p: P,
) -> combine::parser::repeat::Many<F, combine::parser::combinator::Try<P>> {
    combine::many(attempt(p))
}

pub fn many1<F: Extend<P::Output> + Default, I: combine::Stream, P: Parser<I>>(
    p: P,
) -> combine::parser::repeat::Many1<F, combine::parser::combinator::Try<P>> {
    combine::many1(attempt(p))
}

pub fn optional<I: combine::Stream, P: Parser<I>>(
    p: P,
) -> combine::parser::choice::Optional<combine::parser::combinator::Try<P>> {
    combine::optional(attempt(p))
}

pub fn sep_end_by<
    F: Extend<P::Output> + Default,
    I: combine::Stream,
    P: Parser<I>,
    S: Parser<I>,
>(
    p: P,
    s: S,
) -> combine::parser::repeat::SepEndBy<
    F,
    combine::parser::combinator::Try<P>,
    combine::parser::combinator::Try<S>,
> {
    combine::sep_end_by(attempt(p), attempt(s))
}

pub fn sep_end_by1<
    F: Extend<P::Output> + Default,
    I: combine::Stream,
    P: Parser<I>,
    S: Parser<I>,
>(
    p: P,
    s: S,
) -> combine::parser::repeat::SepEndBy1<
    F,
    combine::parser::combinator::Try<P>,
    combine::parser::combinator::Try<S>,
> {
    combine::sep_end_by1(attempt(p), attempt(s))
}
