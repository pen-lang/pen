use nom_locate::LocatedSpan;

pub type Input<'a> = LocatedSpan<&'a str, &'a str>;

pub fn input<'a>(source: &'a str, path: &'a str) -> Input<'a> {
    LocatedSpan::new_extra(source, path)
}
