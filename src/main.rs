//! StarDict in Rust!
//! Use offline or online dictionary to look up words and memorize words in the terminal!
use anyhow::Result;
use clap::CommandFactory;
use dioxionary::{
    cli::{Action, Cli, Parser},
    dicts::{default_llm_dict_config_path, default_local_dict_path, DictManager, DictOptions},
    history,
};
use std::env;

fn main() -> Result<()> {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            match e.kind() {
                clap::error::ErrorKind::InvalidSubcommand
                | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                | clap::error::ErrorKind::UnknownArgument => {
                    // Maybe omit the subcommand, so insert it
                    let mut args = std::env::args().collect::<Vec<_>>();
                    args.insert(1, "lookup".to_string());
                    Cli::parse_from(args)
                }
                _ => {
                    e.exit();
                }
            }
        }
    };

    match cli.action {
        Action::LookUp(look_up) => {
            let options = DictOptions::default()
                .prioritize_offline(look_up.local_first)
                .prioritize_online(look_up.use_online)
                .use_llm_dicts(look_up.use_llm)
                .require_exact_match(look_up.exact_search);
            #[cfg(feature = "pronunciation")]
            let options = options.read_aloud(look_up.read_aloud);
            let local_dicts = if let Some(path) = look_up.local_dicts {
                Some(path)
            } else {
                default_local_dict_path()
            };
            let manager =
                DictManager::new(local_dicts, default_llm_dict_config_path(), options).unwrap();
            if let Some(words) = look_up.word {
                words.iter().for_each(|word| manager.query(word));
            } else {
                manager.repl();
            }
        }
        Action::Dicts => {
            let manager = DictManager::new(
                default_local_dict_path(),
                default_llm_dict_config_path(),
                DictOptions::default(),
            )
            .unwrap();
            manager.list_dicts();
        }
        Action::Count => {
            history::count_history_records().unwrap();
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
