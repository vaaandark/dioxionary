use rmall::{
    cli::{Action, Cli, Parser},
    error::{Error, Result},
    history, lookup_online,
    stardict::lookup,
    stardict::StarDict,
};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    match cli.action {
        Action::Count => history::count_history(),
        Action::List(t) => history::list_history(t.type_, t.sort, t.table, t.column),
        Action::Lookup(w) => {
            if let Some(path) = w.local {
                // use offline dictionary
                let stardict = StarDict::new(path.into())?;

                if w.exact {
                    // assuming no typos, look up for exact results
                    let entry = stardict
                        .exact_lookup(&w.word)
                        .ok_or(Error::WordNotFound(stardict.dict_name().to_string()))?;
                    println!("{}\n{}", entry.word, entry.trans);
                    history::add_history(&w.word, &None)?;
                } else {
                    // enable fuzzy search
                    match stardict.lookup(&w.word) {
                        Ok(found) => match found {
                            lookup::Found::Exact(entry) => {
                                println!("{}\n{}", entry.word, entry.trans);
                                history::add_history(&w.word, &None)?;
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
                            if w.local_first {
                                lookup_online(&w.word).await?;
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
            } else {
                // only use online dictionary
                lookup_online(&w.word).await?;
            }
            Ok(())
        }
    }
}
