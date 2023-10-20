//! Dioxionary command line parameters.
pub use clap::{Args, Parser};
use clap_complete::Shell;

/// Dioxionary command line parameters.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, bin_name = "dioxionary", after_help =
"Examples:
  When no subcommand is specified, the default is 'lookup'.
  you can list all records:
    dioxionary list
  you can also list the following types:
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
    pub action: Option<Action>,

    /// Specify local dictionary.
    #[arg(short, long)]
    pub local: Option<String>,

    /// Use online dictionary.
    #[arg(short = 'x', long, default_value_t = false)]
    pub online: bool,

    /// Try offline dictionary first, then the online.
    #[arg(short = 'L', long, default_value_t = true)]
    pub local_first: bool,

    /// Disable fuzzy search, only use exact search, conflict with `-x`.
    #[arg(short, long, default_value_t = false)]
    pub exact_search: bool,

    /// Play word pronunciation.
    #[arg(short, long, default_value_t = false)]
    pub read_aloud: bool,

    /// Generate shell completion scripts.
    #[arg(short, long, value_enum, value_name = "SHELL")]
    pub completions: Option<Shell>,

    /// The word being looked up.
    pub word: Option<String>,
}

/// Dioxionary subcommands.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Lookup the following word, default offline dictionary.
    Lookup(Lookup),

    /// List the specific types of records.
    List(List),

    /// Count the number of each type.
    Count,

    /// Display list of available dictionaries and exit.
    Dicts,
}

/// Subcommand line parameters for looking up words.
#[derive(Args, Debug)]
pub struct Lookup {
    /// Specify local dictionary.
    #[arg(short, long)]
    pub local: Option<String>,

    /// Use online dictionary.
    #[arg(short = 'x', long, default_value_t = false)]
    pub online: bool,

    /// Try offline dictionary first, then the online.
    #[arg(short = 'L', long, default_value_t = true)]
    pub local_first: bool,

    /// Disable fuzzy search, only use exact search, conflict with `-x`.
    #[arg(short, long, default_value_t = false)]
    pub exact_search: bool,

    /// Play word pronunciation.
    #[arg(short, long, default_value_t = false)]
    pub read_aloud: bool,

    /// The word being looked up.
    pub word: Option<String>,
}

/// Subcommand line parameters for listing history.
#[derive(Args, Debug)]
pub struct List {
    /// Sort lexicographically.
    #[arg(short, long, default_value_t = false)]
    pub sort: bool,

    /// Output to a table.
    #[arg(short, long, default_value_t = false)]
    pub table: bool,

    /// The number of columns in the table.
    #[arg(short, long, default_value_t = 5, requires("table"))]
    pub column: usize,

    /// The difficulty level of the word.
    pub type_: Option<String>,
}
