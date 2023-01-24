pub use clap::{Args, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, after_help =
"Examples:
  you can list all records:
    rmall list
  you can also list the following types:
    'CET4', 'CET6', 'CET8', 'TOEFL', 'IELTS', 'GMAT', 'GRE', 'SAT'

  When you need to use local dictionary, please rename the contents as following:
    cdict-gb
    ├── cdict-gb.dict
    ├── cdict-gb.dict.dz
    ├── cdict-gb.idx
    └── cdict-gb.ifo
  Their prefixes must be the same as the dirname.
  Then you can look up like:
    lookup --local <DICTDIR> <WORD>
  or:
    lookup -l <DICTDIR> <WORD>
"
)]
pub struct Cli {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// lookup the following word, default web dictionary
    Lookup(Lookup),

    /// list the specific types of records
    List(List),

    /// count the number of each type
    Count,
}

#[derive(Args, Debug)]
pub struct Lookup {
    /// use local dictionary
    #[arg(short, long)]
    pub local: Option<String>,

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
