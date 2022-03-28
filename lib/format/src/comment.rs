use ast::Comment;

pub fn sort(comments: &[Comment]) -> Vec<Comment> {
    let mut comments = comments.to_vec();

    comments.sort_by_key(|comment| comment.position().line_number());

    comments
}

pub fn split_before(comments: &[Comment], line_number: usize) -> (&[Comment], &[Comment]) {
    let index = comments
        .iter()
        .position(|comment| comment.position().line_number() >= line_number)
        .unwrap_or(comments.len());

    (&comments[..index], &comments[index..])
}

pub fn split_current(comments: &[Comment], line_number: usize) -> (Option<&Comment>, &[Comment]) {
    if let Some(index) = comments
        .iter()
        .position(|comment| comment.position().line_number() == line_number)
    {
        (comments.get(index), &comments[index + 1..])
    } else {
        (None, comments)
    }
}
