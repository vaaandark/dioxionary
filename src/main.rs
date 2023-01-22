use rmall::{
    cli::{Action, Cli, Parser},
    dict,
    error::Result,
    history,
};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    match cli.action {
        Action::Count => history::count_history(),
        Action::List(t) => history::list_history(t.type_, t.sort, t.table, t.column),
        Action::Lookup(w) => {
            let word = dict::lookup(&w.word).await?;
            println!("{}", word);
            if word.is_en() {
                history::add_history(word.word(), word.types())?;
            }
            Ok(())
        }
    }
}
