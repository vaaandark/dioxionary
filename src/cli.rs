pub use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, after_help =
"Examples:
  `rmall -l all` to list all
  you can also list the following types:
    'CET4', 'CET6', 'CET8', 'TOEFL', 'IELTS', 'GMAT', 'GRE', 'SAT'"
)]
pub struct Args {
    /// list the records by the type of words
    #[arg(short, long)]
    pub list: Option<String>,

    pub word: Option<String>,
}
