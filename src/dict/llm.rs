use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;

use super::{Dict, DictType, LookUpResult, LookUpResultItem};

const DEFAULT_PROMPT_TEMPLATE: &str = concat!(
    "如果文本是一个英文单词，则模仿双语词典格式（不要 Markdown 格式）输出读音、释义和例句，",
    "其他情况则直接翻译（直接翻译到另一种语言，不需要说明）：\n",
    "\n",
    "为下面的文本进行{{targets}}互译：\n",
    "{{text}}",
);

fn default_prompt_template() -> String {
    DEFAULT_PROMPT_TEMPLATE.to_string()
}

fn default_temperature() -> f64 {
    0.7
}

fn default_targets() -> Vec<String> {
    vec!["中文".to_string(), "English".to_string()]
}

#[derive(Deserialize, Debug, Default)]
pub struct LlmDict {
    pub name: String,
    pub model_name: String,
    pub api_url: String,
    pub api_keys: Vec<String>,
    #[serde(default = "default_targets")]
    pub targets: Vec<String>,
    #[serde(default = "default_prompt_template")]
    pub prompt_template: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
}

impl LlmDict {
    fn build_prompt(&self, text: impl Into<String>) -> String {
        let text = text.into();
        let targets_str = self.targets.join(", ");
        self.prompt_template
            .replace("{{text}}", &text)
            .replace("{{targets}}", &targets_str)
    }
}

impl LlmDict {
    fn chat(&self, prompt: impl Into<String>, api_key: &str) -> Result<String> {
        let client = Client::new();

        let payload = json!({
            "model": self.model_name,
            "messages": [{
                "role": "user",
                "content": prompt.into()
            }],
            "temperature": self.temperature,
        });

        let response = client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()?
            .error_for_status()?;

        let response_json: serde_json::Value = response.json()?;
        response_json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .with_context(|| "Invalid response format")
    }

    fn translate(&self, text: impl Into<String>) -> Result<String> {
        let prompt = self.build_prompt(text);
        let api_key = self
            .api_keys
            .get((self.api_keys.len() as f64 * rand::random::<f64>()) as usize)
            .with_context(|| "No API key available")?;
        self.chat(prompt, api_key)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_replacement() {
        let llm_dict = LlmDict {
            prompt_template: "Text: {{text}}. Targets: {{targets}}".to_string(),
            targets: vec!["Chinese".to_string(), "English".to_string()],
            ..Default::default()
        };

        let result = llm_dict.build_prompt("rust");
        assert_eq!(result, "Text: rust. Targets: Chinese, English");
    }

    #[test]
    fn test_translate() {
        let api_key = std::env::var("API_KEY");
        if api_key.is_err() {
            println!("API_KEY not set");
            return;
        }

        let llm_dict = LlmDict {
            name: "DeepSeek".to_string(),
            model_name: "deepseek-chat".to_string(),
            api_url: "https://api.deepseek.com/chat/completions".to_string(),
            api_keys: vec![api_key.unwrap()],
            targets: default_targets(),
            prompt_template: default_prompt_template(),
            temperature: default_temperature(),
        };

        for text in vec!["rust", "铁锈"] {
            let result = llm_dict.translate(text);
            assert!(result.is_ok());
            println!("{}", result.unwrap());
        }
    }
}

impl Dict for LlmDict {
    fn name(&self) -> &str {
        &self.name
    }

    fn type_(&self) -> DictType {
        DictType::LLM
    }

    fn supports_fuzzy_search(&self) -> bool {
        false
    }

    fn look_up(&self, _: bool, word: &str) -> super::LookUpResult {
        if let Ok(translation) = self.translate(word) {
            LookUpResult::Exact(LookUpResultItem::new(word, translation))
        } else {
            LookUpResult::None
        }
    }

    fn word_count(&self) -> Option<usize> {
        None
    }
}
