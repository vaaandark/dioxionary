use anyhow::Context;
use dioxionary::stardict::{Idx, Ifo};
use std::collections::{BTreeMap, BTreeSet};
use std::env::args;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

fn main() {
    let file_name = args().nth(1).expect("file_name expected");
    let file_path = PathBuf::from(file_name);
    let Ok(mut lines) = read_lines(&file_path) else {
        panic!("can't read file")
    };
    let mut content: String = String::new();
    let mut btree_map: BTreeMap<(String, String), (u32, u32)> = BTreeMap::new();

    while let Some(line) = lines.next() {
        let Ok(line) = line else { continue };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key_word, c)) = line.split_once('\u{0009}') {
            let c = c.replace("\\n", "\n");
            let size: u32 = c.len().try_into().unwrap();
            if size == 0 {
                eprintln!("no content found for {line}");
                continue;
            }
            let offset: u32 = content.len().try_into().unwrap();
            content += &c;

            if let Some(_) = btree_map.insert(
                (key_word.to_lowercase(), key_word.to_owned()),
                (offset, size),
            ) {
                eprintln!("duplicate {key_word}");
            };
        } else {
            let offset = content.len();
            while let Some(Ok(line)) = lines.next() {
                if line.trim().is_empty() {
                    break;
                }
                content.push('\n');
                let line = line.replace("\\n", "\n");
                if let Some(x) = line.find('\u{0009}') {
                    let key_word = &line[0..x];
                    let c = &line[x + 1..];
                    if let Some(_) = btree_map.insert(
                        (key_word.to_lowercase(), key_word.to_owned()),
                        (
                            (content.len() + x + 1).try_into().unwrap(),
                            c.len().try_into().unwrap(),
                        ),
                    ) {
                        eprintln!("duplicate {key_word}");
                    };
                }
                content += &line;
            }
            let size = content.len() - offset;
            if size == 0 {
                eprintln!("no content found for {line}");
                continue;
            }

            let offset: u32 = offset.try_into().unwrap();
            let size: u32 = size.try_into().unwrap();

            for key_word in line
                .split('|')
                .map(|word| word.trim())
                .filter(|word| !word.is_empty())
                .collect::<BTreeSet<_>>()
            {
                if let Some(_) = btree_map.insert(
                    (key_word.to_lowercase(), key_word.to_owned()),
                    (offset, size),
                ) {
                    eprintln!("duplicate {key_word}");
                };
            }
        };
    }

    let ifo = Ifo {
        version: dioxionary::stardict::Version::V242,
        bookname: file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        wordcount: btree_map.len(),
        synwordcount: 0,
        idxfilesize: 0,
        idxoffsetbits: 0,
        author: String::new(),
        email: String::new(),
        website: String::new(),
        description: String::new(),
        date: String::new(),
        sametypesequence: "m".to_string(),
        dicttype: String::new(),
    };
    fs::write(&file_path.with_extension("ifo"), ifo.to_string()).expect("can't write ifo");

    let items: Vec<(String, u32, u32)> = btree_map
        .into_iter()
        .map(|((_, k), (offset, size))| (k, offset, size))
        .collect();
    Idx::write_bytes(file_path.with_extension("idx"), items).expect("can't write idx");

    fs::write(file_path.with_extension("dict"), content)
        .with_context(|| format!("Failed to create dict file"))
        .unwrap();
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
