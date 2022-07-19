use reqwest;
use scraper::{Html, Selector};

pub fn generate_url(word: &str) -> String {
    let mut s = "https://www.youdao.com/result?word=".to_string();
    s.push_str(&word);
    s.push_str("&lang=en");
    s
}

pub fn is_enword(word: &str) -> bool {
    for c in word.chars() {
        if !c.is_ascii_alphabetic() {
            return false
        }
    }
    true
}

pub async fn lookup(word: &str) -> Result<String, reqwest::Error> {
    // header
    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
    );
    let client = reqwest::Client::builder().user_agent(APP_USER_AGENT).build()?;

    let url = generate_url(word);
    println!("look up in youdao at {}", url);

    let res = client.get(url).send().await?;
    let body = res.text().await?;
    Ok(body)
}

pub fn get_meaning(body: String, is_zh2en: bool) -> String {
    let html = Html::parse_document(&body);
    let mut meaning = "".to_string();
    if is_zh2en {
        let trans = Selector::parse("ul.basic").unwrap();
        for trans in html.select(&trans) {
            let vt = trans.text().collect::<Vec<_>>();
            for t in vt {
                if t.as_bytes()[0].is_ascii_digit() {
                    meaning.push_str("\n");
                } else {
                    meaning.push_str(t);
                    meaning.push_str("\n");
                }
            }
        }
    } else {
        let trans = Selector::parse(".trans").unwrap();
        for trans in html.select(&trans) {
            let vt = trans.text().collect::<Vec<_>>();
            for t in vt {
                meaning.push_str(t);
                meaning.push_str("\n");
            }
        }
    }
    meaning
}
