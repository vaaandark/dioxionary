use reqwest;
use tokio;
use scraper::{Html, Selector};

fn generate_url(word: &str) -> String {
    let mut s = "https://www.youdao.com/result?word=".to_string();
    s.push_str(&word);
    s.push_str("&lang=en");
    s
}

fn is_enword(word: &str) -> bool {
    for c in word.chars() {
        if !c.is_ascii_alphabetic() {
            return false
        }
    }
    true
}

async fn lookup(word: &str) -> Result<String, reqwest::Error> {
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

fn get_meaning(body: String, is_zh2en: bool) -> String {
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

#[tokio::main]
async fn main() {
    let word = match std::env::args().nth(1) {
        Some(arg) => arg,
        _ => "rust".to_string()
    };

    // is tran zh to en?
    let is_zh2en = !is_enword(&word);

    let body = match lookup(&word).await {
        Ok(body) => body,
        Err(e) => {
            panic!("rmall: {:?}", e)
        }
    };

    let meaning = get_meaning(body, is_zh2en);
    println!("{}", meaning.trim());
}