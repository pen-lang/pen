use super::Document;

pub fn is_broken(document: &Document) -> bool {
    match document {
        Document::Break => true,
        Document::Indent(document) => is_broken(document),
        Document::Sequence(documents) => documents.iter().all(is_broken),
        Document::Comment(_) | Document::Flatten(_) | Document::Line | Document::String(_) => false,
    }
}
