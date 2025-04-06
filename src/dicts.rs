#[cfg(feature = "pronunciation")]
use crate::pronunciation;
use crate::{
    dict::{
        llm::LlmDict, offline::OfflineDict, online::OnlineDict, Dict, LookUpResult,
        LookUpResultItem,
    },
    history,
};
use anyhow::{Context, Result};
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use prettytable::{Attr, Cell, Row, Table};
use rustyline::error::ReadlineError;
use std::path::{Path, PathBuf};

pub struct DictManager {
    options: DictOptions,
    online_dicts: Vec<Box<dyn Dict>>,
    offline_dicts: Vec<Box<dyn Dict>>,
    llm_dicts: Vec<Box<dyn Dict>>,
}

impl DictManager {
    pub fn new<P: AsRef<Path>>(
        offline_dict_path: Option<P>,
        llm_dict_config_path: Option<P>,
        options: DictOptions,
    ) -> Result<Self> {
        let offline_dicts = if let Some(offline_dict_path) = offline_dict_path {
            let path = offline_dict_path.as_ref();
            load_offline_dicts(path)?
        } else {
            vec![]
        };

        let online_dict = vec![Box::new(OnlineDict) as Box<dyn Dict>];

        let llm_dicts = if let Some(llm_dict_config_path) = llm_dict_config_path {
            let path = llm_dict_config_path.as_ref();
            load_llm_dicts(path)?
        } else {
            vec![]
        };

        Ok(Self {
            online_dicts: online_dict,
            offline_dicts,
            llm_dicts,
            options,
        })
    }

    fn find_exact_match<'a>(
        &'a self,
        mut dicts: impl Iterator<Item = &'a &'a Box<dyn Dict>>,
        word: &str,
    ) -> Option<LookUpResultItem> {
        dicts.find_map(|dict| {
            let result = dict.look_up(false, word);
            match result {
                LookUpResult::Exact(item) => Some(item),
                _ => {
                    eprintln!("Failed to look up `{}` in dict {}", word, dict.name(),);
                    None
                }
            }
        })
    }

    // TODO: fuzzily look up all dictionaries and rank them
    fn find_fuzzy_matches<'a>(
        &'a self,
        mut dicts: impl Iterator<Item = &'a &'a Box<dyn Dict>>,
        word: &str,
    ) -> Vec<LookUpResultItem> {
        dicts
            .find_map(|dict| {
                if !dict.supports_fuzzy_search() {
                    return None;
                }
                let result = dict.look_up(true, word);
                match result {
                    LookUpResult::Exact(item) => Some(vec![item]),
                    LookUpResult::Fuzzy(items) => Some(items),
                    LookUpResult::None => {
                        eprintln!(
                            "Failed to fuzzily look up `{}` in dict {}",
                            word,
                            dict.name(),
                        );
                        None
                    }
                }
            })
            .unwrap_or_default()
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

        let enable_fuzzy = !options.exact_match_only;

        let dicts = if options.prioritize_online_dict {
            self.online_dicts.iter().chain(&self.offline_dicts)
        } else {
            self.offline_dicts.iter().chain(&self.online_dicts)
        };

        let dicts: Vec<_> = if options.use_llm_dicts {
            self.llm_dicts.iter().chain(dicts).collect()
        } else {
            dicts.chain(&self.llm_dicts).collect()
        };

        let item = if let Some(exact_result) = self.find_exact_match(dicts.iter(), &word) {
            Some(exact_result)
        } else if enable_fuzzy {
            println!("Fuzzy search enabled");
            let fuzzy_results = self.find_fuzzy_matches(dicts.iter(), &word);
            if let Some(selection) = Select::with_theme(&ColorfulTheme::default())
                .items(
                    &fuzzy_results
                        .iter()
                        .map(|w| w.word.as_str())
                        .collect::<Vec<&str>>(),
                )
                .default(0)
                .interact_on_opt(&Term::stderr())
                .unwrap()
            {
                fuzzy_results.into_iter().nth(selection)
            } else {
                None
            }
        } else {
            None
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
        } else {
            eprintln!("No result found");
        }
    }

    pub fn list_dicts(&self) {
        let mut table: Table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Dictionary's name").with_style(Attr::Bold),
            Cell::new("Type").with_style(Attr::Bold),
            Cell::new("Word count").with_style(Attr::Bold),
        ]));

        self.offline_dicts
            .iter()
            .chain(&self.online_dicts)
            .chain(&self.llm_dicts)
            .for_each(|dict| {
                let row = Row::new(vec![
                    Cell::new(dict.name()),
                    Cell::new(dict.type_().to_string().as_str()),
                    Cell::new(
                        dict.word_count()
                            .map(|n| n.to_string())
                            .unwrap_or("-".to_owned())
                            .as_str(),
                    ),
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

pub fn load_llm_dicts<P: AsRef<Path>>(path: P) -> Result<Vec<Box<dyn Dict>>> {
    let content = std::fs::read_to_string(path)?;
    let config: toml::Value = content.parse()?;

    let dicts = config["service"]
        .as_array()
        .with_context(|| "Invalid config format")?;

    let mut result = Vec::new();
    for entry in dicts {
        let dict: Box<dyn Dict> = entry.clone().try_into::<LlmDict>().map(Box::new)?;
        result.push(dict);
    }

    Ok(result)
}

#[derive(Debug, Clone, Copy)]
pub struct DictOptions {
    pub prioritize_online_dict: bool,
    pub prioritize_offline_dicts: bool,
    pub use_llm_dicts: bool,
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
            use_llm_dicts: false,
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
            if prefix.contains("%") {
                options.use_llm_dicts = true;
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

    pub fn use_llm_dicts(mut self, use_llm: bool) -> Self {
        self.use_llm_dicts = use_llm;
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

pub fn default_llm_dict_config_path() -> Option<PathBuf> {
    let dioxionary_dir = dirs::config_dir()
        .map(|dir| dir.join("dioxionary"))
        .filter(|dir| dir.is_dir());

    match dioxionary_dir {
        Some(dir) => {
            let llm_config_path = dir.join("llm.toml");
            if llm_config_path.exists() {
                Some(llm_config_path)
            } else {
                None
            }
        }
        None => None,
    }
}
