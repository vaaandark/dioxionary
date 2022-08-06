mod dict;
mod history;
use std::process::exit;
use dict::Html;

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

    let html = Html::parse_document(&body);
    if is_zh2en {
        let meaning = dict::zh2en(&html);
        if meaning.trim().len() == 0 {
            println!("`{}` is not found", word);
            exit(1);
        } else {
            println!("{}", meaning.trim());
        }
    } else {
        let meaning = dict::en2zh(&html);
        if meaning.trim().len() == 0 {
            println!("`{}` is not found", word);
            exit(1);
        }

        println!("{}\n{}", word, meaning.trim());
        let vtype = dict::get_exam_type(&html);
        if !vtype.is_empty() {
            println!();
            for tp in vtype {
                print!("<{}> ", tp);
            }
            println!();
        }

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
    };
}