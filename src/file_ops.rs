use chrono::{NaiveDate, NaiveDateTime};
use colored::Colorize;
use inquire::InquireError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::{self, Display},
    fs::{self, create_dir_all, File, OpenOptions},
    io::{Error, Write},
    path::{Path, PathBuf},
    process::Command,
};

use crate::options;

// --- Notes ---
#[derive(Debug, Clone)]
pub(crate) struct Note(pub PathBuf);

impl Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stem = self.0.file_name().and_then(|s| s.to_str());

        if let Some(filename) = stem {
            let (ident_opt, tail) = filename.split_once("--").map_or((None, filename), |(id, t)| (Some(id), t));
            let (name, raw_kws) = tail.split_once("__").unwrap_or((tail, ""));

            let (kws_part, ext_part) = raw_kws
                .split_once('.')
                .map(|(kws, ext)| (kws, Some(ext)))
                .unwrap_or((raw_kws, None));

            let styled_kws = kws_part
                .split('_')
                .filter(|k| !k.is_empty())
                .map(|k| format!("{}", k.yellow()))
                .collect::<Vec<_>>()
                .join("_");

            let styled_ext = ext_part.map(|ext| format!(".{}", ext.clear())).unwrap_or_default();
            let separator = if !styled_kws.is_empty() { "__" } else { "" };

            match ident_opt {
                Some(ident) => writeln!(
                    f,
                    "{}--{}{}{}{}",
                    ident.cyan(),
                    name,
                    separator,
                    styled_kws,
                    styled_ext
                ),
                None => writeln!(
                    f,
                    "{}{}{}{}",
                    name,
                    separator,
                    styled_kws,
                    styled_ext
                ),
            }
        } else {
            writeln!(f, "{:?}", self.0)
        }
    }
}

impl Note {
    pub(crate) fn date(&self) -> Option<NaiveDate> {
        let filename = self.0.file_stem()?.to_str()?;
        let (ident, _) = filename.split_once("--")?;

        NaiveDateTime::parse_from_str(ident, "%Y%m%dT%H%M%S")
            .ok()
            .map(|dt| dt.date())
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FrontMatter {
    pub title: String,
    pub date: String,
    pub file_tags: Vec<String>,
    pub indentifier: String,
}

impl FrontMatter {
    pub fn to_org_front_matter(&self) -> String {
        let mut lines = vec![];

        lines.push(format!("#+TITLE: {}", self.title));
        lines.push(format!("#+DATE: {}", self.date));
        lines.push(format!("#+FILETAGS: {}", self.file_tags.join(" ")));
        lines.push(format!("#+IDENTIFIER: {}", self.indentifier));

        lines.join("\n")
    }
}

// --- Load things ---
pub(crate) fn load_notes(note_dir: &Path) -> Result<Vec<Note>, InquireError> {
    let mut notes: Vec<Note> = Vec::new();

    for entry in fs::read_dir(note_dir)? {
        let dir = entry?;
        notes.push(Note(dir.path()));
    }

    Ok(notes)
}

pub(crate) fn load_key_words(notes: &[Note]) -> Vec<String> {
    notes
        .iter()
        .filter_map(|p| p.0.file_stem()?.to_str())
        .flat_map(|name| name.split("__").skip(1).flat_map(|seg| seg.split('_')))
        .map(|tag| tag.to_string())
        .filter(|tag| !tag.is_empty())
        .unique()
        .collect()
}

// --- File manipulation ---
pub(crate) fn search_by_kws(notes: &[Note], keywords: Vec<String>) -> Vec<Note> {
    if keywords.is_empty() {
        notes.to_vec()
    } else {
        notes
            .iter()
            .filter(|note| {
                let tags = note
                    .0
                    .file_stem()
                    .and_then(|os_str| os_str.to_str())
                    .map(|name| {
                        name.split("__")
                            .skip(1)
                            .flat_map(|seg| seg.split('_'))
                            .map(|tag| tag.to_string())
                            .unique()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                tags.iter().any(|tag| keywords.contains(tag))
            })
            .cloned()
            .collect()
    }
}


pub(crate) fn search_by_date(notes: &[Note], date: NaiveDate) -> Vec<Note> {
    notes.iter()
        .filter(|note| {
            match note.date() {
                Some(note_date) => note_date == date,
                None => false,
            }
        })
        .cloned()
        .collect()
}

pub(crate) fn write_new_note(
    path: &Path,
    frontmatter: FrontMatter,
    ext: options::FileType,
) -> std::io::Result<()> {
    let fm = match ext {
        options::FileType::Org => frontmatter.to_org_front_matter().into_bytes(),
        _ => format!(
            "---\n{}---\n",
            serde_yaml::to_string(&frontmatter).map_err(Error::other)?
        )
        .into_bytes(),
    };

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut file: File = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    // Typst files have no frontmatter
    if !(ext == options::FileType::Typst) {
        file.write_all(&fm)?;
    }

    Ok(())
}

pub(crate) fn rename_file(path: &mut PathBuf, new_name: &str) -> std::io::Result<()> {
    let new_path = path.with_file_name(new_name);
    fs::rename(&*path, &new_path)?;
    *path = new_path;
    Ok(())
}

pub(crate) fn open_with_editor(path: &Path) -> std::io::Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    Command::new(editor).arg(path).status()?;

    Ok(())
}
