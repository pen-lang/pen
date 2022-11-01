use nom::{
    combinator::opt,
    error::ParseError,
    multi::{separated_list0, separated_list1},
    IResult, InputLength, Parser,
};

pub fn separated_or_terminated_list0<I: Clone + InputLength, O, S, E: ParseError<I>>(
    separator: impl Parser<I, S, E> + Clone,
    element: impl Parser<I, O, E>,
) -> impl FnMut(I) -> IResult<I, Vec<O>, E> {
    let mut list = separated_list0(separator.clone(), element);
    let mut end = opt(separator);

    move |input: I| {
        let (input, xs) = list(input)?;

        Ok(if xs.is_empty() {
            (input, xs)
        } else {
            let (input, _) = end(input)?;

            (input, xs)
        })
    }
}

pub fn separated_or_terminated_list1<I: Clone + InputLength, O, S, E: ParseError<I>>(
    separator: impl Parser<I, S, E> + Clone,
    element: impl Parser<I, O, E>,
) -> impl FnMut(I) -> IResult<I, Vec<O>, E> {
    let mut list = separated_list1(separator.clone(), element);
    let mut end = opt(separator);

    move |input: I| {
        let (input, xs) = list(input)?;
        let (input, _) = end(input)?;

        Ok((input, xs))
    }
}
