use std::{env, fs::{self, File, OpenOptions}, io::Write, path::PathBuf};
use inquire::InquireError;
use serde::{Deserialize, Serialize};

use crate::{files::types, options::editor};


// --- Basic CLI opts ---
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Opts {
    pub opts_path: PathBuf,
    pub note_dir: PathBuf,
    pub notes_filetype: types::FileType,
    #[serde(default = "editor::Editor::default")]
    pub editor: editor::Editor,
}

impl Default for Opts {
    fn default() -> Self {
        // Notes are always in either home/notes or somewhere else
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Opts {
            opts_path: get_path(),
            note_dir: PathBuf::from(format!("{}/notes/", home)),
            notes_filetype: types::FileType::Markdown,
            editor: editor::Editor::default(),
        }
    }
}

// --- Load things ---
fn get_path() -> PathBuf {
    let home: String = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let opts_path: PathBuf = PathBuf::from(format!("{}/.decoy/opts.toml", home));

    opts_path
}

fn generate_default_opts_file() -> std::io::Result<()> {
    let opts_path = get_path();

    if !opts_path.exists() {
        // create the opts dir
        if let Some(parent) = opts_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file: File = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&opts_path)?;

        let toml: String = toml::to_string_pretty(&Opts::default())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid Options"))?;

        file.write_all(toml.as_bytes())?;
    }

    Ok(())
}

pub fn load() -> Result<Opts, InquireError> {
    let opts_path: PathBuf = get_path();

    if opts_path.exists() {
        // Read file content
        let opts = fs::read_to_string(&opts_path).unwrap_or_default();
        let opts: Opts = toml::from_str(&opts).unwrap_or_default();

        return Ok(opts);
    }

    // Use the default opts if there is no opt file
    generate_default_opts_file()?;
    Ok(Opts::default())
}
