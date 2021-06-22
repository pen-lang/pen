use std::ops::Deref;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FilePath {
    components: Vec<String>,
}

impl FilePath {
    pub fn new<I: IntoIterator<Item = impl AsRef<str>>>(components: I) -> Self {
        Self {
            components: components
                .into_iter()
                .map(|string| string.as_ref().into())
                .collect(),
        }
    }

    pub fn empty() -> Self {
        Self { components: vec![] }
    }

    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.components.iter().map(Deref::deref)
    }

    pub fn with_extension(&self, extension: &str) -> Self {
        if let Some(last) = self.components.last() {
            Self::new(
                self.components().take(self.components.len() - 1).chain(
                    vec![regex::Regex::new(r"(\..*)?$")
                        .unwrap()
                        .replace(
                            last,
                            if extension.is_empty() {
                                "".into()
                            } else {
                                format!(".{}", extension)
                            }
                            .as_str(),
                        )
                        .deref()]
                    .into_iter(),
                ),
            )
        } else {
            self.clone()
        }
    }

    pub fn join(&self, file_path: &Self) -> Self {
        Self::new(self.components().chain(file_path.components()))
    }

    pub fn has_extension(&self, extension: &str) -> bool {
        self.check_extension(extension).unwrap_or_default()
    }

    fn check_extension(&self, extension: &str) -> Option<bool> {
        let component = self.components.last()?;
        let element = component.split('.').last()?;

        Some(if element == component {
            extension.is_empty()
        } else {
            element == extension
        })
    }

    // TODO Check a directory path.
    pub fn relative_to(&self, path: &Self) -> Self {
        Self::new(self.components().skip(path.components().count()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_extension() {
        assert_eq!(FilePath::empty().with_extension(""), FilePath::empty());
        assert_eq!(
            FilePath::new(&["foo"]).with_extension("c"),
            FilePath::new(&["foo.c"])
        );
        assert_eq!(
            FilePath::new(&["foo", "bar"]).with_extension("c"),
            FilePath::new(&["foo", "bar.c"])
        );
        assert_eq!(
            FilePath::new(&["foo.c"]).with_extension(""),
            FilePath::new(&["foo"])
        );
        assert_eq!(
            FilePath::new(&["foo.c"]).with_extension("h"),
            FilePath::new(&["foo.h"])
        );
    }

    #[test]
    fn join() {
        assert_eq!(
            FilePath::new(&["foo"]).join(&FilePath::empty()),
            FilePath::new(&["foo"])
        );

        assert_eq!(
            FilePath::empty().join(&FilePath::new(&["foo"])),
            FilePath::new(&["foo"])
        );

        assert_eq!(
            FilePath::new(&["foo"]).join(&FilePath::new(&["bar"])),
            FilePath::new(&["foo", "bar"])
        );

        assert_eq!(
            FilePath::new(&["foo", "bar"]).join(&FilePath::new(&["baz"])),
            FilePath::new(&["foo", "bar", "baz"])
        );
    }

    #[test]
    fn has_extension() {
        assert!(!FilePath::empty().has_extension(""));
        assert!(!FilePath::empty().has_extension("foo"));
        assert!(FilePath::new(&["foo"]).has_extension(""));
        assert!(!FilePath::new(&["foo"]).has_extension("foo"));
        assert!(FilePath::new(&["foo.bar"]).has_extension("bar"));
        assert!(!FilePath::new(&["foo.bar"]).has_extension("baz"));

        assert!(FilePath::new(&["foo", "bar"]).has_extension(""));
        assert!(!FilePath::new(&["foo", "bar"]).has_extension("bar"));
        assert!(FilePath::new(&["foo", "bar.baz"]).has_extension("baz"));
        assert!(!FilePath::new(&["foo", "bar.baz"]).has_extension("blah"));
    }
}
