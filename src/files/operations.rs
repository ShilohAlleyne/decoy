use std::{
    fs::{self, create_dir_all, File, OpenOptions},
    io::{Error, Write},
    path::Path,
    process::Command,
};

use crate::{
    ctx,
    files::{frontmatter, types},
};

// --- File manipulation ---
pub fn write_new_note(
    ctx: &ctx::Ctx,
    path: &Path,
    frontmatter: frontmatter::FrontMatter,
) -> std::io::Result<()> {
    let fm = match ctx.opts.notes_filetype {
        types::FileType::Org => frontmatter::to_org_front_matter(frontmatter).into_bytes(),
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
    if !(ctx.opts.notes_filetype == types::FileType::Typst) {
        file.write_all(&fm)?;
    }

    Ok(())
}

pub(crate) fn rename_file(original: &Path, new_stem: &str) -> std::io::Result<()> {
    let ext = original.extension().and_then(|e| e.to_str());

    let new_name = match ext {
        Some(e) => format!("{}.{}", new_stem, e),
        None => new_stem.to_string(),
    };

    let new_path = original.with_file_name(new_name);
    fs::rename(original, new_path)
}

pub(crate) fn open_with(ctx: &ctx::Ctx, path: &Path) -> std::io::Result<()> {
    // figure out what filetype we are opening
    let editor = match path.extension().and_then(|ext| ext.to_str()) {
        Some("pdf") => ctx.opts.editor.pdf_viewer.to_owned(),
        _ => ctx.opts.editor.text_editor.to_owned(),
    };

    println!("{}", editor);
    // env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    Command::new(editor).arg(path).status()?;

    Ok(())
}
