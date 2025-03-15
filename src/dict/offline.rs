use std::path::Path;

use super::{stardict::StarDict, Dict, LookUpResult, LookUpResultItem};
use anyhow::{Context, Result};

pub struct OfflineDict {
    name: String,
    stardict: StarDict,
}

impl OfflineDict {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self> {
        let path = path.as_ref();
        let stardict = StarDict::new(path)
            .with_context(|| format!("Failed to open dictionary {}", path.display()))?;
        let name = stardict.dict_name().to_owned();
        Ok(Self { name, stardict })
    }
}

impl Dict for OfflineDict {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_online(&self) -> bool {
        false
    }

    fn supports_fuzzy_search(&self) -> bool {
        true
    }

    fn look_up(&self, enable_fuzzy: bool, word: &str) -> LookUpResult {
        if let Some(result) = self.stardict.exact_look_up(word) {
            let word = result.word.to_owned();
            let translation = result.translation.to_owned();
            LookUpResult::Exact(LookUpResultItem::new(word, translation))
        } else if enable_fuzzy {
            if let Some(results) = self.stardict.fuzzy_look_up(word) {
                LookUpResult::Fuzzy(
                    results
                        .iter()
                        .map(|result| {
                            let word = result.word.to_owned();
                            let translation = result.translation.to_owned();
                            LookUpResultItem::new(word, translation)
                        })
                        .collect(),
                )
            } else {
                LookUpResult::None
            }
        } else {
            LookUpResult::None
        }
    }

    fn word_count(&self) -> usize {
        self.stardict.word_count()
    }
}
