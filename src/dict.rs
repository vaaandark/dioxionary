use scraper::{Html, Selector};
use std::fmt;

fn gen_url(word: &str) -> String {
    format!("https://www.youdao.com/result?word={}&lang=en", word)
}

fn is_enword(word: &str) -> bool {
    word.as_bytes().into_iter().all(|x| x.is_ascii_alphabetic())
}

async fn get_html(word: &str) -> Result<Html, reqwest::Error> {
    static APP_USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Mobile Safari/537.36";
    let client = reqwest::Client::builder().user_agent(APP_USER_AGENT).build()?;
    let url = gen_url(word);
    let res = client.get(url).send().await?;
    let body = res.text().await?;
    Ok(Html::parse_document(&body))
}

fn zh2en(html: &Html) -> String {
    let mut res = String::new();
    let trans = Selector::parse("ul.basic").unwrap();
    html.select(&trans).into_iter().for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            res.push_str(x);
            res.push_str("\n");
        });
    });
    res
}

fn en2zh(html: &Html) -> String {
    let mut res = String::new();
    let phonetic = Selector::parse(".phonetic").unwrap();
    html.select(&phonetic).into_iter().for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            res.push_str(x);
            res.push_str(" ");
        });
    });
    res.push_str("\n");
    let trans = Selector::parse(".trans").unwrap();
    html.select(&trans).into_iter().for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            res.push_str(x);
            res.push_str("\n");
        });
    });
    res
}

fn get_exam_type(html: &Html) -> Vec<String> {
    let types = Selector::parse(".exam_type-value").unwrap();
    let mut res: Vec<String> = Vec::new();
    html.select(&types).into_iter().for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            res.push(x.to_string())
        })
    });
    res
}

pub struct WordItem {
    word: String,
    is_en: bool,
    trans: String,
    types: Option<Vec<String>>
}

impl WordItem {
    pub fn new(word: String, is_en: bool, trans: String, types: Option<Vec<String>>) -> WordItem {
        WordItem { word, is_en, trans, types }
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
            types.iter().for_each(|x| {
                types_content.push_str(&format!("<{}> ", x))
            })
        };
        write!(f, "{}\n{}{}", self.word, self.trans.trim(), types_content)
    }
}

pub async fn lookup(word: &str) -> Option<WordItem> {
    let html = match get_html(&word).await {
        Ok(html) => html,
        Err(e) => {
            panic!("rmall: {:?}", e)
        }
    };
    let is_en = is_enword(word);
    let trans = match is_en {
        true => en2zh(&html).trim().to_string(),
        false => zh2en(&html).trim().to_string()
    };
    // cannot find the word
    if trans.is_empty() {
        None
    } else {
        let types = match is_en {
            true => Some(get_exam_type(&html)),
            false => None
        };
        Some(WordItem::new(word.to_string(), is_en, trans, types))
    }
}