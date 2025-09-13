use chrono::Local;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use inquire::{
    formatter::{MultiOptionFormatter, OptionFormatter},
    validator::Validation,
    Autocomplete, DateSelect, InquireError, MultiSelect, Select, Text,
};
use itertools::Itertools;
use std::path::PathBuf;

use crate::{
    ctx,
    files::{frontmatter, note},
};

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

// --- Prompts ---
// Generate note with denote formmatter
pub(crate) fn denote(ctx: &ctx::Ctx) -> Result<(PathBuf, frontmatter::FrontMatter), InquireError> {
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

    // Note generation
    let gen_time_id = || Local::now().format("%Y%m%dT%H%M%S").to_string();

    let gen_date = || Local::now().format("%F %a %R").to_string();

    let format_title = |title: String| title.split_whitespace().join("-").to_string();

    let format_keywords = |kw: String| {
        if !kw.is_empty() {
            return format!("__{}", kw.split(' ').map(str::trim).join("_"));
        }

        "".to_string()
    };

    let identifier = gen_time_id();

    // The prompt
    let title: String = Text::new("New file TITLE:")
        .with_validator(title_validator)
        .prompt()
        .unwrap();

    let keywords: String = Text::new("New file KEYWORDS:")
        .with_help_message("↑↓ to move, <TAB> to autocomplete, type to filter, Tags are space separated and cannot contain '_' or '-'")
        .with_autocomplete(KeywordCompleter::new(ctx.keywords.clone()))
        .with_validator(kw_validator)
        .prompt()
        .unwrap();

    let fmt = frontmatter::FrontMatter {
        title: title.clone(),
        date: gen_date(),
        file_tags: keywords
            .clone()
            .split(' ')
            .map(|kw| kw.to_string())
            .collect(),
        indentifier: identifier.clone(),
    };

    let note = format!(
        "{}--{}{}{}",
        identifier,
        format_title(title),
        format_keywords(keywords),
        &ctx.opts.notes_filetype.as_ref()
    );

    // Create the new file
    let mut path = ctx.opts.note_dir.to_path_buf();
    path.push(&note);

    Ok((path, fmt))
}

// Search notes by keywords
pub(crate) fn search_notes_by_keywords(ctx: &ctx::Ctx) -> Result<PathBuf, InquireError> {
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

    let note_formatter: OptionFormatter<note::Note> = &|a| {
        let formatted = a
            .value
            .0
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "<invalid>".to_string());

        formatted
    };

    // Prompt
    let kws = MultiSelect::new("Select relavent keywords:", ctx.keywords.clone())
        .with_formatter(kw_formatter)
        .prompt()
        .unwrap();

    let note = Select::new("Select note:", note::search_by_keywords(&ctx.notes, kws))
        .with_formatter(note_formatter)
        .prompt()
        .unwrap();

    Ok(note.0)
}

pub(crate) fn search_notes_by_date(ctx: &ctx::Ctx) -> Result<PathBuf, InquireError> {
    let note_formatter: OptionFormatter<note::Note> = &|a| {
        let formatted = a
            .value
            .0
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "<invalid>".to_string());

        formatted
    };

    let date = DateSelect::new("Selected date")
        .with_default(Local::now().date_naive())
        .with_week_start(chrono::Weekday::Mon)
        .with_help_message("Use the arrow keys to select date")
        .prompt()
        .unwrap();

    let note = Select::new("Select note:", note::search_by_date(&ctx.notes, date))
        .with_formatter(note_formatter)
        .prompt()
        .unwrap();

    Ok(note.0)
}
