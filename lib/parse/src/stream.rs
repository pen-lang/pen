use combine::{
    easy,
    stream::{
        position::{self, SourcePosition},
        state,
    },
};

pub struct State<'a> {
    pub path: String,
    pub lines: Vec<&'a str>,
}

pub type Stream<'a> =
    easy::Stream<state::Stream<position::Stream<&'a str, SourcePosition>, State<'a>>>;

pub fn stream<'a>(source: &'a str, path: &str) -> Stream<'a> {
    state::Stream {
        stream: position::Stream::new(source),
        state: State {
            path: path.into(),
            lines: source.split('\n').collect(),
        },
    }
    .into()
}
