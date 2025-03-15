//! History query and addition using [sqlite](https://sqlite.org/index.html).
use anyhow::{Context, Result};
use chrono::Utc;
use dirs::cache_dir;
use prettytable::{Attr, Cell, Row, Table};
use rusqlite::Connection;
use std::fs::create_dir;
use std::path::PathBuf;

use crate::dict::DifficultyLevel;

/// Allowed difficulty levels of a word.
pub static ALLOWED_DIFFICULTY_LEVELS: [&str; 7] =
    ["CET4", "CET6", "TOEFL", "IELTS", "GMAT", "GRE", "SAT"];

/// Check and generate cache directory path.
fn ensure_cache_directory() -> Result<PathBuf> {
    let mut path = cache_dir().with_context(|| "Couldn't find cache directory")?;
    path.push("dioxionary");
    if !path.exists() {
        create_dir(&path).with_context(|| format!("Failed to create directory {:?}", path))?;
    }
    path.push("dioxionary.db");
    Ok(path)
}

/// Inser history record.
pub fn insert_history_record(word: &str, difficulty_levels: Vec<DifficultyLevel>) -> Result<()> {
    let date = Utc::now().timestamp();

    let path = ensure_cache_directory()?;

    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS HISTORY (
        WORD TEXT PRIMARY KEY,
        DATE INTEGER NOT NULL,
        CET4 INTEGER,
        CET6 INTEGER,
        TOEFL INTEGER,
        IELTS INTEGER,
        GMAT INTEGER,
        GRE INTEGER,
        SAT INTEGER
        )",
        (), // empty list of parameters.
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO HISTORY (word, date) VALUES (?1, ?2)",
        (word, date),
    )?;

    difficulty_levels.iter().for_each(|x| {
        if ALLOWED_DIFFICULTY_LEVELS.contains(&x.as_str()) {
            let sql = format!("UPDATE HISTORY SET {} = 1 WHERE WORD = '{}'", x, word);
            conn.execute(sql.as_str(), ()).unwrap();
        }
    });

    Ok(())
}

/// List sorted or not history of a word difficulty level or all levels.
///
/// The output will be like:
/// txt
/// +------+------+-------+-------+------+-----+-----+
/// | CET4 | CET6 | TOEFL | IELTS | GMAT | GRE | SAT |
/// +------+------+-------+-------+------+-----+-----+
/// | 220  | 305  | 207   | 203   | 142  | 242 | 126 |
/// +------+------+-------+-------+------+-----+-----+
///
pub fn list_history_records(
    difficulty_level: Option<DifficultyLevel>,
    sort_alphabetically: bool,
    format_as_table: bool,
    max_column: usize,
) -> Result<()> {
    let path = ensure_cache_directory()?;

    let mut stmt = "SELECT WORD, DATE FROM HISTORY".to_string();

    if let Some(level) = difficulty_level {
        if ALLOWED_DIFFICULTY_LEVELS.contains(&level.as_str()) {
            stmt.push_str(format!(" WHERE {} = 1", level).as_str())
        }
    }

    let conn = Connection::open(path)?;

    let mut stmt = conn.prepare(&stmt)?;
    let word_iter = stmt.query_map([], |row| row.get(0) as rusqlite::Result<String>)?;

    let mut words: Vec<String> = word_iter.filter_map(|x| x.ok()).collect();

    if sort_alphabetically {
        words.sort();
    }

    if format_as_table {
        let mut table = Table::new();
        words.chunks(max_column).for_each(|x| {
            table.add_row(x.iter().map(|x| Cell::new(x)).collect());
        });
        table.printstd();
    } else {
        words.into_iter().for_each(|x| {
            println!("{}", x);
        });
    }

    Ok(())
}

/// Count history of a word difficulty level or all levels.
pub fn count_history_records() -> Result<()> {
    let path = ensure_cache_directory()?;

    let conn = Connection::open(path)?;

    let header: Row = ALLOWED_DIFFICULTY_LEVELS
        .into_iter()
        .map(|x| Cell::new(x).with_style(Attr::Bold))
        .collect();

    let mut table: Table = Table::new();
    table.add_row(header);

    let body: Row = ALLOWED_DIFFICULTY_LEVELS
        .into_iter()
        .map(|x| {
            let stmt = format!("SELECT COUNT(*) FROM HISTORY WHERE {} = 1", x);
            let mut stmt = conn.prepare(&stmt).unwrap();
            let res = stmt.query_row([], |row| row.get(0) as rusqlite::Result<i32>);
            Cell::new(&res.unwrap().to_string())
        })
        .collect();

    table.add_row(body);

    table.printstd();

    Ok(())
}
