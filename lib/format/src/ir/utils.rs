use super::Document;

pub fn is_broken(document: &Document) -> bool {
    match document {
        Document::Sequence(documents) => documents.iter().all(is_broken),
        Document::Flatten(_) => false,
        Document::Indent(document) => is_broken(document),
        Document::HardLine => true,
        Document::HardLine => true,
        Document::Comment(_) | Document::SoftLine | Document::String(_) => false,
    }
}
