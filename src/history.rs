//! History query and addition using [sqlite](https://sqlite.org/index.html).
use crate::spaced_repetition::SpacedRepetiton;
use anyhow::{Context, Result};
use dirs::data_dir;
use prettytable::{Attr, Cell, Row, Table};
use rusqlite::Connection;
use std::fs::create_dir;

/// Allowed diffculty level types of a word.
pub static ALLOWED_TYPES: [&str; 7] = ["CET4", "CET6", "TOEFL", "IELTS", "GMAT", "GRE", "SAT"];

/// Check and generate cache directory path.
pub fn get_db() -> Result<Connection> {
    let mut path = data_dir().with_context(|| "Couldn't find data directory")?;
    path.push("dioxionary");
    if !path.exists() {
        create_dir(&path).with_context(|| format!("Failed to create directory {:?}", path))?;
    }
    path.push("dioxionary.db");
    let db = Connection::open(path)?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS fsrs (
        word TEXT PRIMARY KEY,
        difficulty REAL NOT NULL,
        stability REAL NOT NULL,
        interval INTEGER NOT NULL,
        last_reviewed TEXT NOT NULL
        )",
        (), // empty list of parameters.
    )?;
    Ok(db)
}

/// Add a looked up word to history.
pub fn add_history(word: String) -> Result<()> {
    let mut d = crate::fsrs::Deck::default();
    d.add_fresh_word(word)?;
    Ok(())
    // crate::sm2::Deck::add_history(word)
}

/// List sorted or not history of a word type or all types.
///
/// The output will be like:
/// txt
/// +------+------+-------+-------+------+-----+-----+
/// | CET4 | CET6 | TOEFL | IELTS | GMAT | GRE | SAT |
/// +------+------+-------+-------+------+-----+-----+
/// | 220  | 305  | 207   | 203   | 142  | 242 | 126 |
/// +------+------+-------+-------+------+-----+-----+
///
pub fn list_history(type_: Option<String>, sort: bool, table: bool, column: usize) -> Result<()> {
    let mut stmt = "SELECT WORD, DATE FROM HISTORY".to_string();

    if let Some(type_) = type_ {
        if ALLOWED_TYPES.contains(&type_.as_str()) {
            stmt.push_str(format!(" WHERE {} = 1", type_).as_str())
        }
    }

    let conn = get_db()?;

    let mut stmt = conn.prepare(&stmt)?;
    let word_iter = stmt.query_map([], |row| row.get(0) as rusqlite::Result<String>)?;

    let mut words: Vec<String> = word_iter.filter_map(|x| x.ok()).collect();

    if sort {
        words.sort();
    }

    if table {
        let mut table = Table::new();
        words.chunks(column).for_each(|x| {
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

/// Count the history.
pub fn count_history() -> Result<()> {
    let conn = get_db()?;

    let header: Row = ALLOWED_TYPES
        .into_iter()
        .map(|x| Cell::new(x).with_style(Attr::Bold))
        .collect();

    let mut table: Table = Table::new();
    table.add_row(header);

    let body: Row = ALLOWED_TYPES
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
