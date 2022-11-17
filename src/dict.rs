use reqwest;
pub use scraper::{Html, Selector};

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
    static APP_USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Mobile Safari/537.36";
    let client = reqwest::Client::builder().user_agent(APP_USER_AGENT).build()?;

    let url = generate_url(word);
    //println!("look up in youdao at {}", url);

    let res = client.get(url).send().await?;
    let body = res.text().await?;
    Ok(body)
}

pub fn zh2en(html: &Html) -> String {
    let mut meaning = String::new();
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
    meaning
}

pub fn en2zh(html: &Html) -> String {
    let mut meaning = String::new();
    let phonetic = Selector::parse(".phonetic").unwrap();
    for phonetic in html.select(&phonetic) {
        let vp = phonetic.text().collect::<Vec<_>>();
        for p in vp {
            meaning.push_str(p);
            meaning.push_str(" ");
        }
    }
    meaning.push_str("\n");
    let trans = Selector::parse(".trans").unwrap();
    for trans in html.select(&trans) {
        let vt = trans.text().collect::<Vec<_>>();
        for t in vt {
            meaning.push_str(t);
            meaning.push_str("\n");
        }
    }
    meaning
}

pub fn get_exam_type(html: &Html) -> Vec<String> {
    let types = Selector::parse(".exam_type-value").unwrap();
    let mut vtype: Vec<String> = Vec::new();
    for types in html.select(&types) {
        let vtp = types.text().collect::<Vec<_>>();
        for t in vtp {
            vtype.push(t.to_string());
        }
    }
    vtype
}
