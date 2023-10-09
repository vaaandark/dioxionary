use crate::stardict::Entry;
use crate::stardict::NotFoundError;
use crate::stardict::SearchAble;
use crossterm::terminal;
use dirs::home_dir;
use pulldown_cmark_mdcat::resources::{
    DispatchingResourceHandler, FileResourceHandler, ResourceUrlHandler,
};
use pulldown_cmark_mdcat::Settings;
use pulldown_cmark_mdcat::TerminalProgram;
use pulldown_cmark_mdcat::TerminalSize;
use pulldown_cmark_mdcat::Theme;
use pulldown_cmark_mdcat_ratatui::markdown_widget::PathOrStr;
use std::fs::File;
use std::io::stdout;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use syntect::parsing::SyntaxSet;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub struct Logseq {
    pub path: PathBuf,
}

impl SearchAble for Logseq {
    fn push_tty(&self, word: &str) -> anyhow::Result<()> {
        if let Some(p) = self.find_path(word) {
            let terminal = TerminalProgram::detect();
            let terminal_size = TerminalSize::detect().unwrap_or_default();
            let settings = Settings {
                terminal_capabilities: terminal.capabilities(),
                terminal_size,
                syntax_set: &SyntaxSet::load_defaults_newlines(),
                theme: Theme::default(),
            };
            mdcat::process_file(
                p.path().to_str().unwrap(),
                &settings,
                &FileResourceHandler::new(104_857_600),
                &mut mdcat::output::Output::Stdout(stdout()),
            )?;
            Ok(())
        } else {
            Err(NotFoundError.into())
        }
    }

    fn exact_lookup<'a>(&'a self, word: &str) -> Option<PathOrStr> {
        self.find_path(word)
            .map(|x| PathOrStr::from_md_path(x.path().to_owned()))
    }

    fn fuzzy_lookup<'a>(&'a self, target_word: &str) -> Vec<Entry<'a>> {
        match self.find(target_word) {
            Some(x) => vec![x],
            None => vec![],
        }
    }

    fn dict_name(&self) -> &str {
        "logseq"
    }
}

impl Logseq {
    fn find<'a>(&'a self, word: &str) -> Option<Entry<'a>> {
        self.find_path(word).map(|dir_entry| Entry {
            word: word.to_string(),
            trans: std::borrow::Cow::Owned(read_file_to_string(dir_entry.path())),
        })
    }

    fn find_path<'a>(&'a self, word: &str) -> Option<DirEntry> {
        let word = word.to_lowercase();
        let root = self.path.join("pages");
        'outer: for entry in WalkDir::new(root) {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let Some(file_name) = entry.file_name().to_str() else {
                continue;
            };
            if file_name.to_lowercase() == format!("{word}.md") {
                return Some(entry);
            }
            if let Some((_, file_name)) = file_name.rsplit_once("%2F")
                && file_name.to_lowercase() == format!("{word}.md")
            {
                return Some(entry);
            }

            let Ok(file) = File::open(path) else { continue };
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let Ok(line) = line else { continue 'outer };
                static ALIAS: &str = "alias:: ";
                if line.starts_with(ALIAS) {
                    let Some(line) = line.get(ALIAS.len()..) else {
                        unreachable!()
                    };
                    if line.split(',').any(|x| x.trim().to_lowercase() == word) {
                        return Some(entry);
                    }
                } else if line.starts_with("- ") {
                    break;
                } else {
                    // other property of page
                }
            }
        }
        None
    }
}

fn read_file_to_string(path: &Path) -> String {
    let contents = std::fs::read_to_string(path).unwrap();
    format!(
        "{}\n{contents}",
        path.file_name().unwrap().to_str().unwrap(),
    )
}
