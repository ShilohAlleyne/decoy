use std::{fmt::{self, Display}, fs, path::{Path, PathBuf}};

use chrono::{NaiveDate, NaiveDateTime};
use colored::Colorize;
use inquire::InquireError;
use itertools::Itertools;

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

// --- Loading ---
pub(crate) fn load(path: &Path) -> Result<Vec<Note>, InquireError> {
    let mut notes: Vec<Note> = Vec::new();

    for entry in fs::read_dir(path)? {
        let dir = entry?;
        notes.push(Note(dir.path()));
    }

    Ok(notes)
}

// --- Parsing ---
pub(crate) fn parse_all_keywords(notes: &[Note]) -> Vec<String> {
    notes
        .iter()
        .filter_map(|p| p.0.file_stem()?.to_str())
        .flat_map(|name| name.split("__").skip(1).flat_map(|seg| seg.split('_')))
        .map(|tag| tag.to_string())
        .filter(|tag| !tag.is_empty())
        .unique()
        .collect()
}

pub(crate) fn parse_date(note: &Note) -> Option<NaiveDate> {
    let filename = note.0.file_stem()?.to_str()?;
    let (ident, _) = filename.split_once("--")?;

    NaiveDateTime::parse_from_str(ident, "%Y%m%dT%H%M%S")
        .ok()
        .map(|dt| dt.date())
}

// --- Note manipulation ---
pub fn search_by_date(notes: &[Note], date: NaiveDate) -> Vec<Note> {
    notes.iter()
        .filter(|note| {
            match parse_date(note) {
                Some(note_date) => note_date == date,
                None => false,
            }
        })
        .cloned()
        .collect()
}

// --- File manipulation ---
pub fn search_by_keywords(notes: &[Note], keywords: Vec<String>) -> Vec<Note> {
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
