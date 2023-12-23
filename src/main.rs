//! StarDict in Rust!
//! Use offline or online dictionary to look up words and memorize words in the terminal!
use anyhow::Result;
use clap::CommandFactory;
use dioxionary::{
    cli::{Action, Cli, Parser},
    history, list_dicts, query, repl,
};
use std::env;

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    if let Some(shell) = cli.completions {
        let bin_name = env::args().next().expect("impossible");
        clap_complete::generate(shell, &mut Cli::command(), bin_name, &mut std::io::stdout());
        std::process::exit(0);
    }

    if let Some(action) = cli.action {
        match action {
            Action::Count => history::count_history(),
            Action::List(t) => history::list_history(t.type_, t.sort, t.table, t.column),
            Action::Lookup(w) => {
                let online = w.online;
                let local_first = w.local_first;
                let exact = w.exact_search;
                let word = w.word;
                let path = &w.local;
                let read_aloud = w.read_aloud;
                if let Some(word_list) = word {
                    let mut found = false;
                    word_list.into_iter().for_each(|word| {
                        if let Err(e) = query(online, local_first, exact, word, path, read_aloud) {
                            eprintln!("{:?}", e);
                        } else {
                            found = true;
                        }
                    });
                    if !found {
                        std::process::exit(1);
                    }
                    Ok(())
                } else {
                    repl(online, local_first, exact, path, read_aloud)
                }
            }
            Action::Dicts => list_dicts(),
        }
    } else {
        let online = cli.online;
        let local_first = cli.local_first;
        let exact = cli.exact_search;
        let word = cli.word;
        let path = &cli.local;
        let read_aloud = cli.read_aloud;
        if let Some(word_list) = word {
            let mut found = false;
            word_list.into_iter().for_each(|word| {
                if let Err(e) = query(online, local_first, exact, word, path, read_aloud) {
                    eprintln!("{:?}", e);
                } else {
                    found = true;
                }
            });
            if !found {
                std::process::exit(1);
            }
            Ok(())
        } else {
            repl(online, local_first, exact, path, read_aloud)
        }
    }
}
