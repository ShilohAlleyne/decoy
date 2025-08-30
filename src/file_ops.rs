use colored::Colorize;
use inquire::InquireError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::{self, Display},
    fs::{self, File, OpenOptions},
    io::{Error, Write},
    path::{Path, PathBuf},
    process::Command,
};

// --- Notes ---
#[derive(Debug, Clone)]
pub(crate) struct Note(pub PathBuf);

impl Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stem: Option<&str> = self.0.file_name().and_then(|s| s.to_str());

        if let Some(filename) = stem {
            if filename.len() < 15 {
                return writeln!(f, "{:?}", self.0);
            }

            let (ident, tail) = filename.split_at(15);

            if let Some((name, raw_kws)) = tail.split_once("__") {
                let kws = raw_kws
                    .split_once('.')
                    .map(|(fn_name, _)| fn_name)
                    .unwrap_or(raw_kws)
                    .split('_')
                    .map(|k| format!("{}", k.yellow()))
                    .collect::<Vec<_>>()
                    .join("_");

                let fformat = raw_kws
                    .split_once('.')
                    .map(|(_, ext)| format!(".{}", ext.clear()))
                    .unwrap_or_default();

                return writeln!(f, "{}{}__{}{}", ident.cyan(), name, kws, fformat);
            }
        }

        writeln!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FrontMatter {
    pub title: String,
    pub date: String,
    pub file_tags: Vec<String>,
    pub indentifier: String,
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
        .unique()
        .collect()
}

// --- File manipulation ---
pub(crate) fn search(notes: &[Note], keywords: Vec<String>) -> Vec<Note> {
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

pub(crate) fn write_new_note(path: &Path, frontmatter: FrontMatter) -> std::io::Result<()> {
    let yaml = serde_yaml::to_string(&frontmatter).map_err(Error::other)?;

    let mut file: File = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    file.write_all(format!("---\n{}---\n", yaml).as_bytes())?;

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
