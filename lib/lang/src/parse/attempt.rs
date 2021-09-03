use combine::{attempt, Parser};

pub fn many<F: Extend<P::Output> + Default, I: combine::Stream, P: Parser<I>>(
    p: P,
) -> combine::parser::repeat::Many<F, combine::parser::combinator::Try<P>> {
    combine::many(attempt(p))
}
