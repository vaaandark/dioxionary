//! Look up words form the offline stardicts.
use anyhow::{anyhow, Context, Result};
use eio::FromBytes;
use flate2::read::GzDecoder;
use std::cmp::min;
use std::fmt::Debug;
use std::fs::{read, File};
use std::io::{prelude::*, BufReader};
use std::mem;
use std::path::Path;

/// The stardict to be looked up.
#[allow(unused)]
pub struct StarDict {
    metadata: Metadata,
    indices: DictIndices,
    contents: DictContents,
}

/// A word entry of the stardict.
/// Represents a dictionary entry with word and its translation
pub struct DictEntry<'a> {
    pub word: &'a str,
    pub translation: &'a str,
}

#[allow(unused)]
impl<'a> StarDict {
    /// Load stardict from a directory.
    pub fn new<P: AsRef<Path>>(dir_path: P) -> Result<StarDict> {
        let mut metadata: Option<_> = None;
        let mut indices: Option<_> = None;
        let mut contents: Option<_> = None;

        let dir_path = dir_path.as_ref();
        for path in dir_path
            .read_dir()
            .with_context(|| format!("Failed to open directory {}", dir_path.display()))?
            .flatten()
        {
            let path = path.path();
            if let Some(extension) = path.extension() {
                match extension.to_str().unwrap() {
                    "ifo" => metadata = Some(path),
                    "idx" => indices = Some(path),
                    "dz" => contents = Some(path),
                    _ => (),
                }
            }
        }

        if metadata.is_none() || indices.is_none() || contents.is_none() {
            return Err(anyhow!("Stardict file is incomplete in {:?}", dir_path));
        }

        let metadata = Metadata::new(metadata.unwrap())?;
        let mut indices = DictIndices::new(indices.unwrap(), metadata.version())?;
        let contents = DictContents::new(contents.unwrap())?;

        indices
            .items
            .retain(|(word, offset, size)| offset + size < contents.str().len());

        Ok(StarDict {
            metadata,
            indices,
            contents,
        })
    }

    /// Look up a word with fuzzy searching disabled.
    /// Performs an exact match lookup for the given word
    pub fn exact_look_up(&self, word: &str) -> Option<DictEntry<'_>> {
        if let Ok(pos) = self.indices.items.binary_search_by(|probe| {
            probe
                .0
                .to_lowercase()
                .cmp(&word.to_lowercase())
                .then(probe.0.as_str().cmp(word))
        }) {
            let (word, offset, size) = &self.indices.items[pos];
            let translation = self.contents.get(*offset, *size);
            Some(DictEntry { word, translation })
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
    /// Performs a fuzzy search for similar words using edit distance
    pub fn fuzzy_look_up(&self, word: &str) -> Option<Vec<DictEntry<'_>>> {
        let distances: Vec<_> = self
            .indices
            .items
            .iter()
            .filter(|s| !s.0.is_empty())
            .map(|s| Self::min_edit_distance(&word.to_lowercase(), &s.0.to_lowercase()))
            .collect();
        let min_dist = distances.iter().min()?;
        let result = self
            .indices
            .items
            .iter()
            .filter(|s| !s.0.is_empty())
            .enumerate()
            .filter(|(idx, _)| distances[*idx] == *min_dist)
            .map(|(_, x)| {
                let (word, offset, size) = x;
                let translation = self.contents.get(*offset, *size);
                DictEntry { word, translation }
            })
            .collect::<Vec<_>>();
        Some(result)
    }

    /// Get the name of the stardict.
    pub fn dict_name(&'a self) -> &'a str {
        &self.metadata.bookname
    }

    /// Get the number of the words in the stardict.
    pub fn word_count(&self) -> usize {
        self.metadata.wordcount
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
/// Represents StarDict metadata information
struct Metadata {
    version: StarDictVersion,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StarDictVersion {
    V2_4_2,
    V3_0_0,
    Unknown,
}

#[allow(unused)]
impl Metadata {
    fn new<P: AsRef<Path>>(path: P) -> Result<Metadata> {
        let path = path.as_ref();
        let mut metadata = Metadata {
            version: StarDictVersion::Unknown,
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

        for line in BufReader::new(
            File::open(path)
                .with_context(|| format!("Failed to open ifo file {}", path.display()))?,
        )
        .lines()
        {
            let line = line?;
            if let Some(id) = line.find('=') {
                let key = &line[..id];
                let val = String::from(&line[id + 1..]);
                match key {
                    "version" => {
                        metadata.version = if val == "2.4.2" {
                            StarDictVersion::V2_4_2
                        } else if val == "3.0.0" {
                            StarDictVersion::V3_0_0
                        } else {
                            StarDictVersion::Unknown
                        }
                    }
                    "bookname" => metadata.bookname = val,
                    "wordcount" => {
                        metadata.wordcount = val
                            .parse()
                            .with_context(|| format!("Failed to parse info file {:?}", path))?
                    }
                    "synwordcount" => {
                        metadata.synwordcount = val
                            .parse()
                            .with_context(|| format!("Failed to parse info file {:?}", path))?
                    }
                    "idxfilesize" => {
                        metadata.idxfilesize = val
                            .parse()
                            .with_context(|| format!("Failed to parse info file {:?}", path))?
                    }
                    "idxoffsetbits" => {
                        metadata.idxoffsetbits = val
                            .parse()
                            .with_context(|| format!("Failed to parse info file {:?}", path))?
                    }
                    "author" => metadata.author = val,
                    "email" => metadata.email = val,
                    "website" => metadata.website = val,
                    "description" => metadata.description = val,
                    "date" => metadata.date = val,
                    "sametypesequence" => metadata.sametypesequence = val,
                    "dicttype" => metadata.dicttype = val,
                    _ => (),
                };
            }
        }
        Ok(metadata)
    }

    fn version(&self) -> StarDictVersion {
        self.version
    }
}

#[allow(unused)]
/// Contains the actual dictionary content data
struct DictContents(String);

#[allow(unused)]
impl<'a> DictContents {
    fn new<P: AsRef<Path>>(path: P) -> Result<DictContents> {
        let path = path.as_ref();
        let s =
            read(path).with_context(|| format!("Failed to open stardict directory {:?}", path))?;
        let mut d = GzDecoder::new(s.as_slice());
        let mut contents = String::new();
        d.read_to_string(&mut contents).with_context(|| {
            format!("Failed to open stardict directory {:?} as dz format", path)
        })?;
        Ok(DictContents(contents))
    }

    fn str(&'a self) -> &'a str {
        &self.0
    }

    fn get(&'a self, offset: usize, size: usize) -> &'a str {
        &self.0[offset..offset + size]
    }
}

#[allow(unused)]
#[derive(Debug)]
/// Represents the dictionary index structure
struct DictIndices {
    items: Vec<(String, usize, usize)>,
}

#[allow(unused)]
impl DictIndices {
    fn read_bytes<T, const N: usize, P: AsRef<Path>>(path: P) -> Result<Vec<(String, usize, usize)>>
    where
        T: FromBytes<N> + TryInto<usize>,
        <T as TryInto<usize>>::Error: Debug,
    {
        let path = path.as_ref();
        let f = File::open(path).with_context(|| format!("Failed to open idx file {:?}", path))?;
        let mut f = BufReader::new(f);

        let mut items: Vec<_> = Vec::new();

        loop {
            let mut buf: Vec<u8> = Vec::new();

            let read_bytes = f
                .read_until(0, &mut buf)
                .with_context(|| format!("Failed to parse idx file {:?}", path))?;

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
            f.read(&mut b)
                .with_context(|| format!("Failed to parse idx file {:?}", path))?;
            let offset = T::from_be_bytes(b).try_into().unwrap();

            let mut b = [0; N];
            f.read(&mut b)
                .with_context(|| format!("Failed to parse idx file {:?}", path))?;
            let size = T::from_be_bytes(b).try_into().unwrap();

            if !word.is_empty() {
                items.push((word, offset, size))
            }
        }
        Ok(items)
    }

    fn new<P: AsRef<Path>>(path: P, version: StarDictVersion) -> Result<DictIndices> {
        let path = path.as_ref();
        match version {
            StarDictVersion::V2_4_2 => Ok(DictIndices {
                items: DictIndices::read_bytes::<u32, { mem::size_of::<u32>() }, _>(path)?,
            }),
            StarDictVersion::V3_0_0 => Ok(DictIndices {
                items: DictIndices::read_bytes::<u64, { mem::size_of::<u64>() }, _>(path)?,
            }),
            StarDictVersion::Unknown => {
                Err(anyhow!("Wrong stardict version in idx file {:?}", path))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use itertools::izip;

    use super::StarDict;

    #[test]
    fn load_stardict() {
        let stardict = StarDict::new("./stardict-heritage/cdict-gb").unwrap();
        assert_eq!(stardict.dict_name(), "CDICT5英汉辞典");
        assert_eq!(stardict.word_count(), 57510);
    }

    #[test]
    fn lookup_offline() {
        let stardict = StarDict::new("./stardict-heritage/cdict-gb").unwrap();
        stardict.exact_look_up("rust").unwrap();
    }

    #[test]
    fn lookup_offline_fuzzy() {
        let stardict = StarDict::new("./stardict-heritage/cdict-gb").unwrap();
        let misspell = ["rst", "cago", "crade"];
        let correct = ["rust", "cargo", "crate"];
        for (mis, cor) in izip!(misspell, correct) {
            let fuzzy = stardict.fuzzy_look_up(mis).unwrap();
            fuzzy.iter().find(|w| w.word == cor).unwrap();
        }
    }
}
