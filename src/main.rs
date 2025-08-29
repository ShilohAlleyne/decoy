use chrono::Local;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use inquire::{
    error::InquireResult,
    formatter::{MultiOptionFormatter, OptionFormatter},
    ui::{Attributes, Color, RenderConfig, StyleSheet, Styled},
    validator::Validation,
    Autocomplete, InquireError, MultiSelect, Select, Text,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::{self, Display},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

// --- Basic CLI opts ---
#[derive(Debug, Serialize, Deserialize)]
struct Opts {
    note_dir: PathBuf,
    notes_filetype: FileType,
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
enum FileType {
    Markdown,
    Text,
    Org,
}

// --- Auto complete ---
#[derive(Clone, Default)]
struct KeywordCompleter {
    keywords: Vec<String>,
}

impl KeywordCompleter {
    fn new(keywords: Vec<String>) -> Self {
        Self { keywords }
    }

    fn fuzzy_sort(&self, input: &str) -> Vec<(String, i64)> {
        let mut matches: Vec<(String, i64)> = self
            .keywords
            .iter()
            .filter_map(|path| {
                SkimMatcherV2::default()
                    .smart_case()
                    .fuzzy_match(path, input)
                    .map(|score| (path.clone(), score))
            })
            .collect();

        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches
    }
}

impl Autocomplete for KeywordCompleter {
    fn get_suggestions(&mut self, _input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        Ok(self.keywords.clone())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        let last_word = input.split_whitespace().last().unwrap_or(""); // fallback to empty string if input is blank

        let prefix = input
            .rsplit_once(last_word)
            .map(|(before, _)| before)
            .unwrap_or("");

        let suggestion = highlighted_suggestion.or_else(|| {
            self.fuzzy_sort(last_word)
                .into_iter()
                .map(|(kw, _)| kw)
                .next()
        });

        Ok(suggestion.map(|s| format!("{}{}", prefix, s)))
    }
}

// --- Notes ---
#[derive(Debug, Clone)]
struct Note(PathBuf);

impl Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FrontMatter {
    title: String,
    date: String,
    file_tags: Vec<String>,
    indentifier: String,
}

fn main() -> InquireResult<()> {
    inquire::set_global_render_config(get_render_config());

    let opts = load_opts()?;
    let notes = load_notes(opts.note_dir.as_path())?;
    let keywords = load_key_words(&notes);

    // Figure out what mode we are in
    let args: Vec<String> = env::args().collect();

    if args.is_empty() {
        return Err(InquireError::InvalidConfiguration(
            "You must supply an argument either:\n-a for adding a command\n-s for searching a saved command".to_string(),
        ));
    }

    let mode = args[1].trim();

    // Run a prompt
    match mode {
        "--new" => {
            // Create new note
            let path = denote(&opts.note_dir, opts.notes_filetype, keywords)?;
            // Open editor
            open_with_editor(&path)?;
            Ok(())
        }
        "--find" => {
            // Find note
            let path = search_notes(&notes, keywords)?;
            // Open editor
            open_with_editor(&path)?;
            Ok(())
        }
        _ => Err(InquireError::InvalidConfiguration(
            "Incorrect Flag used".to_string(),
        )),
    }
}

// --- Load things ---
fn load_opts() -> Result<Opts, InquireError> {
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

fn load_notes(note_dir: &Path) -> Result<Vec<Note>, InquireError> {
    let mut notes: Vec<Note> = Vec::new();

    for entry in fs::read_dir(note_dir)? {
        let dir = entry?;
        notes.push(Note(dir.path()));
    }

    Ok(notes)
}

fn load_key_words(notes: &[Note]) -> Vec<String> {
    notes
        .iter()
        .filter_map(|p| p.0.file_stem()?.to_str())
        .flat_map(|name| name.split("__").skip(1).flat_map(|seg| seg.split('_')))
        .map(|tag| tag.to_string())
        .unique()
        .collect()
}

// --- Note generation ---
fn gen_time_id() -> String {
    Local::now().format("%Y%m%dT%H%M%S").to_string()
}

fn gen_date() -> String {
    Local::now().format("%F %a %R").to_string()
}

fn format_title(title: String) -> String {
    title.split_whitespace().join("-").to_string()
}

fn format_keywords(kw: String) -> String {
    kw.split(' ').map(str::trim).join("_").to_string()
}

fn option_split(input: &str) -> Option<()> {
    let parts: Vec<&str> = input.split(' ').collect();

    match parts.first() {
        Some(h)
            if h.contains(";")
                || h.contains("-")
                || h.contains("\t")
                || h.contains(",")
                || h.contains("_") =>
        {
            None
        }
        Some(_) => Some(()),
        None => None,
    }
}

// --- Note manipulation ---
fn search(notes: &[Note], keywords: Vec<String>) -> Vec<Note> {
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

// --- Prompts ---
fn denote(note_dir: &Path, ft: FileType, keywords: Vec<String>) -> Result<PathBuf, InquireError> {
    let identifier = gen_time_id();

    // Input validators
    let title_validator = |input: &str| match input.is_empty() {
        true => Ok(Validation::Invalid("You must provide a title".into())),
        false => Ok(Validation::Valid),
    };

    let kw_validator = |input: &str| match option_split(input) {
        Some(()) => Ok(Validation::Valid),
        None => Ok(Validation::Invalid(
            "Keywords must be space separated".into(),
        )),
    };

    // The prompt
    let title: String = Text::new("New file TITLE:")
        .with_validator(title_validator)
        .prompt()
        .unwrap();

    let keywords: String = Text::new("New file KEYWORDS:")
        .with_help_message("Each tag must be space separated, '-' and '_' are not allowed")
        .with_autocomplete(KeywordCompleter::new(keywords))
        .with_validator(kw_validator)
        .prompt()
        .unwrap();

    let fmt = FrontMatter {
        title: title.clone(),
        date: gen_date(),
        file_tags: keywords
            .clone()
            .split(' ')
            .map(|kw| kw.to_string())
            .collect(),
        indentifier: identifier.clone(),
    };

    let yaml = serde_yaml::to_string(&fmt)
        .map_err(|e| InquireError::InvalidConfiguration(e.to_string()))?;

    let note = match ft {
        FileType::Markdown => format!(
            "{}--{}__{}.md",
            identifier,
            format_title(title),
            format_keywords(keywords)
        ),
        FileType::Text => format!(
            "{}--{}__{}.txt",
            identifier,
            format_title(title),
            format_keywords(keywords)
        ),
        FileType::Org => format!(
            "{}--{}__{}.org",
            identifier,
            format_title(title),
            format_keywords(keywords)
        ),
    };

    // Create the new file
    let mut path = note_dir.to_path_buf();
    path.push(&note);

    let mut file: File = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    file.write_all(format!("---\n{}---\n", yaml).as_bytes())?;

    println!("Created Note: {}!", note);

    Ok(path)
}

fn search_notes(notes: &[Note], keywords: Vec<String>) -> Result<PathBuf, InquireError> {
    // Generate formatters
    let kw_formatter: MultiOptionFormatter<String> = &|a| {
        format!(
            "[{}]",
            a.iter()
                .map(|item| item.value.clone())
                .collect::<Vec<String>>()
                .join(" ")
        )
    };

    let note_formatter: OptionFormatter<Note> = &|a| {
        let formatted = a
            .value
            .0
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .map(|s| {
                let truncated = s.chars().take(30).collect::<String>();
                truncated.to_string()
            })
            .unwrap_or_else(|| "<invalid>".to_string());

        formatted
    };

    // Prompt
    let kws = MultiSelect::new("Select relavent keywords:", keywords)
        .with_formatter(kw_formatter)
        .prompt()
        .unwrap();

    let note = Select::new("Select note:", search(notes, kws))
        .with_formatter(note_formatter)
        .prompt()
        .unwrap();

    Ok(note.0)
}

// --- Editor functions ---
fn open_with_editor(path: &Path) -> std::io::Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    Command::new(editor).arg(path).status()?;

    Ok(())
}

// --- Rendering ---
fn get_render_config() -> RenderConfig<'static> {
    let mut render_config = RenderConfig::default();
    render_config.prompt_prefix = Styled::new("?").with_fg(Color::DarkMagenta);
    render_config.answered_prompt_prefix = Styled::new(">").with_fg(Color::DarkMagenta);
    render_config.highlighted_option_prefix = Styled::new(">").with_fg(Color::LightMagenta);
    render_config.selected_checkbox = Styled::new("[x]").with_fg(Color::LightMagenta);
    render_config.scroll_up_prefix = Styled::new("^");
    render_config.scroll_down_prefix = Styled::new("v");
    render_config.unselected_checkbox = Styled::new("[ ]");

    render_config.error_message = render_config
        .error_message
        .with_prefix(Styled::new("#").with_fg(Color::LightRed));

    render_config.answer = StyleSheet::new()
        .with_attr(Attributes::ITALIC)
        .with_fg(Color::LightMagenta);

    render_config.help_message = StyleSheet::new().with_fg(Color::DarkMagenta);

    render_config
}
