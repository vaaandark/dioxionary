use anyhow::{Context, Result};
use chrono::{prelude::*, Duration};
use dirs::data_dir;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fs, path::PathBuf};

use crate::spaced_repetition::SpacedRepetiton;

pub mod review;

#[derive(Serialize, Deserialize, Debug)]
pub struct Sm {
    /// the number of times the card has been successfully recalled in a row since the last time it was not.
    n: u32,
    /// how "easy" the card is (more precisely, it determines how quickly the inter-repetition interval grows)
    ef: f32,
    ///  is the length of time (in days) SuperMemo will wait after the previous review before asking the user to review the card again.
    interval: u32,

    last_reviewed: DateTime<Local>,
}

impl Default for Sm {
    // for init (new word)
    fn default() -> Self {
        Self {
            n: 0,
            ef: 2.5,
            interval: 1,
            last_reviewed: Local::now(),
        }
    }
}

impl Sm {
    fn next_review_time(&self) -> DateTime<Local> {
        self.last_reviewed + Duration::days(self.interval.into())
    }

    // requires{0 <= user_grade <= 5}
    fn sm2(&self, user_grade: u8) -> Self {
        let n: u32;
        #[allow(non_snake_case)]
        let I: u32;

        if user_grade >= 3 {
            if self.n == 0 {
                I = 1;
            } else if self.n == 1 {
                I = 6;
            } else {
                I = (self.interval as f32 * self.ef).round() as u32;
            }
            n = self.n + 1;
        } else {
            I = 1;
            n = 0;
        }

        let mut ef =
            self.ef + (0.1 - (5 - user_grade) as f32 * (0.08 + ((5 - user_grade) as f32) * 0.02));
        if ef < 1.3 {
            ef = 1.3;
        }
        Self {
            n,
            ef,
            interval: I,
            last_reviewed: Local::now(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Deck(pub HashMap<String, Sm>);

impl Deck {
    fn load_inner() -> Result<Self> {
        let path = get_json_location()?;
        let contents = std::fs::read_to_string(path)?;
        let hm: HashMap<String, Sm> = serde_json::from_str(&contents)?;
        Ok(Self(hm))
    }

    #[cfg(test)]
    fn fake_data() -> Self {
        Self(
            [(
                "hello".to_owned(),
                Sm {
                    n: 1,
                    ef: 2.5,
                    interval: 1,
                    last_reviewed: "2014-11-28T12:00:09Z".parse::<DateTime<Local>>().unwrap(),
                },
            )]
            .into_iter()
            .collect(),
        )
    }
}

impl SpacedRepetiton for Deck {
    fn next_to_review(&self) -> Option<String> {
        for (k, v) in &self.0 {
            if v.next_review_time() <= Local::now() {
                return Some(k.to_owned());
            }
        }
        None
    }

    fn dump(&self) -> Result<()> {
        let json_string = serde_json::to_string(&self.0)?;
        let path = get_json_location()?;
        fs::write(path, json_string)?;
        Ok(())
    }

    fn load() -> Self {
        match Self::load_inner() {
            Ok(s) => s,
            Err(_) => Self::default(),
        }
    }

    fn add_fresh_word(&mut self, word: String) {
        match self.0.entry(word) {
            std::collections::hash_map::Entry::Occupied(_) => {}
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(Sm::default());
            }
        }
    }

    // requires{0 <= q <= 5}
    fn update(&mut self, question: String, q: u8) {
        let sm = self.0[&question].sm2(q);
        let Some(__) = self.0.insert(question, sm) else {
            unreachable!()
        };
    }

    fn remove(&mut self, question: &str) {
        self.0.remove(question);
    }
}

/// Check and generate cache directory path.
fn get_json_location() -> Result<PathBuf> {
    let mut path = data_dir().with_context(|| "Couldn't find cache directory")?;
    path.push("dioxionary");
    if !path.exists() {
        std::fs::create_dir(&path)
            .with_context(|| format!("Failed to create directory {:?}", path))?;
    }
    path.push("sm2.json");
    Ok(path)
}
