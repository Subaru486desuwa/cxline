use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct SessionData {
    pub model: Option<String>,
    pub token_usage: Option<TokenUsage>,
    pub cost: Option<CostInfo>,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub permission: Option<String>,
    pub elapsed_seconds: Option<f64>,
    pub git_branch: Option<String>,
    pub git_dirty: Option<bool>,
    pub turn: Option<u32>,

    /// Capture unknown fields for forward compatibility
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[allow(dead_code)]
pub struct TokenUsage {
    pub used: Option<u64>,
    pub total: Option<u64>,
    pub input: Option<u64>,
    pub output: Option<u64>,
    pub cached: Option<u64>,
    pub reasoning: Option<u64>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[allow(dead_code)]
pub struct CostInfo {
    pub total: Option<f64>,
    pub currency: Option<String>,
    pub input_cost: Option<f64>,
    pub output_cost: Option<f64>,
}

/// Parse a JSON string into SessionData, returning Default on failure
pub fn parse_input(input: &str) -> SessionData {
    serde_json::from_str(input).unwrap_or_default()
}

// --- Codex JSONL event types ---

/// Raw JSONL line from Codex session log
#[derive(Debug, Deserialize)]
pub struct CodexLine {
    #[allow(dead_code)]
    pub timestamp: Option<String>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Parsed Codex event
#[derive(Debug)]
pub enum CodexEvent {
    SessionMeta {
        id: String,
        cwd: Option<String>,
        cli_version: Option<String>,
        model_provider: Option<String>,
    },
    TurnContext {
        turn_id: String,
        model: Option<String>,
        approval_policy: Option<String>,
        cwd: Option<String>,
    },
    TaskStarted {
        turn_id: String,
        model_context_window: Option<u64>,
    },
    TokenCount {
        input_tokens: Option<u64>,
        cached_input_tokens: Option<u64>,
        output_tokens: Option<u64>,
        reasoning_output_tokens: Option<u64>,
        model_context_window: Option<u64>,
    },
    TaskComplete {
        #[allow(dead_code)]
        turn_id: String,
    },
    Unknown,
}

/// Parse a single JSONL line into a CodexEvent
pub fn parse_codex_line(line: &str) -> Option<CodexEvent> {
    let raw: CodexLine = serde_json::from_str(line).ok()?;
    let p = &raw.payload;

    match raw.event_type.as_str() {
        "session_meta" => Some(CodexEvent::SessionMeta {
            id: p.get("id")?.as_str()?.to_string(),
            cwd: p.get("cwd").and_then(|v| v.as_str()).map(|s| s.to_string()),
            cli_version: p.get("cli_version").and_then(|v| v.as_str()).map(|s| s.to_string()),
            model_provider: p.get("model_provider").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }),
        "turn_context" => Some(CodexEvent::TurnContext {
            turn_id: p.get("turn_id")?.as_str()?.to_string(),
            model: p.get("model").and_then(|v| v.as_str()).map(|s| s.to_string()),
            approval_policy: p.get("approval_policy").and_then(|v| v.as_str()).map(|s| s.to_string()),
            cwd: p.get("cwd").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }),
        "event_msg" => {
            let msg_type = p.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match msg_type {
                "task_started" => Some(CodexEvent::TaskStarted {
                    turn_id: p.get("turn_id")?.as_str()?.to_string(),
                    model_context_window: p.get("model_context_window").and_then(|v| v.as_u64()),
                }),
                "token_count" => {
                    let info = p.get("info");
                    let total_usage = info.and_then(|i| i.get("total_token_usage"));
                    let ctx_window = info.and_then(|i| i.get("model_context_window")).and_then(|v| v.as_u64());
                    Some(CodexEvent::TokenCount {
                        input_tokens: total_usage.and_then(|u| u.get("input_tokens")).and_then(|v| v.as_u64()),
                        cached_input_tokens: total_usage.and_then(|u| u.get("cached_input_tokens")).and_then(|v| v.as_u64()),
                        output_tokens: total_usage.and_then(|u| u.get("output_tokens")).and_then(|v| v.as_u64()),
                        reasoning_output_tokens: total_usage.and_then(|u| u.get("reasoning_output_tokens")).and_then(|v| v.as_u64()),
                        model_context_window: ctx_window,
                    })
                }
                "task_complete" => Some(CodexEvent::TaskComplete {
                    turn_id: p.get("turn_id")?.as_str()?.to_string(),
                }),
                _ => Some(CodexEvent::Unknown),
            }
        }
        _ => Some(CodexEvent::Unknown),
    }
}

/// Aggregated Codex session state, built incrementally from events
#[derive(Debug, Default)]
pub struct CodexSession {
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub cli_version: Option<String>,
    pub model_provider: Option<String>,
    pub model: Option<String>,
    pub approval_policy: Option<String>,
    pub model_context_window: Option<u64>,
    pub input_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reasoning_output_tokens: Option<u64>,
    pub turn_count: u32,
    pub start_time: Option<std::time::Instant>,
    seen_turn_ids: Vec<String>,
}

impl CodexSession {
    pub fn new() -> Self {
        Self {
            start_time: Some(std::time::Instant::now()),
            ..Default::default()
        }
    }

    /// Apply a single event to update session state
    pub fn apply_event(&mut self, event: &CodexEvent) {
        match event {
            CodexEvent::SessionMeta { id, cwd, cli_version, model_provider } => {
                self.session_id = Some(id.clone());
                if cwd.is_some() { self.cwd = cwd.clone(); }
                if cli_version.is_some() { self.cli_version = cli_version.clone(); }
                if model_provider.is_some() { self.model_provider = model_provider.clone(); }
            }
            CodexEvent::TurnContext { turn_id, model, approval_policy, cwd } => {
                if model.is_some() { self.model = model.clone(); }
                if approval_policy.is_some() { self.approval_policy = approval_policy.clone(); }
                if cwd.is_some() { self.cwd = cwd.clone(); }
                if !self.seen_turn_ids.contains(turn_id) {
                    self.seen_turn_ids.push(turn_id.clone());
                    self.turn_count += 1;
                }
            }
            CodexEvent::TaskStarted { turn_id, model_context_window } => {
                if let Some(w) = model_context_window {
                    self.model_context_window = Some(*w);
                }
                if !self.seen_turn_ids.contains(turn_id) {
                    self.seen_turn_ids.push(turn_id.clone());
                    self.turn_count += 1;
                }
            }
            CodexEvent::TokenCount {
                input_tokens, cached_input_tokens, output_tokens,
                reasoning_output_tokens, model_context_window,
            } => {
                if input_tokens.is_some() { self.input_tokens = *input_tokens; }
                if cached_input_tokens.is_some() { self.cached_input_tokens = *cached_input_tokens; }
                if output_tokens.is_some() { self.output_tokens = *output_tokens; }
                if reasoning_output_tokens.is_some() { self.reasoning_output_tokens = *reasoning_output_tokens; }
                if let Some(w) = model_context_window {
                    self.model_context_window = Some(*w);
                }
            }
            CodexEvent::TaskComplete { .. } => {}
            CodexEvent::Unknown => {}
        }
    }

    /// Convert to SessionData for rendering via existing modules
    pub fn to_session_data(&self) -> SessionData {
        let total_used = [
            self.input_tokens,
            self.output_tokens,
            self.reasoning_output_tokens,
        ]
        .iter()
        .filter_map(|v| *v)
        .sum::<u64>();

        let token_usage = if total_used > 0 || self.model_context_window.is_some() {
            Some(TokenUsage {
                used: if total_used > 0 { Some(total_used) } else { None },
                total: self.model_context_window,
                input: self.input_tokens,
                output: self.output_tokens,
                cached: self.cached_input_tokens,
                reasoning: self.reasoning_output_tokens,
            })
        } else {
            None
        };

        let elapsed = self.start_time.map(|t| t.elapsed().as_secs_f64());

        SessionData {
            model: self.model.clone(),
            token_usage,
            cost: None,
            session_id: self.session_id.clone(),
            cwd: self.cwd.clone(),
            permission: self.approval_policy.clone(),
            elapsed_seconds: elapsed,
            git_branch: None,
            git_dirty: None,
            turn: if self.turn_count > 0 { Some(self.turn_count) } else { None },
            extra: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full() {
        let json = r#"{"model":"o3-mini","token_usage":{"used":12500,"total":128000},"cost":{"total":0.42}}"#;
        let data = parse_input(json);
        assert_eq!(data.model.as_deref(), Some("o3-mini"));
        assert_eq!(data.token_usage.as_ref().unwrap().used, Some(12500));
        assert_eq!(data.cost.as_ref().unwrap().total, Some(0.42));
    }

    #[test]
    fn test_parse_partial() {
        let json = r#"{"model":"gpt-4"}"#;
        let data = parse_input(json);
        assert_eq!(data.model.as_deref(), Some("gpt-4"));
        assert!(data.token_usage.is_none());
    }

    #[test]
    fn test_parse_invalid() {
        let data = parse_input("not json");
        assert!(data.model.is_none());
    }

    #[test]
    fn test_parse_empty() {
        let data = parse_input("{}");
        assert!(data.model.is_none());
    }

    #[test]
    fn test_unknown_fields() {
        let json = r#"{"model":"o3","future_field":"value"}"#;
        let data = parse_input(json);
        assert_eq!(data.model.as_deref(), Some("o3"));
        assert!(data.extra.contains_key("future_field"));
    }

    #[test]
    fn test_parse_codex_session_meta() {
        let line = r#"{"timestamp":"2026-03-04T11:38:17.368Z","type":"session_meta","payload":{"id":"abc123","cwd":"/home/user","cli_version":"0.104.0","model_provider":"openai"}}"#;
        let event = parse_codex_line(line).unwrap();
        match event {
            CodexEvent::SessionMeta { id, cwd, cli_version, .. } => {
                assert_eq!(id, "abc123");
                assert_eq!(cwd.as_deref(), Some("/home/user"));
                assert_eq!(cli_version.as_deref(), Some("0.104.0"));
            }
            _ => panic!("Expected SessionMeta"),
        }
    }

    #[test]
    fn test_parse_codex_token_count() {
        let line = r#"{"timestamp":"t","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":10259,"cached_input_tokens":4096,"output_tokens":836,"reasoning_output_tokens":622,"total_tokens":11095},"model_context_window":258400},"rate_limits":{}}}"#;
        let event = parse_codex_line(line).unwrap();
        match event {
            CodexEvent::TokenCount { input_tokens, cached_input_tokens, output_tokens, reasoning_output_tokens, model_context_window } => {
                assert_eq!(input_tokens, Some(10259));
                assert_eq!(cached_input_tokens, Some(4096));
                assert_eq!(output_tokens, Some(836));
                assert_eq!(reasoning_output_tokens, Some(622));
                assert_eq!(model_context_window, Some(258400));
            }
            _ => panic!("Expected TokenCount"),
        }
    }

    #[test]
    fn test_codex_session_aggregate() {
        let mut session = CodexSession::new();

        session.apply_event(&CodexEvent::SessionMeta {
            id: "sess1".to_string(),
            cwd: Some("/project".to_string()),
            cli_version: Some("0.104.0".to_string()),
            model_provider: Some("openai".to_string()),
        });

        session.apply_event(&CodexEvent::TurnContext {
            turn_id: "turn1".to_string(),
            model: Some("gpt-5.3-codex".to_string()),
            approval_policy: Some("on-request".to_string()),
            cwd: None,
        });

        session.apply_event(&CodexEvent::TokenCount {
            input_tokens: Some(10259),
            cached_input_tokens: Some(4096),
            output_tokens: Some(836),
            reasoning_output_tokens: Some(622),
            model_context_window: Some(258400),
        });

        assert_eq!(session.turn_count, 1);
        assert_eq!(session.model.as_deref(), Some("gpt-5.3-codex"));

        let data = session.to_session_data();
        assert_eq!(data.model.as_deref(), Some("gpt-5.3-codex"));
        assert_eq!(data.permission.as_deref(), Some("on-request"));
        assert_eq!(data.turn, Some(1));

        let usage = data.token_usage.unwrap();
        assert_eq!(usage.used, Some(10259 + 836 + 622));
        assert_eq!(usage.total, Some(258400));
        assert_eq!(usage.input, Some(10259));
        assert_eq!(usage.cached, Some(4096));
        assert_eq!(usage.reasoning, Some(622));
    }
}
