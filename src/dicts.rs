#[cfg(feature = "pronunciation")]
use crate::pronunciation;
use crate::{
    dict::{offline::OfflineDict, online::OnlineDict, Dict, LookUpResult},
    history,
};
use anyhow::{Context, Result};
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use prettytable::{Attr, Cell, Row, Table};
use rustyline::error::ReadlineError;
use std::path::{Path, PathBuf};

pub struct DictManager {
    options: DictOptions,
    online_dict: Box<dyn Dict>,
    offline_dicts: Vec<Box<dyn Dict>>,
}

impl DictManager {
    pub fn new<P: AsRef<Path>>(offline_dict_path: Option<P>, options: DictOptions) -> Result<Self> {
        let offline_dicts = if let Some(offline_dict_path) = offline_dict_path {
            let path = offline_dict_path.as_ref();
            load_offline_dicts(path)?
        } else {
            vec![]
        };
        let online_dict = Box::new(OnlineDict) as Box<dyn Dict>;
        Ok(Self {
            online_dict,
            offline_dicts,
            options,
        })
    }

    fn look_up(&self, options: &DictOptions, word: &str) -> LookUpResult {
        let enable_fuzzy = !options.exact_match_only;

        let mut dicts = self.offline_dicts.iter().collect::<Vec<_>>();
        if options.prioritize_online_dict {
            dicts.insert(0, &self.online_dict);
        } else {
            dicts.push(&self.online_dict);
        }

        dicts
            .into_iter()
            .find_map(|dict| match dict.look_up(enable_fuzzy, word) {
                LookUpResult::None => None,
                result => Some(result),
            })
            .unwrap_or(LookUpResult::None)
    }

    pub fn repl(&self) {
        let mut rl = rustyline::DefaultEditor::new().unwrap();
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(word) => {
                    let _ = rl.add_history_entry(&word);
                    self.query(&word);
                }
                Err(ReadlineError::Interrupted) => break,
                Err(ReadlineError::Eof) => break,
                _ => {
                    eprintln!("Failed to read lines");
                    break;
                }
            }
        }
    }

    pub fn query(&self, word: &str) {
        let (options, word) = match DictOptions::parse_prefixed_word(word) {
            (Some(new_options), word) => (new_options, word),
            (None, word) => (self.options, word.to_owned()),
        };

        let results = self.look_up(&options, &word);
        let item = match results {
            LookUpResult::None => {
                eprintln!("Found nothing in the dictionaries");
                None
            }
            LookUpResult::Exact(item) => Some(item),
            LookUpResult::Fuzzy(items) => {
                println!("Fuzzy search enabled");
                if let Some(selection) = Select::with_theme(&ColorfulTheme::default())
                    .items(&items.iter().map(|w| w.word.as_str()).collect::<Vec<&str>>())
                    .default(0)
                    .interact_on_opt(&Term::stderr())
                    .unwrap()
                {
                    items.into_iter().nth(selection)
                } else {
                    None
                }
            }
        };

        if let Some(item) = item {
            println!("{}", item);
            if let Err(e) = history::insert_history_record(&item.word, item.difficulty_levels) {
                eprintln!("Failed to insert history record: {}", e);
            }
            #[cfg(feature = "pronunciation")]
            if options.read_aloud {
                if let Err(e) = pronunciation::pronounce(&item.word) {
                    eprintln!("Failed to read aloud: {}", e);
                }
            }
        }
    }

    pub fn list_dicts(&self) {
        let mut table: Table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Dictionary's name").with_style(Attr::Bold),
            Cell::new("Word count").with_style(Attr::Bold),
        ]));
        self.offline_dicts.iter().for_each(|dict| {
            let row = Row::new(vec![
                Cell::new(dict.name()),
                Cell::new(dict.word_count().to_string().as_str()),
            ]);
            table.add_row(row);
        });
        table.printstd();
    }
}

