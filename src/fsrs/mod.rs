use crate::history;
use crate::spaced_repetition::SpacedRepetiton;
use anyhow::{Context, Result};
use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use fsrs::MemoryState;
use fsrs::DEFAULT_WEIGHTS;
use fsrs::FSRS;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::cell::LazyCell;
use std::str::FromStr;

pub mod review;

#[derive(Serialize, Deserialize, Debug)]
struct MemoryStateWrapper {
    stability: f32,
    difficulty: f32,
    interval: u32,
    last_reviewed: DateTime<Local>,
}

impl Default for MemoryStateWrapper {
    fn default() -> Self {
        Self {
            stability: DEFAULT_WEIGHTS[0],
            difficulty: DEFAULT_WEIGHTS[4] + 2.0 * DEFAULT_WEIGHTS[5],
            interval: 1,
            last_reviewed: Local::now(),
        }
    }
}

impl MemoryStateWrapper {
    fn next_review_time(&self) -> DateTime<Local> {
        self.last_reviewed + Duration::days(self.interval.into())
    }

    fn to_memory_state(&self) -> MemoryState {
        MemoryState {
            stability: self.stability,
            difficulty: self.difficulty,
        }
    }
}

#[derive(Debug)]
pub struct Deck {
    fsrs: LazyCell<FSRS>,
    conn: LazyCell<Connection>,
}

impl Default for Deck {
    fn default() -> Self {
        Self {
            fsrs: LazyCell::new(|| FSRS::new(Some(&DEFAULT_WEIGHTS)).unwrap()),
            conn: LazyCell::new(|| history::get_db().unwrap()),
        }
    }
}

impl SpacedRepetiton for Deck {
    fn next_to_review(&self) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT word, stability, difficulty, interval, last_reviewed FROM fsrs")?;
        let person_iter = stmt.query_map([], |row| {
            let time: String = row.get(4)?;
            let sm = MemoryStateWrapper {
                stability: row.get(1)?,
                difficulty: row.get(2)?,
                interval: row.get(3)?,
                last_reviewed: DateTime::<Local>::from_str(&time).unwrap(),
            };
            let word = row.get(0)?;
            Ok((word, sm))
        })?;
        for (word, sm) in person_iter.flatten() {
            if sm.next_review_time() <= Local::now() {
                return Ok(Some(word));
            }
        }
        Ok(None)
    }

    fn add_fresh_word(&mut self, word: String) -> Result<()> {
        insert_if_not_exists(&self.conn, &word, Default::default())?;
        Ok(())
    }

    /// requires 1 <= q <= 4
    fn update(&mut self, question: String, q: u8) -> Result<()> {
        let old_state = get_word(&self.conn, &question)?;
        let next_states = self.fsrs.next_states(
            Some(old_state.to_memory_state()),
            0.9,
            (Local::now() - old_state.last_reviewed)
                .num_days()
                .abs()
                .try_into()?,
        )?;
        let new_memory_state = match q {
            1 => next_states.again,
            2 => next_states.hard,
            3 => next_states.good,
            4 => next_states.easy,
            _ => unreachable!(),
        };
        let x = MemoryStateWrapper {
            stability: new_memory_state.memory.stability,
            difficulty: new_memory_state.memory.difficulty,
            interval: new_memory_state.interval,
            last_reviewed: Local::now(),
        };
        insert(&self.conn, &question, x)?;
        Ok(())
    }

    fn remove(&mut self, question: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM fsrs WHERE word = ?", [question])?;
        Ok(())
    }
}

fn insert_if_not_exists(conn: &Connection, word: &str, sm: MemoryStateWrapper) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO fsrs (word, stability, difficulty, interval, last_reviewed) VALUES (?1, ?2, ?3, ?4, ?5)",
        (word, sm.stability, sm.difficulty, sm.interval, sm.last_reviewed.to_string()),
    )?;
    Ok(())
}

fn insert(conn: &Connection, word: &str, sm: MemoryStateWrapper) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO fsrs (word, stability, difficulty, interval, last_reviewed) VALUES (?1, ?2, ?3, ?4, ?5)",
        (word, sm.stability, sm.difficulty, sm.interval, sm.last_reviewed.to_string()),
    )?;
    Ok(())
}

fn get_word(conn: &Connection, word: &str) -> Result<MemoryStateWrapper> {
    let sm = conn.query_row(
        "SELECT stability, difficulty, interval, last_reviewed FROM fsrs WHERE word = ?",
        [word],
        |row| {
            let time: String = row.get(3)?;
            let sm = MemoryStateWrapper {
                stability: row.get(0)?,
                difficulty: row.get(1)?,
                interval: row.get(2)?,
                last_reviewed: DateTime::<Local>::from_str(&time).unwrap(),
            };
            Ok(sm)
        },
    )?;
    Ok(sm)
}
