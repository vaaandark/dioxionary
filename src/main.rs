use reqwest;
use tokio;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // is tran zh to en?
    let mut is_zh2en = false;

    // header
    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
    );

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let url = match std::env::args().nth(1) {
        Some(word) => {
            for c in word.chars() {
                if !c.is_ascii_alphabetic() {
                    is_zh2en = true;
                }
            }
            let mut s = "https://www.youdao.com/result?word=".to_string();
            s.push_str(&word);
            s.push_str("&lang=en");
            s
        }
        None => {
            eprintln!("No words provided!");
            "https://www.youdao.com/result?word=rust&lang=en".into()
        }
    };

    println!("look up in youdao at {}", url);

    let res = client.get(url).send().await?;

//    eprintln!("Response: {:?} {}", res.version(), res.status());
//    eprintln!("Headers: {:#?}\n", res.headers());

    let body = res.text().await?;
    let html = Html::parse_document(&body);
    if is_zh2en {
        let trans = Selector::parse("ul.basic").unwrap();
        for trans in html.select(&trans) {
            let vp = trans.text().collect::<Vec<_>>();
            for t in vp {
                if t.as_bytes()[0].is_ascii_digit() {
                    println!();
                } else {
                    println!("{}", t);
                }
            }
        }
    } else {
        let trans = Selector::parse(".trans").unwrap();
        for trans in html.select(&trans) {
            let vtran = trans.text().collect::<Vec<_>>();
            for t in vtran {
                println!("\n{}", t);
            }
        }
    }

    Ok(())
}