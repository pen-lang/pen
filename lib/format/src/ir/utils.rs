use super::Document;

pub fn is_broken(document: &Document) -> bool {
    match document {
        Document::Indent(document) => is_broken(document),
        Document::Line(hard) => *hard,
        Document::Sequence(documents) => documents.iter().any(is_broken),
        Document::LineSuffix(_) | Document::Flatten(_) | Document::String(_) => false,
    }
}

pub fn count_lines(document: &Document) -> usize {
    match document {
        Document::Indent(document) => count_lines(document),
        Document::Line(_) => 1,
        Document::Sequence(documents) => documents.iter().map(count_lines).sum(),
        Document::LineSuffix(_) | Document::Flatten(_) | Document::String(_) => 0,
    }
}
