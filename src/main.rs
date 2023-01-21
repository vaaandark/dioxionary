use rmall::cli::{Action, Cli, Parser};
use rmall::dict;
use rmall::history;
use std::process::exit;
use tokio;

#[tokio::main]
async fn main() {
    let cli: Cli = Cli::parse();

    match cli.action {
        Action::Count => {
            if let Err(_) = history::count_history() {
                eprintln!("rmall: cannot count the records");
            };
        }
        Action::List(t) => {
            if let Err(_) = history::list_history(t.type_) {
                eprintln!("rmall: cannot list the records");
            };
        }
        Action::Lookup(w) => {
            let word = w.word;
            let item = dict::lookup(&word).await;
            if let Some(word) = item {
                println!("{}", word);
                if word.is_en() {
                    match history::add_history(word.word(), word.types()) {
                        Ok(_) => (),
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
}
