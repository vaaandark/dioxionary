pub use clap::{Parser, Args};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, after_help =
"Examples:
  `rmall list` to list all
  you can also list the following types:
    'CET4', 'CET6', 'CET8', 'TOEFL', 'IELTS', 'GMAT', 'GRE', 'SAT'"
)]
pub struct Cli {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// lookup the following word
    Lookup(Lookup),

    /// list the specific types of records
    List(List),

    /// count the number of each type
    Count,
}

#[derive(Args, Debug)]
pub struct Lookup {
    pub word: String,
}

#[derive(Args, Debug)]
pub struct List {
    /// sort lexicographically
    #[arg(short, long, default_value_t = false)]
    pub sort: bool,

    /// output to a table
    #[arg(short, long, default_value_t = false, group = "output_format")]
    pub table: bool,

    /// the number of columns in the table
    #[arg(short, long, default_value_t = 5, requires = "output_format")]
    pub column: usize,

    pub type_: Option<String>,
}
