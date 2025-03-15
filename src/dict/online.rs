use anyhow::{anyhow, Context, Result};
use itertools::{
    EitherOrBoth::{Both, Left, Right},
    Itertools,
};
use scraper::{Html, Selector};

use super::{Dict, LookUpResult, LookUpResultItem};

#[derive(Default)]
pub struct OnlineDict;

fn look_up(word: &str) -> Result<LookUpResult> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let html = fetch_html_content(word).await?;
        let is_en = is_english(word);
        let translation_direction = if is_en {
            parse_english_to_chinese
        } else {
            parse_chinese_to_english
        };
        let translation = translation_direction(&html)?.trim().to_string();
        // find nothing about the word
        if translation.is_empty() {
            Err(anyhow!("Found nothing in online dict"))
        } else {
            let difficulty_levels = if is_en {
                extract_difficulty_levels(&html)?
            } else {
                vec![]
            };
            let word = word.to_owned();
            Ok(LookUpResult::Exact(
                LookUpResultItem::new_with_difficulty_levels(word, translation, difficulty_levels),
            ))
        }
    })
}

impl Dict for OnlineDict {
    fn name(&self) -> &str {
        "Youdao"
    }

    fn is_online(&self) -> bool {
        true
    }

    fn supports_fuzzy_search(&self) -> bool {
        false
    }

    fn look_up(&self, _: bool, word: &str) -> LookUpResult {
        if let Ok(result) = look_up(word) {
            result
        } else {
            eprintln!(
                "{}: {}",
                self.name(),
                anyhow::anyhow!("Failed to search online dict")
            );
            LookUpResult::None
        }
    }

    fn word_count(&self) -> usize {
        0
    }
}

/// build url for looking up.
fn build_translation_url(word: &str) -> String {
    format!("https://www.youdao.com/result?word={}&lang=en", word)
}

/// Is an English word?
fn is_english(word: &str) -> bool {
    word.as_bytes()
        .iter()
        .all(|x| x.is_ascii_alphabetic() || x.is_ascii_whitespace())
}

/// fetch web dictionary html by word.
async fn fetch_html_content(word: &str) -> Result<Html> {
    static APP_USER_AGENT: &str =
        "Mozilla/5.0 (X11; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0";
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .with_context(|| "Failed build up a client for reqwest")?;
    let url = build_translation_url(word);
    let res = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Url {} is unreachable", url))?;
    let body = res
        .text()
        .await
        .with_context(|| "Failed to get full text of the response")?;
    Ok(Html::parse_document(&body))
}

/// Lookup words by Chinese meaning.
fn parse_chinese_to_english(html: &Html) -> Result<String> {
    let mut res = String::new();
    let trans = Selector::parse("ul.basic")
        .map_err(|_| anyhow!("Failed to select the fields of ul.basic in the HTML body"))?;
    html.select(&trans).for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            res.push_str(x);
            res.push('\n');
        });
    });
    Ok(res)
}

/// Lookup words by English word.
fn parse_english_to_chinese(html: &Html) -> Result<String> {
    let mut res = String::new();
    let phonetic = Selector::parse(".per-phone")
        .map_err(|_| anyhow!("Failed select the fields of .per-phone in the HTML body"))?;
    html.select(&phonetic).for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            res.push_str(x);
            res.push(' ');
        });
    });
    res.push('\n');
    let mut pos_text: Vec<&str> = Vec::new();
    let pos = Selector::parse(".pos")
        .map_err(|_| anyhow!("Failed select the fields of .pos in the HTML body"))?;
    html.select(&pos).for_each(|x| {
        x.text().collect::<Vec<_>>().iter().for_each(|x| {
            pos_text.push(*x);
        });
    });
    let mut trans_text: Vec<&str> = Vec::new();
    let trans = Selector::parse(".trans")
        .map_err(|_| anyhow!("Failed to select the fields of .trans in the HTML body"))?;
    html.select(&trans).for_each(|x| {
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

/// Get the diffculty level of the word from html.
fn extract_difficulty_levels(html: &Html) -> Result<Vec<String>> {
    let types = Selector::parse(".exam_type-value")
        .map_err(|_| anyhow!("Failed to select the fields of .exam_type-value in the HTML body"))?;
    let mut res: Vec<String> = Vec::new();
    html.select(&types).for_each(|x| {
        x.text()
            .collect::<Vec<_>>()
            .iter()
            .for_each(|x| res.push(x.to_string()))
    });
    Ok(res)
}

#[cfg(test)]
mod test {
    use core::panic;

    use crate::dict::{Dict, LookUpResult};

    use super::OnlineDict;

    #[test]
    fn look_up_online_by_english() {
        if let LookUpResult::Exact(e) = OnlineDict::default().look_up(false, "rust") {
            println!("{}", e);
        } else {
            panic!("Failed to look up online by english");
        }
    }

    #[test]
    fn look_up_online_by_chinese() {
        if let LookUpResult::Exact(e) = OnlineDict::default().look_up(false, "铁锈") {
            println!("{}", e);
        } else {
            panic!("Failed to look up online by chinese");
        }
    }
}
