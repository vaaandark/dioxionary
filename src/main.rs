mod dict;
mod history;
use std::process::exit;

use tokio;

#[tokio::main]
async fn main() {
    let word = match std::env::args().nth(1) {
        Some(arg) => arg,
        _ => "rust".to_string()
    };

    if word.as_str() == "-l" {
        if let Err(_) = history::list_history() {
            eprintln!("rmall: cannot list history");
            exit(1);
        };
        exit(0);
    }

    // is tran zh to en?
    let is_zh2en = !dict::is_enword(&word);

    let body = match dict::lookup(&word).await {
        Ok(body) => body,
        Err(e) => {
            panic!("rmall: {:?}", e)
        }
    };

    let meaning = dict::get_meaning(body, is_zh2en);
    println!("{}", meaning.trim());

    if !is_zh2en {
        if let Err(e) = history::add_history(word) {
            match e {
                // maybe the word has been looked up before
                rusqlite::Error::SqliteFailure(_, _) => {
                    exit(0);
                }
                _ => {
                    eprintln!("rmall: cannot add history");
                    exit(1);
                }
            }
        };
    }
}