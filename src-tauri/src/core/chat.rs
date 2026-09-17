use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub thinking: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub tokens_count: Option<usize>,
    pub generation_speed_tps: Option<f32>,
    pub is_streaming: bool,
    #[serde(default)]
    pub metrics: Option<crate::core::model::GenerationMetrics>,
}


pub fn estimate_text_tokens(text: &str) -> usize {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return 0;
    }
    let words = trimmed.split_whitespace().count();
    let chars = trimmed.chars().count();
    let by_chars = (chars as f32 / 3.7).ceil() as usize;
    let by_words = (words as f32 * 1.25).ceil() as usize;
    by_chars.max(by_words)
}

impl ChatMessage {
    pub fn estimate_tokens(&self) -> usize {
        let content_toks = match self.tokens_count {
            Some(t) if t > 0 => t,
            _ => estimate_text_tokens(&self.content),
        };
        let think_toks = self.thinking.as_deref().map(estimate_text_tokens).unwrap_or(0);
        content_toks + think_toks
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: format!("msg-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            role: MessageRole::User,
            content: content.into(),
            thinking: None,
            timestamp: Utc::now(),
            tokens_count: None,
            generation_speed_tps: None,
            is_streaming: false,
            metrics: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: format!("msg-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            role: MessageRole::Assistant,
            content: content.into(),
            thinking: None,
            timestamp: Utc::now(),
            tokens_count: None,
            generation_speed_tps: None,
            is_streaming: false,
            metrics: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub model_id: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub archived_at: Option<DateTime<Utc>>,
}

impl ChatSession {
    pub fn new(title: impl Into<String>, model_id: Option<String>) -> Self {
        Self {
            id: format!("session-{}", Utc::now().timestamp_millis()),
            title: title.into(),
            model_id,
            messages: Vec::new(),
            created_at: Utc::now(),
            archived: None,
            archived_at: None,
        }
    }

    pub fn total_context_tokens(&self, system_prompt: &str) -> usize {
        let sys_toks = estimate_text_tokens(system_prompt);
        let msgs_toks: usize = self.messages.iter().map(|m| m.estimate_tokens()).sum();
        sys_toks + msgs_toks
    }

    #[allow(dead_code)]
    pub fn sample() -> Self {
        let mut session = Self::new("Análise de Código Rust & Otimizações", Some("meta-llama-3.1-8b-instruct-q4_k_m".into()));
        session.messages = vec![
            ChatMessage {
                id: "msg-1".into(),
                role: MessageRole::User,
                content: "Como o framework Iced gerencia o ciclo de vida da interface em Rust e como ele se compara ao Elm?".into(),
                thinking: None,
                timestamp: Utc::now(),
                tokens_count: Some(28),
                generation_speed_tps: None,
                is_streaming: false,
                metrics: None,
            },
            ChatMessage {
                id: "msg-2".into(),
                role: MessageRole::Assistant,
                content: "O **Iced** é inspirado diretamente na arquitetura The Elm Architecture (TEA). Seu ciclo de vida é fundamentado em 4 pilares:\n\n1. **Model (State):** O estado estruturado da sua aplicação.\n2. **Message (Events):** Enum com todas as ações que o usuário ou o sistema podem disparar.\n3. **Update:** Função pura onde o estado é atualizado com base na mensagem recebida, retornando opcionalmente um `Task` ou `Command` assíncrono.\n4. **View:** Função que constrói a árvore de widgets declarativos a partir do estado atual.\n\nEssa abordagem garante previsibilidade de estado, facilidade de testes unitários e evita mutações concorrentes descontroladas.".into(),
                thinking: Some("O usuário está perguntando sobre o Iced framework em Rust. Preciso explicar a TEA (The Elm Architecture) que é a base do Iced. Vou cobrir os 4 pilares: Model, Message, Update e View, e comparar com Elm para dar contexto.".into()),
                timestamp: Utc::now(),
                tokens_count: Some(142),
                generation_speed_tps: Some(48.5),
                is_streaming: false,
                metrics: None,
            },
        ];
        session
    }
}
