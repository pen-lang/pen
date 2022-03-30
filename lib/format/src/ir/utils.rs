use super::Document;

pub fn is_broken(document: &Document) -> bool {
    match document {
        Document::Break => true,
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
    count_lines_in_document(document) + 1
}

fn count_lines_in_document(document: &Document) -> usize {
    match document {
        Document::Indent(document) => count_lines(document),
        Document::Sequence(documents) => documents.iter().map(count_lines).sum(),
        Document::LineSuffix(_) | Document::Flatten(_) | Document::Break | Document::String(_) => 0,
        Document::Line | Document::SoftLine => 1,
    }
}
