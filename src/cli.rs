//! Dioxionary command line parameters.
use std::path::PathBuf;

pub use clap::{Args, Parser};
use clap_complete::Shell;

/// Dioxionary command line parameters.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, bin_name = "dioxionary", after_help =
"Examples:
  When no subcommand is specified, the default is 'lookup'.
  you can list all records:
    dioxionary list
  you can also list the following difficulty level records:
    'CET4', 'CET6', 'TOEFL', 'IELTS', 'GMAT', 'GRE', 'SAT'
  you can count all records:
    dioxionary count
  you can list all dictionaries:
    dioxionary dicts
"
)]
pub struct Cli {
    /// Dioxionary subcommands.
    #[command(subcommand)]
    pub action: Action,
}

/// Dioxionary subcommands.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Look up the following word, default offline dictionary.
    #[command(name = "lookup", visible_alias = "l")]
    LookUp(LookUp),

    /// List the specific difficulty level records.
    #[command(visible_alias = "ls")]
    List(List),

    /// Count the number of each difficulty level records.
    #[command(visible_alias = "c")]
    Count,

    /// Display list of available dictionaries and exit.
    Dicts,

    /// Generate shell completion scripts.
    Completion(Completion),
}

/// Subcommand line parameters for looking up words.
#[derive(Args, Debug)]
pub struct LookUp {
    /// Specify local dictionary.
    #[arg(short, long, name = "local")]
    pub local_dicts: Option<PathBuf>,

    /// Use online dictionary.
    #[arg(short = 'x', long, default_value_t = false, name = "online")]
    pub use_online: bool,

    /// Try offline dictionary first, then the online.
    #[arg(short = 'L', long, default_value_t = true)]
    pub local_first: bool,

    /// Disable fuzzy search, only use exact search, conflict with `-x`.
    #[arg(short, long, default_value_t = false)]
    pub exact_search: bool,

    /// Play word pronunciation.
    #[cfg(feature = "pronunciation")]
    #[arg(short, long, default_value_t = false)]
    pub read_aloud: bool,

    /// The word being looked up.
    pub word: Option<Vec<String>>,
}

/// Subcommand line parameters for listing history.
#[derive(Args, Debug)]
pub struct List {
    /// Sort lexicographically.
    #[arg(short, long, default_value_t = false, name = "sort")]
    pub sort_alphabetically: bool,

    /// Output to a table.
    #[arg(short, long, default_value_t = false, name = "table")]
    pub format_as_table: bool,

    /// The number of columns in the table.
    #[arg(short, long, default_value_t = 5, name = "column", requires("table"))]
    pub max_column: usize,

    /// The difficulty level of the word.
    pub difficulty_level: Option<String>,
}

/// Subcommand line parameters for shell completion.
#[derive(Args, Debug)]
pub struct Completion {
    pub shell: Shell,
}
