pub mod cli;
pub mod dict;
pub mod error;
pub mod history;
pub mod stardict;
use error::{Error, Result};
use stardict::{lookup, StarDict};

pub async fn lookup_online(word: &str) -> Result<()> {
    let word = dict::lookup(word).await?;
    println!("{}", word);
    if word.is_en() {
        history::add_history(word.word(), word.types())?;
    }
    Ok(())
}

pub async fn query(
    path: Option<String>,
    local_first: bool,
    exact: bool,
    word: String,
) -> Result<()> {
    if let Some(path) = path {
        // use offline dictionary
        let stardict = StarDict::new(path.into())?;

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
                    if local_first {
                        lookup_online(&word).await?;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    } else {
        // only use online dictionary
        lookup_online(&word).await?;
    }
    Ok(())
}
