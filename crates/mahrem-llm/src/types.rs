//! İstek ve yanıt tipleri (OpenAI `chat/completions` biçiminin kullanılan alt kümesi).

use serde::{Deserialize, Serialize};

/// Mesajın sahibi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Sistem talimatı.
    System,
    /// Kullanıcı.
    User,
    /// Model.
    Assistant,
}

/// Sohbetteki bir mesaj.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Mesajın sahibi.
    pub role: Role,
    /// Metin.
    pub content: String,
}

impl Message {
    /// Sistem mesajı.
    pub fn system(content: impl Into<String>) -> Self {
        Message {
            role: Role::System,
            content: content.into(),
        }
    }

    /// Kullanıcı mesajı.
    pub fn user(content: impl Into<String>) -> Self {
        Message {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Model mesajı.
    pub fn assistant(content: impl Into<String>) -> Self {
        Message {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Yanıt biçimi.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Serbest metin.
    Text,
    /// Geçerli bir JSON nesnesi.
    JsonObject,
    /// Verilen şemaya uyan JSON.
    JsonSchema {
        /// `{"name": …, "schema": …}` nesnesi.
        json_schema: serde_json::Value,
    },
}

/// Sohbet isteği.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatRequest {
    /// Model adı.
    pub model: String,
    /// Mesajlar.
    pub messages: Vec<Message>,
    /// Örnekleme sıcaklığı.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Çekirdek örnekleme eşiği.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Örnekleme tohumu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Üretilecek en çok token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Yanıt biçimi.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    /// Akış kapalı; istemci akışı desteklemez.
    stream: bool,
}

impl ChatRequest {
    /// Örnekleme ayarı olmayan bir istek.
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        ChatRequest {
            model: model.into(),
            messages,
            temperature: None,
            top_p: None,
            seed: None,
            max_tokens: None,
            response_format: None,
            stream: false,
        }
    }

    /// Sıcaklığı ayarlar.
    #[must_use]
    pub fn temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }

    /// Çekirdek örnekleme eşiğini ayarlar.
    #[must_use]
    pub fn top_p(mut self, p: f64) -> Self {
        self.top_p = Some(p);
        self
    }

    /// Tohumu ayarlar.
    #[must_use]
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// En çok token sayısını ayarlar.
    #[must_use]
    pub fn max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = Some(n);
        self
    }

    /// Yanıt biçimini ayarlar.
    #[must_use]
    pub fn response_format(mut self, f: ResponseFormat) -> Self {
        self.response_format = Some(f);
        self
    }
}

/// Token kullanımı.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// İstemdeki token sayısı.
    #[serde(default)]
    pub prompt_tokens: u64,
    /// Üretilen token sayısı.
    #[serde(default)]
    pub completion_tokens: u64,
}

/// Sohbet yanıtı.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatResponse {
    /// Modelin ürettiği metin.
    pub content: String,
    /// Bitiş nedeni (`stop`, `length` …), sunucu verdiyse.
    pub finish_reason: Option<String>,
    /// Sunucunun bildirdiği model adı.
    pub model: Option<String>,
    /// Bu isteğin token kullanımı; sunucu vermediyse sıfır.
    pub usage: Usage,
}

#[derive(Deserialize)]
pub(crate) struct RawResponse {
    model: Option<String>,
    choices: Vec<RawChoice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct RawChoice {
    message: RawMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct RawMessage {
    content: Option<String>,
}

impl RawResponse {
    pub(crate) fn into_response(self) -> Result<ChatResponse, String> {
        let choice = self
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| "yanıtta seçenek yok".to_owned())?;
        let content = choice
            .message
            .content
            .ok_or_else(|| "yanıtta metin yok".to_owned())?;
        Ok(ChatResponse {
            content,
            finish_reason: choice.finish_reason,
            model: self.model,
            usage: self.usage.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes_only_set_fields() {
        let req = ChatRequest::new("m", vec![Message::system("s"), Message::user("u")])
            .seed(3)
            .response_format(ResponseFormat::JsonObject);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "model": "m",
                "messages": [
                    {"role": "system", "content": "s"},
                    {"role": "user", "content": "u"}
                ],
                "seed": 3,
                "response_format": {"type": "json_object"},
                "stream": false
            })
        );
    }
}
