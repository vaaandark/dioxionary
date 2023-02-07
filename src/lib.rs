pub mod cli;
pub mod dict;
pub mod error;
pub mod history;
pub mod stardict;
use std::path::PathBuf;

use dirs::config_dir;
use error::{Error, Result};
use rustyline::{error::ReadlineError, Editor};
use stardict::{lookup, StarDict};

async fn lookup_online(word: &str) -> Result<()> {
    let word = dict::lookup(word).await?;
    println!("{}", word);
    if word.is_en() {
        history::add_history(word.word(), word.types())?;
    }
    Ok(())
}

fn lookup_offline(path: PathBuf, exact: bool, word: &str) -> Result<()> {
    let stardict = StarDict::new(path)?;

    if exact {
        // assuming no typos, look up for exact results
        let entry = stardict
            .exact_lookup(&word)
            .ok_or(Error::WordNotFound(stardict.dict_name().to_string()))?;
        println!("{}\n{}", entry.word, entry.trans);
        history::add_history(&word, &None)?;
    } else {
        // enable fuzzy search
        match stardict.lookup(&word) {
            Ok(found) => match found {
                lookup::Found::Exact(entry) => {
                    println!("{}\n{}", entry.word, entry.trans);
                    history::add_history(&word, &None)?;
                }
                lookup::Found::Fuzzy(entries) => {
                    println!("Fuzzy search enabled");
                    entries.into_iter().for_each(|e| {
                        println!(
                            "==============================\n>>>>> {} <<<<<\n{}",
                            e.word, e.trans
                        );
                    })
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}

pub async fn query(
    online: bool,
    local_first: bool,
    exact: bool,
    word: String,
    path: &Option<String>,
) -> Result<()> {
    if online {
        // only use online dictionary
        return lookup_online(&word).await;
    }

    if let Some(path) = path {
        return lookup_offline(path.into(), exact, &word);
    }

    let mut path = config_dir().ok_or(Error::ConfigDirNotFound)?;
    path.push("rmall");

    let mut dicts: Vec<_> = path
        .read_dir()
        .map_err(|_| Error::ConfigDirNotFound)?
        .into_iter()
        .filter_map(|x| x.ok())
        .collect();

    dicts.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for d in dicts {
        // use offline dictionary
        if let Err(e) = lookup_offline(d.path(), exact, &word) {
            println!("{:?}", e);
        } else {
            return Ok(());
        }
    }

    let all_fail = Error::WordNotFound("All Dictionaries".to_string());
    if local_first {
        if let Err(e) = lookup_online(&word).await {
            println!("{:?}", e);
            Err(all_fail)
        } else {
            Ok(())
        }
    } else {
        Err(all_fail)
    }
}

pub async fn repl(
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
                if let Err(e) = query(online, local_first, exact, word, path).await {
                    println!("{:?}", e);
                }
            }
            Err(ReadlineError::Interrupted) => break Ok(()),
            Err(ReadlineError::Eof) => break Ok(()),
            _ => break Err(Error::ReadlineError),
        }
    }
}
