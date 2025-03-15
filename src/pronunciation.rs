use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

fn build_pronuciation_url(word: &str) -> String {
    format!("https://dict.youdao.com/dictvoice?audio={}&type=1", word)
}

pub fn pronounce(word: &str) -> Result<()> {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let url = build_pronuciation_url(word);
    let response = reqwest::blocking::get(url)?;
    let inner = response.bytes()?;
    if let Ok(source) = Decoder::new(Cursor::new(inner)) {
        if let Ok(sink) = Sink::try_new(&stream_handle) {
            sink.append(source);
            sink.sleep_until_end();
        }
    }
    Ok(())
}
