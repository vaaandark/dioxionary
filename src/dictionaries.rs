use crate::dictionary::{offline::OfflineDict, online::OnlineDict, Dict, LookUpResult};
use anyhow::{anyhow, Context, Result};
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use prettytable::{Attr, Cell, Row, Table};
use rustyline::error::ReadlineError;
use std::path::{Path, PathBuf};

pub struct DictionaryManager {
    options: DictionaryOptions,
    online_dict: Box<dyn Dict>,
    offline_dicts: Vec<Box<dyn Dict>>,
}

impl DictionaryManager {
    pub fn new<P: AsRef<Path>>(offline_dict_path: P, options: DictionaryOptions) -> Result<Self> {
        let path = offline_dict_path.as_ref();
        let offline_dicts = load_offline_dictionaries(path)?;
        let online_dict = Box::new(OnlineDict) as Box<dyn Dict>;
        Ok(Self {
            online_dict,
            offline_dicts,
            options,
        })
    }

    fn look_up(&self, word: &str) -> LookUpResult {
        let mut options = self.options;
        let mut word = word.to_owned();
        if let (Some(options_), word_) = DictionaryOptions::parse_prefixed_word(&word) {
            word = word_;
            options = options_;
        }

        let mut dicts: Vec<&Box<dyn Dict>> = Vec::with_capacity(self.offline_dicts.len() + 1);
        if options.prioritize_online_dictionary {
            dicts.push(&self.online_dict);
            dicts.extend(self.offline_dicts.iter());
        } else if options.prioritize_offline_dictionaries {
            dicts.extend(self.offline_dicts.iter());
            dicts.push(&self.online_dict);
        }

        let mut enable_fuzzy = true;
        if options.exact_match_only {
            enable_fuzzy = false;
        }
        for dict in dicts {
            let result = dict.look_up(enable_fuzzy, &word);
            match result {
                LookUpResult::None => continue,
                found => return found,
            }
        }
        LookUpResult::None
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
        let results = self.look_up(word);
        match results {
            LookUpResult::None => {
                eprintln!("Found nothing in the dictionaries");
            }
            LookUpResult::Exact(item) => {
                println!("{}", item);
            }
            LookUpResult::Fuzzy(items) => {
                println!("Fuzzy search enabled");
                if let Some(selection) = Select::with_theme(&ColorfulTheme::default())
                    .items(&items.iter().map(|w| w.word.as_str()).collect::<Vec<&str>>())
                    .default(0)
                    .interact_on_opt(&Term::stderr())
                    .unwrap()
                {
                    if let Some(item) = items.get(selection) {
                        println!("{}", item);
                    }
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

fn load_offline_dictionaries<P: AsRef<Path>>(offline_dict_dir: P) -> Result<Vec<Box<dyn Dict>>> {
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
pub struct DictionaryOptions {
    pub prioritize_online_dictionary: bool,
    pub prioritize_offline_dictionaries: bool,
    pub exact_match_only: bool,
}

fn split_non_alphanumberic_prefix(s: &str) -> (&str, &str) {
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
        assert_eq!(split_non_alphanumberic_prefix("123abc"), ("", "123abc"));
        assert_eq!(split_non_alphanumberic_prefix("!!hello"), ("!!", "hello"));
        assert_eq!(split_non_alphanumberic_prefix("abcdef"), ("", "abcdef"));
        assert_eq!(split_non_alphanumberic_prefix("***123"), ("***", "123"));
        assert_eq!(split_non_alphanumberic_prefix(""), ("", ""));
        assert_eq!(split_non_alphanumberic_prefix("á123"), ("", "á123"));
        assert_eq!(split_non_alphanumberic_prefix("&&&ábc"), ("&&&", "ábc"));
    }
}

impl Default for DictionaryOptions {
    fn default() -> Self {
        Self {
            prioritize_online_dictionary: false,
            prioritize_offline_dictionaries: true,
            exact_match_only: false,
        }
    }
}

impl DictionaryOptions {
    fn parse_prefixed_word(word: &str) -> (Option<Self>, String) {
        let (prefix, word) = split_non_alphanumberic_prefix(word);
        if prefix.is_empty() {
            (None, word.to_owned())
        } else {
            let mut options = Self::default();
            if prefix.contains("@") {
                options.prioritize_online_dictionary = true;
            }
            if prefix.contains("//") {
                options.exact_match_only = false;
            }
            if prefix.contains("|") {
                options.exact_match_only = true;
            }
            (Some(options), word.to_owned())
        }
    }

    pub fn priortize_online(mut self, prioritize: bool) -> Self {
        self.prioritize_online_dictionary = prioritize;
        self
    }

    pub fn prioritize_offline(mut self, prioritize: bool) -> Self {
        self.prioritize_offline_dictionaries = prioritize;
        self
    }

    pub fn require_exact_match(mut self, exact: bool) -> Self {
        self.exact_match_only = exact;
        self
    }
}

pub fn default_local_dict_path() -> Result<PathBuf> {
    let dioxionary_dir = dirs::config_dir()
        .map(|dir| dir.join("dioxionary"))
        .filter(|dir| dir.is_dir());

    let stardict_compatible_dir = dirs::home_dir()
        .map(|dir| dir.join(".stardict").join("dic"))
        .filter(|dir| dir.is_dir());

    match (&dioxionary_dir, &stardict_compatible_dir) {
        (Some(dir), _) => Ok(dir.to_path_buf()),
        (None, Some(dir)) => Ok(dir.to_path_buf()),
        (None, None) => Err(anyhow!("Couldn't find configuration directory")),
    }
}