fn load_offline_dicts<P: AsRef<Path>>(offline_dict_dir: P) -> Result<Vec<Box<dyn Dict>>> {
    let path = offline_dict_dir.as_ref();
    let mut dicts: Vec<_> = path
        .read_dir()
        .with_context(|| format!("Failed to open configuration directory {:?}", path))?
        .filter_map(|x| x.ok())
        .collect();
    dicts.sort_by_key(|dict| dict.file_name());
    Ok(dicts
        .into_iter()
        .map(|dir| OfflineDict::new(dir.path()))
        .filter_map(|x| x.ok())
        .map(|dict| Box::new(dict) as Box<dyn Dict>)
        .collect())
}

#[derive(Debug, Clone, Copy)]
pub struct DictOptions {
    pub prioritize_online_dict: bool,
    pub prioritize_offline_dicts: bool,
    pub exact_match_only: bool,
    #[cfg(feature = "pronunciation")]
    pub read_aloud: bool,
}

fn split_non_alphanumeric_prefix(s: &str) -> (&str, &str) {
    for (i, c) in s.char_indices() {
        if c.is_alphanumeric() {
            return (&s[..i], &s[i..]);
        }
    }
    (s, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        assert_eq!(split_non_alphanumeric_prefix("123abc"), ("", "123abc"));
        assert_eq!(split_non_alphanumeric_prefix("!!hello"), ("!!", "hello"));
        assert_eq!(split_non_alphanumeric_prefix("abcdef"), ("", "abcdef"));
        assert_eq!(split_non_alphanumeric_prefix("***123"), ("***", "123"));
        assert_eq!(split_non_alphanumeric_prefix(""), ("", ""));
        assert_eq!(split_non_alphanumeric_prefix("á123"), ("", "á123"));
        assert_eq!(split_non_alphanumeric_prefix("&&&ábc"), ("&&&", "ábc"));
    }
}

impl Default for DictOptions {
    fn default() -> Self {
        Self {
            prioritize_online_dict: false,
            prioritize_offline_dicts: true,
            exact_match_only: false,
            #[cfg(feature = "pronunciation")]
            read_aloud: false,
        }
    }
}

impl DictOptions {
    fn parse_prefixed_word(word: &str) -> (Option<Self>, String) {
        let (prefix, word) = split_non_alphanumeric_prefix(word);
        if prefix.is_empty() {
            (None, word.to_owned())
        } else {
            let mut options = Self::default();
            if prefix.contains("@") {
                options.prioritize_online_dict = true;
            }
            if prefix.contains("//") {
                options.exact_match_only = false;
            }
            if prefix.contains("|") {
                options.exact_match_only = true;
            }
            #[cfg(feature = "pronunciation")]
            if prefix.contains("~") {
                options.read_aloud = true;
            }
            (Some(options), word.to_owned())
        }
    }

    pub fn prioritize_online(mut self, prioritize: bool) -> Self {
        self.prioritize_online_dict = prioritize;
        self
    }

    pub fn prioritize_offline(mut self, prioritize: bool) -> Self {
        self.prioritize_offline_dicts = prioritize;
        self
    }

    pub fn require_exact_match(mut self, exact: bool) -> Self {
        self.exact_match_only = exact;
        self
    }

    #[cfg(feature = "pronunciation")]
    pub fn read_aloud(mut self, read_aloud: bool) -> Self {
        self.read_aloud = read_aloud;
        self
    }
}

pub fn default_local_dict_path() -> Option<PathBuf> {
    let dioxionary_dir = dirs::config_dir()
        .map(|dir| dir.join("dioxionary"))
        .filter(|dir| dir.is_dir());

    let stardict_compatible_dir = dirs::home_dir()
        .map(|dir| dir.join(".stardict").join("dic"))
        .filter(|dir| dir.is_dir());

    match (&dioxionary_dir, &stardict_compatible_dir) {
        (Some(dir), _) => Some(dir.to_path_buf()),
        (None, Some(dir)) => Some(dir.to_path_buf()),
        (None, None) => None,
    }
}
