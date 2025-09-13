use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileType {
    Markdown,
    Text,
    Org,
    Typst,
}

impl AsRef<str> for FileType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Markdown => ".md",
            Self::Text => ".txt",
            Self::Org => ".org",
            Self::Typst => ".typ",
        }
    }
}
