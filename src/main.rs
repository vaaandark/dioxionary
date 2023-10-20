use clap::CommandFactory;
use dioxionary::{
    cli::{Action, Cli, Parser},
    error::Result,
    history, list_dicts, query, repl,
};

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    if let Some(shell) = cli.completions {
        clap_complete::generate(
            shell,
            &mut Cli::command(),
            "dioxionary",
            &mut std::io::stdout(),
        );
        std::process::exit(0);
    }

    if let Some(action) = cli.action {
        match action {
            Action::Count => history::count_history(),
            Action::List(t) => history::list_history(t.type_, t.sort, t.table, t.column),
            Action::Lookup(w) => {
                let online = w.online;
                let local_first = w.local_first;
                let exact = w.exact_search;
                let path = &w.local;
                let read_aloud = w.read_aloud;
                if let Some(word) = w.word {
                    query(online, local_first, exact, word, path, read_aloud)
                } else {
                    repl(online, local_first, exact, path, read_aloud)
                }
            }
            Action::Dicts => list_dicts(),
        }
    } else {
        let online = cli.online;
        let local_first = cli.local_first;
        let exact = cli.exact_search;
        let word = cli.word;
        let path = &cli.local;
        let read_aloud = cli.read_aloud;
        if let Some(word) = word {
            query(online, local_first, exact, word, path, read_aloud)
        } else {
            repl(online, local_first, exact, path, read_aloud)
        }
    }
}
