//! Error handling of dioxionary.
use std::path::Path;

use thiserror::Error;

/// Dioxionary-related errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Wrong arguments.
    #[error("something wrong with readline")]
    ReadlineError,

    /// The word is not found.
    #[error("cannot find the word in {0}")]
    WordNotFound(String),

    /// Something wrong with sqlite or your sql statements.
    #[error("something wrong with sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Cache directory is not found.
    #[error("cache directory is not found")]
    CacheDirNotFound,

    /// Config directory is not found.
    #[error("config directory is not found")]
    ConfigDirNotFound,

    /// Something wrong with creating directory.
    #[error("cannot create directory: {0}")]
    CannotCreateDir(#[from] std::io::Error),

    /// Something wrong with network.
    #[error("something wrong with network: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// Something wrong with html parsing.
    #[error("something wrong with HTML parsing")]
    HtmlParsingError,

    /// Something wrong with your path.
    #[error("something wrong with the path")]
    PathError,

    /// Cannot open ifo file.
    #[error("cannot open ifo file {0}")]
    CannotOpenIfoFile(Box<Path>),

    /// Something wrong with your ifo file.
    #[error("something wrong with ifo file {0} parsing")]
    IfoFileParsingError(Box<Path>),

    /// Wrong dict version: only suppurt 2.8.2 and 3.0.0.
    #[error("something wrong with stardict version")]
    VersionError,

    /// Cannot open dict file.
    #[error("cannot open dict file {0}")]
    CannotOpenDictFile(Box<Path>),

    /// Something wrong with your dict file directory.
    #[error("something wrong with your stardict directory {0}")]
    StarDictDirError(Box<Path>),

    /// Something wrong with your dict file.
    #[error("something wrong with your dict file {0}")]
    DictFileError(Box<Path>),

    /// Cannot open idx file.
    #[error("transparent")]
    CannotOpenIdxFile,

    /// Something wrong with your idx file.
    #[error("transparent")]
    IdxFileParsingError,
}

/// Return type of most dixionary functions.
pub type Result<T> = std::result::Result<T, Error>;
