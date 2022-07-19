mod dict;
use tokio;

#[tokio::main]
async fn main() {
    let word = match std::env::args().nth(1) {
        Some(arg) => arg,
        _ => "rust".to_string()
    };

    // is tran zh to en?
    let is_zh2en = !dict::is_enword(&word);

    let body = match dict::lookup(&word).await {
        Ok(body) => body,
        Err(e) => {
            panic!("rmall: {:?}", e)
        }
    };

    let meaning = dict::get_meaning(body, is_zh2en);
    println!("{}", meaning.trim());
}