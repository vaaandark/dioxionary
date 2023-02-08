pub use clap::{Args, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, after_help =
"Examples:
  When no subcommand is specified, the default is 'lookup'.
  you can list all records:
    rmall list
  you can also list the following types:
    'CET4', 'CET6', 'CET8', 'TOEFL', 'IELTS', 'GMAT', 'GRE', 'SAT'
  you can count all records:
    rmall count
"
)]
pub struct Cli {
    #[command(subcommand)]
    pub action: Option<Action>,

    /// specify local dictionary
    #[arg(short, long)]
    pub local: Option<String>,

    /// use online dictionary
    #[arg(short = 'x', long)]
    pub online: bool,

    /// try offline dictionary first, then the online
    #[arg(short = 'L', long)]
    pub local_first: bool,

    /// disable fuzzy search, only use exact search, conflict with `-x`
    #[arg(short, long)]
    pub exact_search: bool,

    /// for use in scripts
    #[arg(short, long)]
    pub non_interactive: bool,

    pub word: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// lookup the following word, default offline dictionary
    Lookup(Lookup),

    /// list the specific types of records
    List(List),

    /// count the number of each type
    Count,

    /// display list of available dictionaries and exit
    Dicts,
}

#[derive(Args, Debug)]
pub struct Lookup {
    /// specify local dictionary
    #[arg(short, long)]
    pub local: Option<String>,

    /// use online dictionary
    #[arg(short = 'x', long)]
    pub online: bool,

    /// try offline dictionary first, then the online
    #[arg(short = 'L', long)]
    pub local_first: bool,

    /// disable fuzzy search, only use exact search, conflict with `-x`
    #[arg(short, long)]
    pub exact_search: bool,

    /// for use in scripts
    #[arg(short, long)]
    pub non_interactive: bool,

    pub word: Option<String>,
}

#[derive(Args, Debug)]
pub struct List {
    /// sort lexicographically
    #[arg(short, long, default_value_t = false)]
    pub sort: bool,

    /// output to a table
    #[arg(short, long, default_value_t = false)]
    pub table: bool,

    /// the number of columns in the table
    #[arg(short, long, default_value_t = 5, requires("table"))]
    pub column: usize,

    pub type_: Option<String>,
}
