use anyhow::Result;

pub trait SpacedRepetiton: Sized + Default {
    /// find next reviewable word
    fn next_to_review(&self) -> Result<Option<String>>;

    fn add_fresh_word(&mut self, w: String) -> Result<()>;

    fn update(&mut self, question: String, q: u8) -> Result<()>;

    fn remove(&mut self, question: &str) -> Result<()>;
}
