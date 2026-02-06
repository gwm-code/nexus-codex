use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: None,
            base_url: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderSettings {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Gemini,
    OpenRouter,
    OpenCode,
    Claude,
}

pub trait Provider {
    fn kind(&self) -> ProviderKind;
    fn display_name(&self) -> &'static str;
    fn dry_run_prompt(&self, input: &str) -> String;
    fn send_prompt(&self, input: &str) -> anyhow::Result<String>;
}

pub struct GeminiProvider {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
}

pub struct OpenRouterProvider {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
}

pub struct OpenCodeProvider {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
}

pub struct ClaudeProvider {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
}

impl Provider for GeminiProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Gemini
    }

    fn display_name(&self) -> &'static str {
        "Gemini"
    }

    fn dry_run_prompt(&self, input: &str) -> String {
        format!(
            "[Gemini dry-run] Would send prompt of {} chars. API key {}",
            input.len(),
            if self.api_key.is_some() { "loaded" } else { "missing" }
        )
    }

    fn send_prompt(&self, input: &str) -> anyhow::Result<String> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Gemini API key not configured"))?;
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url, self.model, api_key
        );

        let payload = serde_json::json!({
            "contents": [{
                "parts": [{"text": input}]
            }]
        });

        let response: serde_json::Value = reqwest::blocking::Client::new()
            .post(url)
            .json(&payload)
            .send()?
            .error_for_status()?
            .json()?;

        let text = response["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(text)
    }
}

impl Provider for OpenRouterProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenRouter
    }

    fn display_name(&self) -> &'static str {
        "OpenRouter"
    }

    fn dry_run_prompt(&self, input: &str) -> String {
        format!(
            "[OpenRouter dry-run] Would send prompt of {} chars. API key {}",
            input.len(),
            if self.api_key.is_some() { "loaded" } else { "missing" }
        )
    }

    fn send_prompt(&self, input: &str) -> anyhow::Result<String> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not configured"))?;
        let payload = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": input}],
        });
        let response: serde_json::Value = reqwest::blocking::Client::new()
            .post(&self.base_url)
            .bearer_auth(api_key)
            .header("HTTP-Referer", "https://localhost")
            .header("X-Title", "Nexus")
            .json(&payload)
            .send()?
            .error_for_status()?
            .json()?;
        let text = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(text)
    }
}

impl Provider for OpenCodeProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenCode
    }

    fn display_name(&self) -> &'static str {
        "OpenCode"
    }

    fn dry_run_prompt(&self, input: &str) -> String {
        format!(
            "[OpenCode dry-run] Would send prompt of {} chars. API key {}",
            input.len(),
            if self.api_key.is_some() { "loaded" } else { "missing" }
        )
    }

    fn send_prompt(&self, input: &str) -> anyhow::Result<String> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenCode API key not configured"))?;
        let payload = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": input}],
        });
        let response: serde_json::Value = reqwest::blocking::Client::new()
            .post(&self.base_url)
            .bearer_auth(api_key)
            .json(&payload)
            .send()?
            .error_for_status()?
            .json()?;
        let text = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(text)
    }
}

impl Provider for ClaudeProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Claude
    }

    fn display_name(&self) -> &'static str {
        "Claude"
    }

    fn dry_run_prompt(&self, input: &str) -> String {
        format!(
            "[Claude dry-run] Would send prompt of {} chars. API key {}",
            input.len(),
            if self.api_key.is_some() { "loaded" } else { "missing" }
        )
    }

    fn send_prompt(&self, input: &str) -> anyhow::Result<String> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Claude API key not configured"))?;
        let payload = serde_json::json!({
            "model": self.model,
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": input}],
        });
        let response: serde_json::Value = reqwest::blocking::Client::new()
            .post(&self.base_url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()?
            .error_for_status()?
            .json()?;
        let text = response["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(text)
    }
}

pub fn build_provider(kind: &ProviderKind, settings: ProviderSettings) -> Box<dyn Provider> {
    match kind {
        ProviderKind::Gemini => Box::new(GeminiProvider {
            api_key: settings.api_key,
            model: settings.model.unwrap_or_else(|| "gemini-1.5-pro".to_string()),
            base_url: settings
                .base_url
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string()),
        }),
        ProviderKind::OpenRouter => Box::new(OpenRouterProvider {
            api_key: settings.api_key,
            model: settings
                .model
                .unwrap_or_else(|| "openai/gpt-4o-mini".to_string()),
            base_url: settings
                .base_url
                .unwrap_or_else(|| "https://openrouter.ai/api/v1/chat/completions".to_string()),
        }),
        ProviderKind::OpenCode => Box::new(OpenCodeProvider {
            api_key: settings.api_key,
            model: settings.model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
            base_url: settings
                .base_url
                .unwrap_or_else(|| "https://api.opencode.ai/v1/chat/completions".to_string()),
        }),
        ProviderKind::Claude => Box::new(ClaudeProvider {
            api_key: settings.api_key,
            model: settings
                .model
                .unwrap_or_else(|| "claude-3-5-sonnet-20240620".to_string()),
            base_url: settings
                .base_url
                .unwrap_or_else(|| "https://api.anthropic.com/v1/messages".to_string()),
        }),
    }
}
