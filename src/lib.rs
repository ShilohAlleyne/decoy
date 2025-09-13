use colored::Colorize;
use inquire::{
    error::InquireResult,
    ui::{Attributes, Color, RenderConfig, StyleSheet, Styled},
    InquireError,
};
use std::env;

mod prompts;
mod ctx;
mod files;
mod options;


pub fn go() -> InquireResult<()> {
    // Set styling
    inquire::set_global_render_config(get_render_config());

    // Load deps
    let ctx = ctx::Ctx::new()?;

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
            let (path, front_matter) = prompts::denote(&ctx)?;

            // Write new note with front matter
            files::operations::write_new_note(&ctx, &path, front_matter)?;

            // Open editor
            files::operations::open_with(&ctx, &path)?;

            Ok(())
        }
        "--find" => {
            // Find note
            let path = prompts::search_notes_by_keywords(&ctx)?;

            // Open editor
            files::operations::open_with(&ctx, &path)?;

            Ok(())
}
        // Generate denote for already exisiting file
        "--rename" => {
            // Search old file
            let old_path = prompts::search_notes_by_keywords(&ctx)?;

            // Create new note
            let (new_path, _) = prompts::denote(&ctx)?;
            let new_name = new_path.file_stem().and_then(|name| name.to_str()).ok_or(
                InquireError::InvalidConfiguration("Invalid filename".to_string()),
            )?;

            // Rename file
            files::operations::rename_file(&old_path, new_name)?;
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
            let path = prompts::search_notes_by_date(&ctx)?;

            // Open editor
            files::operations::open_with(&ctx, &path)?;

            Ok(())
        }
        "--config" => {
            // open config
            files::operations::open_with(&ctx, &ctx.opts.opts_path)?;

            Ok(())
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
