use ast::Comment;

pub struct Context {
    comments: Vec<Comment>,
}

impl Context {
    pub fn new(mut comments: Vec<Comment>) -> Self {
        comments.sort_by_key(|comment| comment.position().line_number());

        Self { comments }
    }

    pub fn pop_before_line(&mut self, line_number: usize) -> Vec<Comment> {
        self.comments
            .splice(
                ..self
                    .comments
                    .iter()
                    .position(|comment| comment.position().line_number() < line_number)
                    .map(|index| index + 1)
                    .unwrap_or_default(),
                [],
            )
            .collect()
    }

    pub fn pop_on_line(&mut self, line_number: usize) -> Option<Comment> {
        self.comments
            .iter()
            .position(|comment| comment.position().line_number() == line_number)
            .map(|index| self.comments.remove(index))
    }
}
