//! StarDict in Rust!
//! Use offline or online dictionary to look up words and memorize words in the terminal!
pub mod cli;
pub mod dict;
pub mod history;
pub mod stardict;
use std::fs::DirEntry;

use anyhow::{anyhow, Context, Result};
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use prettytable::{Attr, Cell, Row, Table};
use rustyline::{error::ReadlineError, Editor};
use stardict::StarDict;

/// Lookup word from the Internel and add the result to history.
fn lookup_online(word: &str) -> Result<()> {
    let word = dict::WordItem::lookup(word)?;
    println!("{}", word);
    if word.is_en {
        history::add_history(&word.word, &word.types).with_context(|| "Cannot look up online")?;
    }
    Ok(())
}

/// Get the entries of the stardicts.
fn get_dicts_entries() -> Result<Vec<DirEntry>> {
    let mut dioxionary_dir = dirs::config_dir();
    let dioxionary_dir = dioxionary_dir
        .as_mut()
        .map(|dir| {
            dir.push("dioxionary");
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

    let path = match (&dioxionary_dir, &stardict_compatible_dir) {
        (Some(dir), _) => dir,
        (None, Some(dir)) => dir,
        (None, None) => return Err(anyhow!("Couldn't find configuration directory")),
    };

    let mut dicts: Vec<_> = path
        .read_dir()
        .with_context(|| format!("Failed to open configuration directory {:?}", path))?
        .filter_map(|x| x.ok())
        .collect();

    dicts.sort_by_key(|a| a.file_name());

    Ok(dicts)
}

/// Look up a word with many flags.
///
/// # Params
/// - `online`: use online dictionary?
/// - `local_first`: Try offline dictionary first, then the online?
/// - `exact`: disable fuzzy searching?
/// - `word`: the word being looked up.
/// - `path`: the path of the stardict directory.
/// - `read_aloud`: play word pronunciation?
///
/// ## Word prefix
/// - `/terraria`: enable fuzzy searching.
/// - `|terraria`: disable fuzzy searching.
/// - `@terraria`: use online dictionary.
pub fn query(
    online: bool,
    local_first: bool,
    exact: bool,
    word: String,
    path: &Option<String>,
    read_aloud: bool,
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

    let len = word.len();
    let read_aloud = match word.chars().last() {
        Some('~') => {
            word = &word[..len - 1];
            true
        }
        _ => read_aloud,
    };

    if online {
        // only use online dictionary
        lookup_online(word)?;
    } else {
        let mut dicts = Vec::new();
        if let Some(path) = path {
            dicts.push(StarDict::new(path.into())?);
        } else {
            for d in get_dicts_entries()? {
                dicts.push(StarDict::new(d.path())?);
            }
        }

        let mut found = false;
        for d in &dicts {
            match d.exact_lookup(word) {
                Some(entry) => {
                    println!("{}\n{}", entry.word, entry.trans);
                    found = true;
                    break;
                }
                _ => eprintln!("Found nothing in {}", d.dict_name()),
            }
        }

        if !found && local_first {
            if lookup_online(word).is_ok() {
                found = true;
            } else {
                eprintln!("Found nothing in online dict");
            }
        }

        if !found && !exact {
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
                    }
                }
            }
        }
    }

    if read_aloud {
        dict::read_aloud(word)?;
    }

    Ok(())
}

/// Look up a word with many flags interactively using [query].
pub fn repl(
    online: bool,
    local_first: bool,
    exact: bool,
    path: &Option<String>,
    read_aloud: bool,
) -> Result<()> {
    let mut rl = Editor::<()>::new().with_context(|| "Failed to read lines")?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(word) => {
                rl.add_history_entry(&word);
                if let Err(e) = query(online, local_first, exact, word, path, read_aloud) {
                    println!("{:?}", e);
                }
            }
            Err(ReadlineError::Interrupted) => break Ok(()),
            Err(ReadlineError::Eof) => break Ok(()),
            _ => break Err(anyhow!("Failed to read lines")),
        }
    }
}

/// List stardicts in the dioxionary config path.
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
