use colored::Colorize;
use inquire::{
    error::InquireResult,
    ui::{Attributes, Color, RenderConfig, StyleSheet, Styled},
    InquireError,
};
use std::env;

pub mod file_ops;
pub mod options;
pub mod prompts;

pub fn go() -> InquireResult<()> {
    // Set styling
    inquire::set_global_render_config(get_render_config());

    // Load deps
    let opts = options::load_opts()?;
    let notes = file_ops::load_notes(opts.note_dir.as_path())?;
    let keywords = file_ops::load_key_words(&notes);
    let defualt_ft = match opts.notes_filetype {
        options::FileType::Markdown => ".md",
        options::FileType::Text => ".txt",
        options::FileType::Org => ".org",
        options::FileType::Typst => ".typ",
    };

    //  --- Super basic arg parsing ---
    let args: Vec<String> = env::args()
        .skip(1) // Skip the binary name
        .map(|arg| arg.trim().to_owned())
        .collect();

    if args.is_empty() {
        return Err(InquireError::InvalidConfiguration(
            "You must supply an argument: use --help for argument list".to_string(),
        ));
    }

    let mode = args[0].trim();

    // Run a prompt
    match mode {
        "--new" => {
            // Create new note
            let (path, front_matter) = prompts::denote(&opts.note_dir, defualt_ft, keywords)?;

            // Write new note with front matter
            file_ops::write_new_note(&path, front_matter, opts.notes_filetype)?;

            // Open editor
            file_ops::open_with_editor(&path)?;

            Ok(())
        }
        "--find" => {
            // Find note
            let path = prompts::search_notes(&notes, keywords)?;

            // Open editor
            file_ops::open_with_editor(&path)?;

            Ok(())
        }
        // Generate denote for already exisiting file
        "--rename" => {
            // Search old file
            let mut old_path = prompts::search_notes(&notes, keywords.clone())?;
            let ext = old_path
                .extension()
                .and_then(|ext| ext.to_str().map(|s| format!(".{}", s)))
                .ok_or_else(|| {
                    InquireError::InvalidConfiguration(
                        "Missing or invalid file extension".to_string(),
                    )
                })?;

            // Create new note
            let (new_path, _) = prompts::denote(&opts.note_dir, &ext, keywords)?;
            let new_name = new_path.file_name().and_then(|name| name.to_str()).ok_or(
                InquireError::InvalidConfiguration("Invalid filename".to_string()),
            )?;

            // Rename file
            file_ops::rename_file(&mut old_path, new_name)?;
            println!(
                "{} Renamed file: {:?} -> {}",
                ">".magenta(),
                old_path,
                new_name.italic().magenta(),
            );

            Ok(())
        }
        "--date" => {
            // Search old file
            let path = prompts::search_by_date(&notes)?;

            // Open editor
            file_ops::open_with_editor(&path)?;

            Ok(())
        }
        "--config" => {
            // open config
            let opts_path = options::get_opts_path();
            if opts_path.exists() {
                file_ops::open_with_editor(&opts_path)?;

                Ok(())
            } else {
                options::generate_default_opts()?;
                file_ops::open_with_editor(&opts_path)?;

                Ok(())
            }
        }
        _ => Err(InquireError::InvalidConfiguration(
            "Incorrect Flag used".to_string(),
        )),
    }
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
