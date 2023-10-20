//! Error handling of dioxionary.
use thiserror::Error;

/// Dioxionary-related errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Wrong arguments.
    #[error("transparent")]
    ReadlineError,

    /// The word is not found.
    #[error("transparent")]
    WordNotFound(String),

    /// Something wrong with sqlite or your sql statements.
    #[error("transparent")]
    Sqlite(#[from] rusqlite::Error),

    /// Cache directory is not found.
    #[error("transparent")]
    CacheDirNotFound,

    /// Config directory is not found.
    #[error("transparent")]
    ConfigDirNotFound,

    /// Something wrong with creating directory.
    #[error("transparent")]
    CannotCreateDir(#[from] std::io::Error),

    /// Something wrong with network.
    #[error("transparent")]
    NetworkError(#[from] reqwest::Error),

    /// Something wrong with html parsing.
    #[error("transparent")]
    HtmlParsingError,

    /// Something wrong with your path.
    #[error("transparent")]
    PathError,

    /// Cannot open ifo file.
    #[error("transparent")]
    CannotOpenIfoFile,

    /// Something wrong with your ifo file.
    #[error("transparent")]
    IfoFileParsingError,

    /// Wrong dict version: only suppurt 2.8.2 and 3.0.0.
    #[error("transparent")]
    VersionError,

    /// Cannot open dict file.
    #[error("transparent")]
    CannotOpenDictFile,

    /// Something wrong with your dict file directory.
    #[error("transparent")]
    StarDictDirError(String),

    /// Something wrong with your dict file.
    #[error("transparent")]
    DictFileError,

    /// Cannot open idx file.
    #[error("transparent")]
    CannotOpenIdxFile,

    /// Something wrong with your idx file.
    #[error("transparent")]
    IdxFileParsingError,
}

/// Return type of most dixionary functions.
pub type Result<T> = std::result::Result<T, Error>;
