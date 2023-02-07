use rmall::{
    cli::{Action, Cli, Parser},
    error::Result,
    history, query, repl,
};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    if let Some(action) = cli.action {
        match action {
            Action::Count => history::count_history(),
            Action::List(t) => history::list_history(t.type_, t.sort, t.table, t.column),
            Action::Lookup(w) => {
                if let Some(word) = w.word {
                    query(w.online, w.local_first, w.exact_search, word, &w.local).await
                } else {
                    repl(w.online, w.local_first, w.exact_search, &w.local).await
                }
            }
        }
    } else {
        if let Some(word) = cli.word {
            query(cli.online, cli.local_first, cli.exact_search, word, &cli.local).await
        } else {
            repl(cli.online, cli.local_first, cli.exact_search, &cli.local).await
        }
    }
}
