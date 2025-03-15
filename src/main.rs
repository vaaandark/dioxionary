//! StarDict in Rust!
//! Use offline or online dictionary to look up words and memorize words in the terminal!
use anyhow::Result;
use clap::CommandFactory;
use dioxionary::{
    cli::{Action, Cli, Parser},
    dictionaries::{default_local_dict_path, DictionaryManager, DictionaryOptions},
    history,
};
use std::env;

fn main() -> Result<()> {
    let mut cli = Cli::parse();

    if cli.action.is_none() {
        let mut args = std::env::args().collect::<Vec<_>>();
        args.insert(1, "lookup".to_string());
        cli = Cli::parse_from(args);
    }

    match cli.action.unwrap() {
        Action::LookUp(look_up) => {
            let options = DictionaryOptions::default()
                .prioritize_offline(look_up.local_first)
                .priortize_online(look_up.use_online)
                .require_exact_match(look_up.exact_search);
            let local_dicts = if let Some(path) = look_up.local_dicts {
                path
            } else {
                default_local_dict_path().unwrap()
            };
            let manager = DictionaryManager::new(local_dicts, options).unwrap();
            if let Some(words) = look_up.word {
                words.iter().for_each(|word| manager.query(word));
            } else {
                manager.repl();
            }
        }
        Action::Count => {
            history::count_history_records().unwrap();
        }
        Action::Dicts => {
            let options = DictionaryOptions::default()
                .prioritize_offline(cli.local_first)
                .priortize_online(cli.use_online)
                .require_exact_match(cli.exact_search);
            let local_dicts = if let Some(path) = cli.local_dicts {
                path
            } else {
                default_local_dict_path().unwrap()
            };
            let manager = DictionaryManager::new(local_dicts, options).unwrap();
            manager.list_dicts();
        }
        Action::List(list) => {
            history::list_history_records(
                list.difficulty_level,
                list.sort_alphabetically,
                list.format_as_table,
                list.max_column,
            )
            .unwrap();
        }
        Action::Completion(completion) => {
            let bin_name = env::args().next().expect("impossible");
            clap_complete::generate(
                completion.shell,
                &mut Cli::command(),
                bin_name,
                &mut std::io::stdout(),
            );
            std::process::exit(0);
        }
    }

    Ok(())
}
