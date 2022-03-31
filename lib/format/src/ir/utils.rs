use super::Document;

pub fn is_broken(document: &Document) -> bool {
    match document {
        Document::HardLine => true,
        Document::Indent(document) => is_broken(document),
        Document::Sequence(documents) => documents.iter().all(is_broken),
        Document::LineSuffix(_)
        | Document::Flatten(_)
        | Document::Line
        | Document::SoftLine
        | Document::String(_) => false,
    }
}

pub fn count_lines(document: &Document) -> usize {
    match document {
        Document::Indent(document) => count_lines(document),
        Document::Sequence(documents) => documents.iter().map(count_lines).sum(),
        Document::LineSuffix(_) | Document::Flatten(_) | Document::String(_) => 0,
        Document::HardLine | Document::Line | Document::SoftLine => 1,
    }
}
