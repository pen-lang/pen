use ast::Comment;

pub struct Context {
    comments: Vec<Comment>,
}

impl Context {
    pub fn new(mut comments: Vec<Comment>) -> Self {
        comments.sort_by_key(|comment| comment.position().line_number());

        Self { comments }
    }

    pub fn drain_comments_before(
        &mut self,
        line_number: usize,
    ) -> impl Iterator<Item = Comment> + '_ {
        // This is O(n) and slow. Ha ha!
        self.comments.splice(
            ..self
                .comments
                .iter()
                .position(|comment| comment.position().line_number() >= line_number)
                .unwrap_or(self.comments.len()),
            [],
        )
    }

    pub fn drain_current_comment(
        &mut self,
        line_number: usize,
    ) -> impl Iterator<Item = Comment> + '_ {
        self.drain_comments_before(line_number + 1)
    }

    pub fn peek_comments_before(&self, line_number: usize) -> impl Iterator<Item = &Comment> {
        self.comments[..self
            .comments
            .iter()
            .position(|comment| comment.position().line_number() >= line_number)
            .unwrap_or(self.comments.len())]
            .iter()
    }
}
