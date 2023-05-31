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
use stardict::StarDict;

fn lookup_online(word: &str) -> Result<()> {
    let word = dict::lookup(word)?;
    println!("{}", word);
    if word.is_en() {
        history::add_history(word.word(), word.types())?;
    }
    Ok(())
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

    let mut dicts = Vec::new();
    if let Some(path) = path {
        dicts.push(StarDict::new(path.into())?);
    } else {
        for d in get_dicts_entries()? {
            dicts.push(StarDict::new(d.path())?);
        }
    }

    for d in &dicts {
        match d.exact_lookup(word) {
            Some(entry) => {
                println!("{}\n{}", entry.word, entry.trans);
                return Ok(());
           }
            _ => println!("{:?}", Error::WordNotFound(d.dict_name().to_owned())),
        }
    }

    if local_first {
        if let Err(_) = lookup_online(word) {
            println!("{:?}", Error::WordNotFound("online dictionary".to_owned()));
        }
    }

    if !exact {
        println!("Fuzzy search enabled");
        if let Some(selection) = Select::with_theme(&ColorfulTheme::default())
            .items(&dicts.iter().map(|x| x.dict_name()).collect::<Vec<&str>>())
            .default(0)
            .interact_on_opt(&Term::stderr())?
        {
            if let Some(entries) = dicts[selection].fuzzy_lookup(word) {
                if let Some(sub_selection) = Select::with_theme(&ColorfulTheme::default())
                    .items(&entries.iter().map(|x| x.word).collect::<Vec<&str>>())
                    .default(0)
                    .interact_on_opt(&Term::stderr())?
                {
                    let entry = &entries[sub_selection];
                    println!("{}\n{}", entry.word, entry.trans);
                    return Ok(())
                }
            }
        }
    }

    Err(Error::WordNotFound("All Dictionaries".to_string()))
}

pub fn repl(online: bool, local_first: bool, exact: bool, path: &Option<String>) -> Result<()> {
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
