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

    let word = dict::lookup(&word).await;
    println!("{}", word);

    if word.is_en() {
        match history::add_history(word.word()) {
            Ok(_) => (),
            // maybe the word has been looked up before
            Err(rusqlite::Error::SqliteFailure(_, _)) => (),
            Err(_) => {
                eprintln!("rmall: cannot add history");
                exit(1);
            }
        }
    }
}