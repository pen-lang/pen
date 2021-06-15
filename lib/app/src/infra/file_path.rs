use std::{fmt::Display, ops::Deref, str::FromStr};

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
        let replacement = if extension.is_empty() {
            "".into()
        } else {
            format!(".{}", extension)
        };

        Self::new(
            self.components().take(self.components.len() - 1).chain(
                vec![regex::Regex::new(r"(\..*)?$")
                    .unwrap()
                    .replace(self.components.iter().last().unwrap(), replacement.as_str())
                    .deref()]
                .into_iter(),
            ),
        )
    }

    pub fn join(&self, file_path: &Self) -> Self {
        Self::new(self.components().chain(file_path.components()))
    }

    pub fn has_extension(&self, file_extension: &str) -> bool {
        let has_extension = || {
            let component = self.components.last()?;
            let element = component.split('.').last()?;

            Some(if element == component {
                file_extension.is_empty()
            } else {
                element == file_extension
            })
        };

        has_extension().unwrap_or(false)
    }

    pub fn relative_to(&self, path: &Self) -> Self {
        Self::new(self.components().skip(path.components().count()))
    }
}

impl Display for FilePath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "{}",
            self.components().collect::<Vec<_>>().join("/")
        )
    }
}

impl FromStr for FilePath {
    type Err = ();

    fn from_str(path: &str) -> Result<Self, ()> {
        Ok(Self::new(path.split('/')))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(format!("{}", FilePath::new(&["foo"])), "foo");
        assert_eq!(format!("{}", FilePath::new(&["foo", "bar"])), "foo/bar");
    }

    #[test]
    fn with_extension() {
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
