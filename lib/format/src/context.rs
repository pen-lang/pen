use ast::Comment;
use bumpalo::Bump;
use mfmt::Builder;
use std::collections::VecDeque;

pub struct Context<'a> {
    builder: Builder<&'a Bump>,
    comments: VecDeque<&'a Comment>,
}

impl<'a> Context<'a> {
    pub fn new(allocator: &'a Bump, comments: &'a [Comment]) -> Self {
        Self {
            builder: Builder::new(allocator),
            comments: comments.iter().collect(),
        }
    }

    pub fn builder(&self) -> Builder<&'a Bump> {
        self.builder.clone()
    }

    pub fn drain_comments_before(
        &mut self,
        line_number: usize,
    ) -> impl Iterator<Item = &'a Comment> + '_ {
        self.comments.drain(
            ..self
                .comments
                .iter()
                .position(|comment| comment.position().line_number() >= line_number)
                .unwrap_or(self.comments.len()),
        )
    }

    pub fn drain_current_comment(
        &mut self,
        line_number: usize,
    ) -> impl Iterator<Item = &'a Comment> + '_ {
        self.drain_comments_before(line_number + 1)
    }

    pub fn peek_comments_before(
        &self,
        line_number: usize,
    ) -> impl Iterator<Item = &'a Comment> + '_ {
        self.comments
            .range(
                ..self
                    .comments
                    .iter()
                    .position(|comment| comment.position().line_number() >= line_number)
                    .unwrap_or(self.comments.len()),
            )
            .copied()
    }
}
