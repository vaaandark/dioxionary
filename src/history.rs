use dirs::cache_dir;
use std::fs::create_dir;
use std::path::PathBuf;
use rusqlite::{Connection, Result};
use chrono::Utc;

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
pub fn add_history(word: String) -> Result<()> {
	let date = Utc::now().timestamp();
	let hist = Hist { word, date };

	let mut path = check_cache();
	path.push("rmall.db");

	let db_exist = path.exists();

    let conn = Connection::open(&path)?;
	if !db_exist {
		conn.execute(
			"CREATE TABLE HIST (
				word TEXT PRIMARY KEY,
				date INTEGER NOT NULL
			)",
			(), // empty list of parameters.
		)?;
	}
    conn.execute(
        "INSERT INTO HIST (word, date) VALUES (?1, ?2)",
        (&hist.word, &hist.date),
    )?;
	Ok(())
}

#[allow(unused)]
pub fn list_history() -> Result<()> {
	let mut path = check_cache();
	path.push("rmall.db");

	// lack of error handling now
	// conside it as OK
	if !path.exists() {
		return Ok(());
	}

    let conn = Connection::open(&path)?;

    let mut stmt = conn.prepare("SELECT word, date FROM HIST")?;
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