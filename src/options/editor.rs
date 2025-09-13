use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Editor {
    #[serde(default = "default_text_editor")]
    pub text_editor: String,
    #[serde(default = "default_pdf_viewer")]
    pub pdf_viewer: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            // Use the default text editor in env
            text_editor: env::var("EDITOR").unwrap_or_else(|_| "nano".to_string()),
            pdf_viewer: "zathura".to_string(),
        }
    }
}

fn default_text_editor() -> String {
    env::var("EDITOR").unwrap_or_else(|_| "nano".to_string())
}

fn default_pdf_viewer() -> String {
   "zathura".to_string()
}
