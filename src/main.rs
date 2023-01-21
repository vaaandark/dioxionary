use rmall::{
    cli::{Action, Cli, Parser},
    dict,
    error::{Error, Result},
    history
};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    match cli.action {
        Action::Count => history::count_history(),
        Action::List(t) => history::list_history(t.type_),
        Action::Lookup(w) => {
            let item = dict::lookup(&w.word).await;
            if let Some(word) = item {
                println!("{}", word);
                if word.is_en() {
                    history::add_history(word.word(), word.types())?;
                }
                Ok(())
            } else {
                Err(Error::WordNotFound)
            }
        }
    }
}
