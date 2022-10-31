use nom::{
    combinator::opt,
    error::ParseError,
    multi::{separated_list0, separated_list1},
    IResult, InputLength, Parser,
};

pub fn separated_or_terminated_list0<I, O, O2, E, F, G>(
    sep: G,
    f: F,
) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    G: Parser<I, O2, E> + Clone,
    E: ParseError<I>,
{
    let mut p = separated_list0(sep.clone(), f);
    let mut term = opt(sep);

    move |input: I| {
        let (input, xs) = p(input)?;

        Ok(if xs.is_empty() {
            (input, xs)
        } else {
            let (input, _) = term(input)?;

            (input, xs)
        })
    }
}

pub fn separated_or_terminated_list1<I, O, O2, E, F, G>(
    sep: G,
    f: F,
) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    G: Parser<I, O2, E> + Clone,
    E: ParseError<I>,
{
    let mut p = separated_list1(sep.clone(), f);
    let mut term = opt(sep);

    move |input: I| {
        let (input, xs) = p(input)?;
        let (input, _) = term(input)?;

        Ok((input, xs))
    }
}
