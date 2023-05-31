pub mod cli;
pub mod dict;
pub mod error;
pub mod history;
pub mod stardict;
use std::fs::DirEntry;

use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use error::{Error, Result};
use prettytable::{Attr, Cell, Row, Table};
use rustyline::{error::ReadlineError, Editor};
use stardict::{lookup, StarDict};

fn lookup_online(word: &str) -> Result<()> {
    let word = dict::lookup(word)?;
    println!("{}", word);
    if word.is_en() {
        history::add_history(word.word(), word.types())?;
    }
    Ok(())
}

fn lookup_offline<'a>(
    stardict: &'a StarDict,
    exact: bool,
    word: &str,
) -> Result<lookup::Found<'a>> {
    if exact {
        stardict.exact_lookup(word)
    } else {
        stardict.lookup(word)
    }
}

fn get_dicts_entries() -> Result<Vec<DirEntry>> {
    let mut rmall_dir = dirs::config_dir();
    let rmall_dir = rmall_dir
        .as_mut()
        .map(|dir| {
            dir.push("rmall");
            dir
        })
        .filter(|dir| dir.is_dir());

    let mut stardict_compatible_dir = dirs::home_dir();
    let stardict_compatible_dir = stardict_compatible_dir
        .as_mut()
        .map(|dir| {
            dir.push(".stardict");
            dir.push("dic");
            dir
        })
        .filter(|dir| dir.is_dir());

    let path = match (&rmall_dir, &stardict_compatible_dir) {
        (Some(dir), _) => dir,
        (None, Some(dir)) => dir,
        (None, None) => return Err(Error::ConfigDirNotFound),
    };

    let mut dicts: Vec<_> = path
        .read_dir()
        .map_err(|_| Error::ConfigDirNotFound)?
        .filter_map(|x| x.ok())
        .collect();

    dicts.sort_by_key(|a| a.file_name());

    Ok(dicts)
}

pub fn query(
    online: bool,
    local_first: bool,
    exact: bool,
    word: String,
    path: &Option<String>,
) -> Result<()> {
    let mut word = word.as_str();
    let online = word.chars().next().map_or(online, |c| {
        if c == '@' {
            word = &word[1..];
            true
        } else {
            online
        }
    });
    if online {
        // only use online dictionary
        return lookup_online(word);
    }

    let exact = match word.chars().next() {
        Some('|') => {
            word = &word[1..];
            true
        }
        Some('/') => {
            word = &word[1..];
            false
        }
        _ => exact,
    };

    if let Some(path) = path {
        let stardict = StarDict::new(path.into())?;
        match lookup_offline(&stardict, exact, word) {
            Ok(lookup::Found::Exact(entry)) => {
                println!("{}\n{}", entry.word, entry.trans);
                return Ok(());
            }
            Ok(lookup::Found::Fuzzy(_, _)) => {
                unreachable!("Fuzzy search is not enabled in this mode!")
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    let mut dicts = Vec::new();
    for d in get_dicts_entries()? {
        dicts.push(StarDict::new(d.path())?);
    }

    if let Ok(results) = dicts
        .iter()
        .map(|d| lookup_offline(&d, exact, word))
        .collect::<Result<Vec<lookup::Found>>>()
    {
        if let Some(first_entry) = results.iter().find_map(|x| x.exact()) {
            println!("{}\n{}", first_entry.word, first_entry.trans);
        } else {
            println!("Fuzzy search enabled");
            if let Some(fuzzy_results) = results
                .iter()
                .map(|x| x.fuzzy())
                .collect::<Option<Vec<(&str, &Vec<lookup::Entry>)>>>()
            {
                if let Some(selection) = Select::with_theme(&ColorfulTheme::default())
                    .items(&fuzzy_results.iter().map(|x| x.0).collect::<Vec<&str>>())
                    .default(0)
                    .interact_on_opt(&Term::stderr())?
                {
                    if let Some(sub_selection) = Select::with_theme(&ColorfulTheme::default())
                        .items(
                            &fuzzy_results[selection]
                                .1
                                .iter()
                                .map(|x| x.word)
                                .collect::<Vec<&str>>(),
                        )
                        .default(0)
                        .interact_on_opt(&Term::stderr())?
                    {
                        let entry = &fuzzy_results[selection].1[sub_selection];
                        println!("{}\n{}", entry.word, entry.trans);
                    }
                }
            }
        }
        Ok(())
    } else {
        let all_fail = Error::WordNotFound("All Dictionaries".to_string());
        if local_first {
            if let Err(e) = lookup_online(word) {
                println!("{:?}", e);
                Err(all_fail)
            } else {
                Ok(())
            }
        } else {
            Err(all_fail)
        }
    }
}

pub fn repl(
    online: bool,
    local_first: bool,
    exact: bool,
    path: &Option<String>,
) -> Result<()> {
    let mut rl = Editor::<()>::new().map_err(|_| Error::ReadlineError)?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(word) => {
                rl.add_history_entry(&word);
                if let Err(e) = query(online, local_first, exact, word, path) {
                    println!("{:?}", e);
                }
            }
            Err(ReadlineError::Interrupted) => break Ok(()),
            Err(ReadlineError::Eof) => break Ok(()),
            _ => break Err(Error::ReadlineError),
        }
    }
}

pub fn list_dicts() -> Result<()> {
    let mut table: Table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Dictionary's name").with_style(Attr::Bold),
        Cell::new("Word count").with_style(Attr::Bold),
    ]));
    get_dicts_entries()?.into_iter().for_each(|x| {
        if let Ok(stardict) = StarDict::new(x.path()) {
            let row = Row::new(vec![
                Cell::new(stardict.dict_name()),
                Cell::new(stardict.wordcount().to_string().as_str()),
            ]);
            table.add_row(row);
        }
    });
    table.printstd();
    Ok(())
}
