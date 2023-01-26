pub mod cli;
pub mod dict;
pub mod error;
pub mod history;
pub mod stardict;
use error::Result;

pub async fn lookup_online(word: &str) -> Result<()> {
    let word = dict::lookup(word).await?;
    println!("{}", word);
    if word.is_en() {
        history::add_history(word.word(), word.types())?;
    }
    Ok(())
}
