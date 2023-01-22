use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// The word is not found
    #[error("transparent")]
    WordNotFound,

    /// Something wrong with sqlite or your sql statements
    #[error("transparent")]
    Sqlite(#[from] rusqlite::Error),

    /// Something wrong with cache directory
    #[error("transparent")]
    CacheDirNotFound,

    /// Something wrong with creating directory
    #[error("transparent")]
    CannotCreateDir(#[from] std::io::Error),

    /// Something wrong with network
    #[error("transparent")]
    NetworkError(#[from] reqwest::Error),

    /// Something wrong with html parsing
    #[error("transparent")]
    ParsingError,
}

pub type Result<T> = std::result::Result<T, Error>;
