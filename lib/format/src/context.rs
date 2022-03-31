use ast::Comment;

pub struct Context {
    comments: Vec<Comment>,
}

impl Context {
    pub fn new(mut comments: Vec<Comment>) -> Self {
        comments.sort_by_key(|comment| comment.position().line_number());

        Self { comments }
    }

    pub fn split_before(&mut self, line_number: usize) -> impl Iterator<Item = Comment> + '_ {
        self.comments.splice(
            ..self
                .comments
                .iter()
                .position(|comment| comment.position().line_number() >= line_number)
                .unwrap_or(self.comments.len()),
            [],
        )
    }

    pub fn split_current(&mut self, line_number: usize) -> impl Iterator<Item = Comment> + '_ {
        self.split_before(line_number + 1)
    }
}
