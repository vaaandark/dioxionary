use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// The word is not found
    #[error("transparent")]
    WordNotFound(String),

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
    HtmlParsingError,

    /// something wrong with your path
    #[error("transparent")]
    PathError,

    /// cannot open ifo file
    #[error("transparent")]
    CannotOpenIfoFile,

    /// something wrong with your ifo file
    #[error("transparent")]
    IfoFileParsingError,

    /// wrong dict version: only suppurt 2.8.2 and 3.0.0
    #[error("transparent")]
    VersionError,

    /// cannot open dict file
    #[error("transparent")]
    CannotOpenDictFile,

    /// something wrong with your dict file
    #[error("transparent")]
    DictFileError,

    /// cannot open idx file
    #[error("transparent")]
    CannotOpenIdxFile,

    /// something wrong with your idx file
    #[error("transparent")]
    IdxFileParsingError,

}

pub type Result<T> = std::result::Result<T, Error>;
