use rmall::{
    cli::{Action, Cli, Parser},
    dict,
    error::Result,
    history,
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
                if let Ok(trans) = stardict.lookup(&w.word) {
                    println!("{}\n{}", w.word, trans);
                    history::add_history(&w.word, &None)?;
                } else {
                    if w.local_first {
                        let word = dict::lookup(&w.word).await?;
                        println!("{}", word);
                        if word.is_en() {
                            history::add_history(word.word(), word.types())?;
                        }
                    } else {
                        return Err(rmall::error::Error::WordNotFound);
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
