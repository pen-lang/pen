use crate::ir;

struct Context {
    outputs: Vec<String>,
    comments: Vec<String>,
}

pub fn format(document: &ir::Document) -> String {
    let mut context = Context {
        outputs: vec![],
        comments: vec![],
    };

    format_document(&mut context, &document, 0, false);

    context.outputs.concat()
}

fn format_document(context: &mut Context, document: &ir::Document, level: usize, broken: bool) {
    match document {
        ir::Document::Comment(comment) => context.comments.push(comment.clone()),
        ir::Document::Documents(documents) => {
            for document in documents {
                format_document(context, document, level, broken);
            }
        }
        ir::Document::Group {
            document,
            broken: group_broken,
        } => format_document(context, document, level, *group_broken),
        ir::Document::Indent(document) => format_document(context, document, level + 1, broken),
        ir::Document::Line { document, soft } => {
            if broken || !soft {
                context
                    .outputs
                    .extend(["\n".into(), context.comments.join(" ").into()]);
            } else {
                context.outputs.extend([" ".into()]);
            }
        }
        ir::Document::String(string) => context.outputs.push(string.clone()),
    }
}
