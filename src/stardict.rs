//! Look up words form the offline stardicts.
use crate::error::{Error, Result};
use eio::FromBytes;
use flate2::read::GzDecoder;
use std::cmp::min;
use std::fmt::Debug;
use std::fs::{read, File};
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;

/// The stardict to be looked up.
#[allow(unused)]
pub struct StarDict {
    ifo: Ifo,
    idx: Idx,
    dict: Dict,
}

/// A word entry of the stardict.
pub struct Entry<'a> {
    pub word: &'a str,
    pub trans: &'a str,
}

#[allow(unused)]
impl<'a> StarDict {
    /// Load stardict from a directory.
    pub fn new(path: PathBuf) -> Result<StarDict> {
        let mut ifo: Option<_> = None;
        let mut idx: Option<_> = None;
        let mut dict: Option<_> = None;

        for p in path.read_dir().map_err(|_| Error::PathError)? {
            let path = p.map_err(|_| Error::PathError)?.path();
            if let Some(extension) = path.extension() {
                match extension.to_str().unwrap() {
                    "ifo" => ifo = Some(path),
                    "idx" => idx = Some(path),
                    "dz" => dict = Some(path),
                    _ => (),
                }
            }
        }

        let path = path.to_str().unwrap();
        let ifo = ifo.ok_or(Error::StarDictDirError(path.into()))?;
        let idx = idx.ok_or(Error::StarDictDirError(path.into()))?;
        let dict = dict.ok_or(Error::StarDictDirError(path.into()))?;

        let ifo = Ifo::new(ifo)?;
        let mut idx = Idx::new(idx, ifo.version())?;
        let dict = Dict::new(dict)?;

        idx.items
            .retain(|(word, offset, size)| offset + size < dict.contents.len());

        Ok(StarDict { ifo, idx, dict })
    }

    /// Look up a word with fuzzy searching disabled.
    pub fn exact_lookup(&self, word: &str) -> Option<Entry> {
        if let Ok(pos) = self.idx.items.binary_search_by(|probe| {
            probe
                .0
                .to_lowercase()
                .cmp(&word.to_lowercase())
                .then(probe.0.as_str().cmp(word))
        }) {
            let (word, offset, size) = &self.idx.items[pos];
            let trans = self.dict.get(*offset, *size);
            Some(Entry { word, trans })
        } else {
            None
        }
    }

    /// Calculate word distence for fuzzy searching.
    fn min_edit_distance(pattern: &str, text: &str) -> usize {
        let pattern_chars: Vec<_> = pattern.chars().collect();
        let text_chars: Vec<_> = text.chars().collect();
        let mut dist = vec![vec![0; pattern_chars.len() + 1]; text_chars.len() + 1];

        #[allow(clippy::needless_range_loop)]
        for i in 0..=text_chars.len() {
            dist[i][0] = i;
        }

        for j in 0..=pattern_chars.len() {
            dist[0][j] = j;
        }

        for i in 1..=text_chars.len() {
            for j in 1..=pattern_chars.len() {
                dist[i][j] = if text_chars[i - 1] == pattern_chars[j - 1] {
                    dist[i - 1][j - 1]
                } else {
                    min(min(dist[i][j - 1], dist[i - 1][j]), dist[i - 1][j - 1]) + 1
                }
            }
        }
        dist[text_chars.len()][pattern_chars.len()]
    }

    /// Look up a word with fuzzy searching enabled.
    pub fn fuzzy_lookup(&self, word: &str) -> Option<Vec<Entry>> {
        let distances: Vec<_> = self
            .idx
            .items
            .iter()
            .filter(|s| !s.0.is_empty())
            .map(|s| Self::min_edit_distance(&word.to_lowercase(), &s.0.to_lowercase()))
            .collect();
        let min_dist = distances.iter().min()?;
        let result = self
            .idx
            .items
            .iter()
            .filter(|s| !s.0.is_empty())
            .enumerate()
            .filter(|(idx, _)| distances[*idx] == *min_dist)
            .map(|(_, x)| {
                let (word, offset, size) = x;
                let trans = self.dict.get(*offset, *size);
                Entry { word, trans }
            })
            .collect::<Vec<_>>();
        Some(result)
    }

    /// Get the name of the stardict.
    pub fn dict_name(&'a self) -> &'a str {
        &self.ifo.bookname
    }

    /// Get the number of the words in the stardict.
    pub fn wordcount(&self) -> usize {
        self.ifo.wordcount
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
/// description=   // You can use <br> for new line.
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
    contents: String,
}

#[allow(unused)]
impl<'a> Dict {
    fn new(path: PathBuf) -> Result<Dict> {
        let s = read(path).map_err(|x| Error::CannotOpenDictFile)?;
        let mut d = GzDecoder::new(s.as_slice());
        let mut contents = String::new();
        d.read_to_string(&mut contents)
            .map_err(|_| Error::DictFileError)?;
        Ok(Dict { contents })
    }

    fn get(&'a self, offset: usize, size: usize) -> &'a str {
        &self.contents[offset..offset + size]
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Idx {
    items: Vec<(String, usize, usize)>,
}

#[allow(unused)]
impl Idx {
    fn read_bytes<const N: usize, T>(path: PathBuf) -> Result<Vec<(String, usize, usize)>>
    where
        T: FromBytes<N> + TryInto<usize>,
        <T as TryInto<usize>>::Error: Debug,
    {
        let f = File::open(path).map_err(|_| Error::CannotOpenIdxFile)?;
        let mut f = BufReader::new(f);

        let mut items: Vec<_> = Vec::new();

        loop {
            let mut buf: Vec<u8> = Vec::new();

            let read_bytes = f
                .read_until(0, &mut buf)
                .map_err(|_| Error::DictFileError)?;

            if read_bytes == 0 {
                break;
            }

            if let Some(&trailing) = buf.last() {
                if trailing == b'\0' {
                    buf.pop();
                }
            }

            let mut word: String = String::from_utf8_lossy(&buf)
                .chars()
                .filter(|&c| c != '\u{fffd}')
                .collect();

            let mut b = [0; N];
            f.read(&mut b).map_err(|_| Error::IdxFileParsingError)?;
            let offset = T::from_be_bytes(b).try_into().unwrap();

            let mut b = [0; N];
            f.read(&mut b).map_err(|_| Error::IdxFileParsingError)?;
            let size = T::from_be_bytes(b).try_into().unwrap();

            if !word.is_empty() {
                items.push((word, offset, size))
            }
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
