use inquire::InquireError;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
};

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
pub(crate) fn get_opts_path() -> PathBuf {
    let home: String = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let opts_path: PathBuf = PathBuf::from(format!("{}/.decoy/opts.toml", home));

    opts_path
}

pub(crate) fn load_opts() -> Result<Opts, InquireError> {
    let opts_path: PathBuf = get_opts_path();

    if opts_path.exists() {
        // Read file content
        let opts = fs::read_to_string(&opts_path).unwrap_or_default();
        let opts: Opts = toml::from_str(&opts).unwrap_or_default();

        return Ok(opts);
    }

    // Use the default opts if there is no opt file
    Ok(Opts::default())
}

pub(crate) fn generate_default_opts() -> std::io::Result<()> {
    let opts_path = get_opts_path();

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
