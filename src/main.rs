use rmall::{
    cli::{Action, Cli, Parser},
    error::{Error, Result},
    history, query,
};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    if let Some(action) = cli.action {
        match action {
            Action::Count => history::count_history(),
            Action::List(t) => history::list_history(t.type_, t.sort, t.table, t.column),
            Action::Lookup(w) => query(w.online, w.local_first, w.exact, w.word, w.local).await,
        }
    } else {
        if let Some(word) = cli.word {
            query(cli.online, cli.local_first, cli.exact, word, cli.local).await
        } else {
            Err(Error::ArgumentsError)
        }
    }
}
