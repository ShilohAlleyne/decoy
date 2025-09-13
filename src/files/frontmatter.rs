use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub date: String,
    pub file_tags: Vec<String>,
    pub indentifier: String,
}

pub fn to_org_front_matter(fmt: FrontMatter) -> String {
    let mut lines = vec![];

    lines.push(format!("#+TITLE: {}", fmt.title));
    lines.push(format!("#+DATE: {}", fmt.date));
    lines.push(format!("#+FILETAGS: {}", fmt.file_tags.join(" ")));
    lines.push(format!("#+IDENTIFIER: {}", fmt.indentifier));

    lines.join("\n")
}
