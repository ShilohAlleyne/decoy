use inquire::InquireError;

use crate::{files::note, options::opts};

#[derive(Debug, Default)]
pub(crate) struct Ctx {
    pub opts: opts::Opts,
    pub notes: Vec<note::Note>,
    pub keywords: Vec<String>,
}

impl Ctx {
    pub fn new() -> Result<Self, InquireError> {
        let opts = opts::load()?;
        let notes = note::load(&opts.note_dir)?;

        Ok(Self {
            opts,
            keywords: note::parse_all_keywords(&notes),
            notes,
        })
    }
}

pub(crate) fn with_ctx<F, R>(ctx: Ctx, f: F) -> R
where
    F: FnOnce(&Ctx) -> R,
{
    f(&ctx)
}
