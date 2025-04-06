pub mod llm;
pub mod offline;
pub mod online;
pub mod stardict;

use std::fmt;

#[derive(derive_more::Display)]
pub enum DictType {
    OnlineDict,
    StarDict,
    LLM,
}

pub trait Dict {
    fn name(&self) -> &str;
    fn type_(&self) -> DictType;
    fn supports_fuzzy_search(&self) -> bool;
    fn look_up(&self, enable_fuzzy: bool, word: &str) -> LookUpResult;
    fn word_count(&self) -> Option<usize>;
}

pub type DifficultyLevel = String;

pub struct LookUpResultItem {
    pub word: String,
    pub translation: String,
    pub difficulty_levels: Vec<DifficultyLevel>,
}

impl fmt::Display for LookUpResultItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut difficulty_levels_str = String::new();
        self.difficulty_levels
            .iter()
            .for_each(|x| difficulty_levels_str.push_str(&format!("<{}> ", x)));
        write!(
            f,
            "{}\n{}\n{}",
            self.word,
            self.translation.trim(),
            difficulty_levels_str
        )
    }
}

impl LookUpResultItem {
    pub fn new(word: impl Into<String>, translation: String) -> LookUpResultItem {
        LookUpResultItem {
            word: word.into(),
            translation,
            difficulty_levels: Vec::new(),
        }
    }

    pub fn new_with_difficulty_levels(
        word: String,
        translation: String,
        difficulty_levels: Vec<DifficultyLevel>,
    ) -> LookUpResultItem {
        LookUpResultItem {
            word,
            translation,
            difficulty_levels,
        }
    }
}

pub enum LookUpResult {
    Exact(LookUpResultItem),
    Fuzzy(Vec<LookUpResultItem>),
    None,
}
