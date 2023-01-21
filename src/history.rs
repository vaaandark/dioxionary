use chrono::Utc;
use dirs::cache_dir;
use prettytable::{Attr, Cell, Row, Table};
use rusqlite::{Connection, Result};
use std::fs::create_dir;
use std::path::PathBuf;

static ALLOWED_TYPES: [&str; 8] = [
    "CET4", "CET6", "CET8", "TOEFL", "IELTS", "GMAT", "GRE", "SAT",
];

#[allow(unused)]
pub fn check_cache() -> PathBuf {
    let mut cache = match cache_dir() {
        Some(dir) => dir,
        _ => {
            panic!("rmall: cannot create cache dir")
        }
    };
    cache.push("rmall");
    create_dir(&cache);
    cache
}

#[allow(unused)]
#[derive(Debug)]
struct Hist {
    word: String,
    date: i64,
}

#[allow(unused)]
pub fn add_history(word: &str, types: &Option<Vec<String>>) -> Result<()> {
    let date = Utc::now().timestamp();
    let hist = Hist {
        word: word.to_string(),
        date,
    };

    let mut path = check_cache();
    path.push("rmall.db");

    let conn = Connection::open(&path).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS HISTORY (
        WORD TEXT PRIMARY KEY,
        DATE INTEGER NOT NULL,
        CET4 INTEGER,
        CET6 INTEGER,
        CET8 INTEGER,
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
        (&hist.word, &hist.date),
    )?;

    if let Some(types) = types {
        types.iter().for_each(|x| {
            ALLOWED_TYPES.into_iter().for_each(|y| {
                if x == y {
                    let sql = format!("UPDATE HISTORY SET {} = 1 WHERE WORD = '{}'", x, word);
                    conn.execute(sql.as_str(), ());
                }
            })
        })
    }

    Ok(())
}

#[allow(unused)]
pub fn list_history(type_: Option<String>) -> Result<()> {
    let mut path = check_cache();
    path.push("rmall.db");

    // lack of error handling now
    // conside it as OK
    if !path.exists() {
        return Ok(());
    }

    let mut stmt = "SELECT word, date FROM HISTORY".to_string();

    if let Some(type_) = type_ {
        if ALLOWED_TYPES.contains(&type_.as_str()) {
            stmt.push_str(format!(" WHERE {} = 1", type_).as_str())
        }
    }

    let conn = Connection::open(&path)?;

    let mut stmt = conn.prepare(&stmt)?;
    let word_iter = stmt.query_map([], |row| {
        Ok(Hist {
            word: row.get(0)?,
            date: row.get(1)?,
        })
    })?;

    for w in word_iter {
        let h = w.unwrap();
        println!("{}", h.word);
    }

    Ok(())
}

pub fn count_history() -> Result<()> {
    let mut path = check_cache();
    path.push("rmall.db");

    // lack of error handling now
    // conside it as OK
    if !path.exists() {
        return Ok(());
    }

    let conn = Connection::open(&path)?;

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
            let res = stmt.query_row([], |row| row.get(0) as Result<i32>);
            Cell::new(&res.unwrap().to_string())
        })
        .collect();

    table.add_row(body);

    table.printstd();

    Ok(())
}
