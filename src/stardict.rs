use crate::error::{Error, Result};
use eio::FromBytes;
use flate2::read::GzDecoder;
use std::fmt::Debug;
use std::fs::{read, File};
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;

#[allow(unused)]
pub struct StarDict {
    ifo: Ifo,
    idx: Idx,
    dict: Dict,
}

#[allow(unused)]
impl<'a> StarDict {
    pub fn new(path: PathBuf) -> Result<StarDict> {
        let dir = path.file_name().ok_or(Error::PathError)?;
        let dir = dir.to_str().unwrap();
        let ifo = Ifo::new(path.join(format!("{}.ifo", dir)))?;
        let idx = Idx::new(path.join(format!("{}.idx", dir)), ifo.version())?;
        let dict = Dict::new(path.join(format!("{}.dict.dz", dir)))?;
        Ok(StarDict { ifo, idx, dict })
    }

    pub fn quick_lookup(&'a self, word: &str) -> Result<&'a str> {
        let mut trans: &str = "";
        self.idx.items.iter().for_each(|x| {
            if x.0 == word {
                let (_, offset, size) = *x;
                trans = self.dict.get(offset, size);
            }
        });

        if trans.len() > 0 {
            Ok(trans)
        } else {
            Err(Error::WordNotFound)
        }
    }

    pub fn binary_lookup_unchecked(&'a self, word: &str) -> Result<&'a str> {
        if let Ok(pos) = self.idx.items.binary_search_by_key(&word, |x| &x.0) {
            let (_, offset, size) = self.idx.items[pos];
            Ok(self.dict.get(offset, size))
        } else {
            Err(Error::WordNotFound)
        }
    }

    pub fn binary_lookup(&'a self, word: &str) -> Result<&'a str> {
        let mut items = self.idx.items.clone();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        self.binary_lookup_unchecked(word)
    }
}

/// bookname=      // required
/// wordcount=     // required
/// synwordcount=  // required if ".syn" file exists.
/// idxfilesize=   // required
/// idxoffsetbits= // New in 3.0.0
/// author=
/// email=
/// website=
/// description=	// You can use <br> for new line.
/// date=
/// sametypesequence= // very important.
/// dicttype=

#[allow(unused)]
#[derive(Debug)]
struct Ifo {
    version: Version,
    bookname: String,
    wordcount: usize,
    synwordcount: usize,
    idxfilesize: usize,
    idxoffsetbits: usize,
    author: String,
    email: String,
    website: String,
    description: String,
    date: String,
    sametypesequence: String,
    dicttype: String,
}

#[derive(Debug, Clone, Copy)]
enum Version {
    V242,
    V300,
    Unknown,
}

#[allow(unused)]
impl Ifo {
    fn new(path: PathBuf) -> Result<Ifo> {
        let mut ifo = Ifo {
            version: Version::Unknown,
            bookname: String::new(),
            wordcount: 0,
            synwordcount: 0,
            idxfilesize: 0,
            idxoffsetbits: 0,
            author: String::new(),
            email: String::new(),
            website: String::new(),
            description: String::new(),
            date: String::new(),
            sametypesequence: String::new(),
            dicttype: String::new(),
        };

        for line in BufReader::new(File::open(path).map_err(|_| Error::CannotOpenIfoFile)?).lines()
        {
            let line = line?;
            if let Some(id) = line.find('=') {
                let key = &line[..id];
                let val = String::from(&line[id + 1..]);
                match key {
                    "version" => {
                        ifo.version = if val == "2.4.2" {
                            Version::V242
                        } else if val == "3.0.0" {
                            Version::V300
                        } else {
                            Version::Unknown
                        }
                    }
                    "bookname" => ifo.bookname = val,
                    "wordcount" => {
                        ifo.wordcount = val.parse().map_err(|_| Error::IfoFileParsingError)?
                    }
                    "synwordcount" => {
                        ifo.synwordcount = val.parse().map_err(|_| Error::IfoFileParsingError)?
                    }
                    "idxfilesize" => {
                        ifo.idxfilesize = val.parse().map_err(|_| Error::IfoFileParsingError)?
                    }
                    "idxoffsetbits" => {
                        ifo.idxoffsetbits = val.parse().map_err(|_| Error::IfoFileParsingError)?
                    }
                    "author" => ifo.author = val,
                    "email" => ifo.email = val,
                    "website" => ifo.website = val,
                    "description" => ifo.description = val,
                    "date" => ifo.date = val,
                    "sametypesequence" => ifo.sametypesequence = val,
                    "dicttype" => ifo.dicttype = val,
                    _ => (),
                };
            }
        }
        Ok(ifo)
    }

    fn version(&self) -> Version {
        self.version
    }
}

#[allow(unused)]
struct Dict {
    content: String,
}

#[allow(unused)]
impl<'a> Dict {
    fn new(path: PathBuf) -> Result<Dict> {
        let s = read(path).map_err(|x| Error::CannotOpenDictFile)?;
        let mut d = GzDecoder::new(s.as_slice());
        let mut content = String::new();
        d.read_to_string(&mut content)
            .map_err(|_| Error::DictFileError)?;
        Ok(Dict { content })
    }

    fn get(&'a self, offset: usize, size: usize) -> &'a str {
        &self.content[offset..offset + size]
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Idx {
    items: Vec<(String, usize, usize)>,
}

#[allow(unused)]
impl Idx {
    fn read_bytes<'a, const N: usize, T>(path: PathBuf) -> Result<Vec<(String, usize, usize)>>
    where
        T: FromBytes<N> + TryInto<usize>,
        <T as TryInto<usize>>::Error: Debug,
    {
        let f = File::open(path).map_err(|_| Error::CannotOpenIdxFile)?;
        let mut f = BufReader::new(f);

        let mut items: Vec<_> = Vec::new();
        let mut buf: Vec<u8> = Vec::new();

        while let Ok(n) = f.read_until(0, &mut buf) {
            if n == 0 {
                break;
            }

            buf.pop();
            let mut word = String::new();
            buf.iter().for_each(|x| word.push(*x as char));
            buf.clear();

            let mut b = [0; N];
            f.read(&mut b).map_err(|_| Error::IdxFileParsingError)?;
            let offset = T::from_be_bytes(b).try_into().unwrap();

            let mut b = [0; N];
            f.read(&mut b).map_err(|_| Error::IdxFileParsingError)?;
            let size = T::from_be_bytes(b).try_into().unwrap();

            items.push((word, offset, size))
        }
        Ok(items)
    }

    fn new(path: PathBuf, version: Version) -> Result<Idx> {
        match version {
            Version::V242 => Ok(Idx {
                items: Idx::read_bytes::<4, u32>(path)?,
            }),
            Version::V300 => Ok(Idx {
                items: Idx::read_bytes::<8, u64>(path)?,
            }),
            Version::Unknown => Err(Error::VersionError),
        }
    }
}
