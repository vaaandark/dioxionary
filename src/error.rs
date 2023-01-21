use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// The word is not found
    #[error("transparent")]
    WordNotFound,

    /// Some thing wrong with sqlite or your sql statements
    #[error("transparent")]
    Sqlite(#[from] rusqlite::Error)
}

pub type Result<T> = std::result::Result<T, Error>;
