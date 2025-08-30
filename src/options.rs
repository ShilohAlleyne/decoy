use std::{env, fs, path::PathBuf};
use inquire::InquireError;
use serde::{Deserialize, Serialize};


// --- Basic CLI opts ---
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Opts {
    pub note_dir: PathBuf,
    pub notes_filetype: FileType,
}

impl Default for Opts {
    fn default() -> Self {
        // Notes are always in either home/notes or somewhere else
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Opts {
            note_dir: PathBuf::from(format!("{}/notes/", home)),
            notes_filetype: FileType::Markdown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) enum FileType {
    Markdown,
    Text,
    Org,
}

// --- Load things ---
pub(crate) fn load_opts() -> Result<Opts, InquireError> {
    let home: String = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let opts_path: PathBuf = PathBuf::from(format!("{}/.decoy/opts.json", home));

    // Read file content
    let json_opts = fs::read_to_string(&opts_path);

    match json_opts {
        Ok(opts_json) => {
            let opts: Opts = serde_json::from_str(&opts_json).unwrap_or_default();

            Ok(opts)
        }
        // Use the default opts if there is no opt file
        Err(_) => Ok(Opts::default()),
    }
}
