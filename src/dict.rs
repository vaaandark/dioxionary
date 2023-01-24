use crate::error::{Error, Result};
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use scraper::{Html, Selector};
use std::fmt;

fn gen_url(word: &str) -> String {
    format!("https://www.youdao.com/result?word={}&lang=en", word)
}

fn is_enword(word: &str) -> bool {
    word.as_bytes()
        .into_iter()
        .all(|x| x.is_ascii_alphabetic() || x.is_ascii_whitespace())
}

async fn get_html(word: &str) -> Result<Html> {
    static APP_USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Mobile Safari/537.36";
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;
    let url = gen_url(word);
    let res = client.get(url).send().await?;
    let body = res.text().await?;
    Ok(Html::parse_document(&body))
}

fn zh2en(html: &Html) -> Result<String> {
    let mut res = String::new();
    let trans = Selector::parse("ul.basic").map_err(|_| Error::HtmlParsingError)?;
    html.select(&trans).into_iter().for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            res.push_str(x);
            res.push_str("\n");
        });
    });
    Ok(res)
}

fn en2zh(html: &Html) -> Result<String> {
    let mut res = String::new();
    let phonetic = Selector::parse(".per-phone").map_err(|_| Error::HtmlParsingError)?;
    html.select(&phonetic).into_iter().for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            res.push_str(x);
            res.push_str(" ");
        });
    });
    res.push_str("\n");
    let mut pos_text: Vec<&str> = Vec::new();
    let pos = Selector::parse(".pos").map_err(|_| Error::HtmlParsingError)?;
    html.select(&pos).into_iter().for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            pos_text.push(*x);
        });
    });
    let mut trans_text: Vec<&str> = Vec::new();
    let trans = Selector::parse(".trans").map_err(|_| Error::HtmlParsingError)?;
    html.select(&trans).into_iter().for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            trans_text.push(*x);
        });
    });
    for i in pos_text
        .iter()
        .zip_longest(trans_text.iter())
        .map(|x| match x {
            Both(a, b) => (a, b),
            Left(a) => (a, &""),
            Right(b) => (&"", b),
        })
    {
        res.push_str(format!("{} {}\n", i.0, i.1).as_str());
    }
    Ok(res)
}

fn get_exam_type(html: &Html) -> Result<Vec<String>> {
    let types = Selector::parse(".exam_type-value").map_err(|_| Error::HtmlParsingError)?;
    let mut res: Vec<String> = Vec::new();
    html.select(&types).into_iter().for_each(|x| {
        x.text()
            .collect::<Vec<_>>()
            .iter()
            .for_each(|x| res.push(x.to_string()))
    });
    Ok(res)
}

pub struct WordItem {
    word: String,
    is_en: bool,
    trans: String,
    types: Option<Vec<String>>,
}

impl WordItem {
    pub fn new(word: String, is_en: bool, trans: String, types: Option<Vec<String>>) -> WordItem {
        WordItem {
            word,
            is_en,
            trans,
            types,
        }
    }
    pub fn is_en(&self) -> bool {
        self.is_en
    }
    pub fn word<'a>(&'a self) -> &'a str {
        &self.word
    }
    #[allow(unused)]
    pub fn trans<'a>(&'a self) -> &'a str {
        &self.trans
    }
    #[allow(unused)]
    pub fn types(&self) -> &Option<Vec<String>> {
        &self.types
    }
}

impl fmt::Display for WordItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut types_content = String::new();
        if let Some(types) = &self.types {
            types_content.push_str("\n");
            types
                .iter()
                .for_each(|x| types_content.push_str(&format!("<{}> ", x)))
        };
        write!(f, "{}\n{}{}", self.word, self.trans.trim(), types_content)
    }
}

pub async fn lookup(word: &str) -> Result<WordItem> {
    let html = get_html(&word).await?;
    let is_en = is_enword(word);
    let dirction = if is_en { en2zh } else { zh2en };
    let trans = dirction(&html)?.trim().to_string();
    // cannot find the word
    if trans.is_empty() {
        Err(Error::WordNotFound)
    } else {
        let types = if is_en {
            Some(get_exam_type(&html)?)
        } else {
            None
        };
        Ok(WordItem::new(word.to_string(), is_en, trans, types))
    }
}
