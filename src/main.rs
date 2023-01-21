use rmall::cli::{Args, Parser};
use rmall::dict;
use rmall::history;
use std::process::exit;
use tokio;

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    if let Some(type_) = args.list {
        if let Err(_) = history::list_history(type_) {
            eprintln!("rmall: cannot list history");
            exit(1);
        };
    } else {
        let word = match args.word {
            Some(w) => w,
            _ => "rust".to_string()
        };
        let item = dict::lookup(&word).await;
        if let Some(word) = item {
            println!("{}", word);
            if word.is_en() {
                match history::add_history(word.word(), word.types()) {
                    Ok(_) => (),
                    // maybe the word has been looked up before
                    Err(rusqlite::Error::SqliteFailure(_, _)) => (),
                    Err(_) => {
                        eprintln!("rmall: cannot add history");
                        exit(1);
                    }
                }
            }
        } else {
            println!("`{}` is not found", word);
            exit(1);
        }
    }
}
