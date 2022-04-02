mod ir;
mod markdown;

use ast::*;
use ir::*;
use ir::build::*;

pub fn generate(path: &ModulePath, module: &Module, comments: &[Comment]) -> String {
    markdown::generate(&compile_module(path, module, comments))
}

fn compile_module(path: &ModulePath, module: &Module, comments: &[Comment]) -> Section {
    section(text([code(format!("{}", path))]), [], [])
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use position::test::PositionFake;
    use position::Position;

    mod module {

        use super::*;

        fn generate(path: &ModulePath, module: &Module, comments: &[Comment]) -> String {
            markdown::generate(&compile_module(path, module, comments))
        }

        #[test]
        fn generate_empty() {
            assert_eq!(
                generate(
                    &ExternalModulePath::new("Foo", vec!["Bar".into()]).into(),
                    &Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                    &[]
                ),
                indoc!(
                    "
                    # `Foo'Bar`
                    "
                )
            );
        }
    }
}
