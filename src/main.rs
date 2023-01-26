use rmall::{
    cli::{Action, Cli, Parser},
    dict,
    error::Result,
    history,
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
                let stardict = StarDict::new(path.into())?;
                match stardict.lookup(&w.word) {
                    Ok(found) => match found {
                        lookup::Found::Exact(entry) => {
                            println!("{}\n{}", entry.word, entry.trans);
                            history::add_history(&w.word, &None)?;
                        }
                        lookup::Found::Fuzzy(entries) => {
                            println!("Fuzzy search enabled");
                            for entry in entries {
                                println!("==============================");
                                println!(">>>>> {} <<<<<\n{}", entry.word, entry.trans);
                            }
                        }
                    },
                    Err(e) => {
                        if w.local_first {
                            let word = dict::lookup(&w.word).await.map_err(|_| {
                                rmall::error::Error::WordNotFound(
                                    "both offline and online".to_string(),
                                )
                            })?;

                            println!("{}", word);

                            if word.is_en() {
                                history::add_history(word.word(), word.types())?;
                            }
                        } else {
                            return Err(e);
                        }
                    }
                }
            } else {
                let word = dict::lookup(&w.word).await?;
                println!("{}", word);
                if word.is_en() {
                    history::add_history(word.word(), word.types())?;
                }
            }
            Ok(())
        }
    }
}
