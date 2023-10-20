use crate::error::{Error, Result};
use chrono::Utc;
use dirs::cache_dir;
use prettytable::{Attr, Cell, Row, Table};
use rusqlite::Connection;
use std::fs::create_dir;
use std::path::PathBuf;

static ALLOWED_TYPES: [&str; 7] = ["CET4", "CET6", "TOEFL", "IELTS", "GMAT", "GRE", "SAT"];

#[allow(unused)]
pub fn check_cache() -> Result<PathBuf> {
    let mut path = cache_dir().ok_or(Error::CacheDirNotFound)?;
    path.push("dioxionary");
    if !path.exists() {
        create_dir(&path)?;
    }
    path.push("dioxionary.db");
    Ok(path)
}

#[allow(unused)]
pub fn add_history(word: &str, types: &Option<Vec<String>>) -> Result<()> {
    let date = Utc::now().timestamp();

    let path = check_cache()?;

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

    if let Some(types) = types {
        types.iter().for_each(|x| {
            if ALLOWED_TYPES.contains(&x.as_str()) {
                let sql = format!("UPDATE HISTORY SET {} = 1 WHERE WORD = '{}'", x, word);
                conn.execute(sql.as_str(), ());
            }
        })
    }

    Ok(())
}

#[allow(unused)]
pub fn list_history(type_: Option<String>, sort: bool, table: bool, column: usize) -> Result<()> {
    let path = check_cache()?;

    let mut stmt = "SELECT WORD, DATE FROM HISTORY".to_string();

    if let Some(type_) = type_ {
        if ALLOWED_TYPES.contains(&type_.as_str()) {
            stmt.push_str(format!(" WHERE {} = 1", type_).as_str())
        }
    }

    let conn = Connection::open(path)?;

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

pub fn count_history() -> Result<()> {
    let path = check_cache()?;

    let conn = Connection::open(path)?;

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
