use super::Document;

pub fn is_broken(document: &Document) -> bool {
    match document {
        Document::Break(broken, _) => *broken,
        Document::Indent(document) => is_broken(document),
        Document::Sequence(documents) => documents.iter().any(is_broken),
        Document::Line | Document::LineSuffix(_) | Document::String(_) => false,
    }
}

pub fn count_lines(document: &Document) -> usize {
    match document {
        Document::Break(broken, document) => {
            if *broken {
                count_lines(document)
            } else {
                0
            }
        }
        Document::Indent(document) => count_lines(document),
        Document::Line => 1,
        Document::Sequence(documents) => documents.iter().map(count_lines).sum(),
        Document::LineSuffix(_) | Document::String(_) => 0,
    }
}
