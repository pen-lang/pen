use nom::{
    IResult, InputLength, Parser,
    combinator::opt,
    error::ParseError,
    multi::{separated_list0, separated_list1},
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::NomError, input::input};
    use nom::{bytes::complete::tag, combinator::all_consuming};
    use std::str;

    mod separated_or_terminated_list0 {
        use super::*;

        #[test]
        fn parse() {
            for (source, expected) in [
                ("", &[] as &[&str]),
                ("a", &["a"]),
                ("a,a", &["a", "a"]),
                ("a,a,", &["a", "a"]),
            ] {
                assert_eq!(
                    separated_or_terminated_list0(
                        |input| tag::<_, _, NomError>(",")(input),
                        tag("a")
                    )(input(source, ""))
                    .unwrap()
                    .1
                    .iter()
                    .map(|span| str::from_utf8(span.as_bytes()).unwrap())
                    .collect::<Vec<_>>(),
                    expected
                );
            }
        }

        #[test]
        fn fail_to_parse() {
            for source in [",", "a,,"] {
                assert!(
                    all_consuming(separated_or_terminated_list0(
                        |input| tag::<_, _, NomError>(",")(input),
                        tag("a")
                    ))(input(source, ""))
                    .is_err()
                );
            }
        }
    }

    mod separated_or_terminated_list1 {
        use super::*;

        #[test]
        fn parse() {
            for (source, expected) in [
                ("a", &["a"] as &[&str]),
                ("a,a", &["a", "a"]),
                ("a,a,", &["a", "a"]),
            ] {
                assert_eq!(
                    separated_or_terminated_list1(
                        |input| tag::<_, _, NomError>(",")(input),
                        tag("a")
                    )(input(source, ""))
                    .unwrap()
                    .1
                    .iter()
                    .map(|span| str::from_utf8(span.as_bytes()).unwrap())
                    .collect::<Vec<_>>(),
                    expected
                );
            }
        }

        #[test]
        fn fail_to_parse() {
            for source in ["", ",", "a,,"] {
                assert!(
                    all_consuming(separated_or_terminated_list1(
                        |input| tag::<_, _, NomError>(",")(input),
                        tag("a")
                    ))(input(source, ""))
                    .is_err()
                );
            }
        }
    }
}
