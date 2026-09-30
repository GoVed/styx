use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    Token(String),
    Thought(String),
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: String,
    },
    Done,
    Error(String),
}

/// Helper stateful parser to extract `<think>...</think>` or `<thought>...</thought>` blocks during token streaming
pub struct StreamTagParser {
    inside_think: bool,
    pending: String,
}

impl StreamTagParser {
    pub fn new() -> Self {
        Self {
            inside_think: false,
            pending: String::new(),
        }
    }

    pub fn process(&mut self, incoming: &str) -> Vec<(bool, String)> {
        let mut results = Vec::new();
        self.pending.push_str(incoming);

        loop {
            if !self.inside_think {
                if let Some(open_idx) = self.pending.find('<') {
                    if open_idx > 0 {
                        let text = self.pending[..open_idx].to_string();
                        results.push((false, text));
                        self.pending.drain(..open_idx);
                    }

                    if self.pending.starts_with("<think>") {
                        self.pending.drain(.."<think>".len());
                        self.inside_think = true;
                        continue;
                    } else if self.pending.starts_with("<thought>") {
                        self.pending.drain(.."<thought>".len());
                        self.inside_think = true;
                        continue;
                    } else if self.pending.starts_with("</think>") {
                        self.pending.drain(.."</think>".len());
                        continue;
                    } else if self.pending.starts_with("</thought>") {
                        self.pending.drain(.."</thought>".len());
                        continue;
                    } else if "<think>".starts_with(&self.pending)
                        || "<thought>".starts_with(&self.pending)
                        || "</think>".starts_with(&self.pending)
                        || "</thought>".starts_with(&self.pending)
                    {
                        break;
                    } else {
                        results.push((false, "<".to_string()));
                        self.pending.drain(..1);
                        continue;
                    }
                } else {
                    if !self.pending.is_empty() {
                        results.push((false, std::mem::take(&mut self.pending)));
                    }
                    break;
                }
            } else {
                if let Some(close_idx) = self.pending.find('<') {
                    if close_idx > 0 {
                        let thought_text = self.pending[..close_idx].to_string();
                        results.push((true, thought_text));
                        self.pending.drain(..close_idx);
                    }

                    if self.pending.starts_with("</think>") {
                        self.pending.drain(.."</think>".len());
                        self.inside_think = false;
                        continue;
                    } else if self.pending.starts_with("</thought>") {
                        self.pending.drain(.."</thought>".len());
                        self.inside_think = false;
                        continue;
                    } else if "</think>".starts_with(&self.pending) || "</thought>".starts_with(&self.pending) {
                        break;
                    } else {
                        results.push((true, "<".to_string()));
                        self.pending.drain(..1);
                        continue;
                    }
                } else {
                    if !self.pending.is_empty() {
                        results.push((true, std::mem::take(&mut self.pending)));
                    }
                    break;
                }
            }
        }

        results
    }

    pub fn flush(&mut self) -> Vec<(bool, String)> {
        let mut results = Vec::new();
        if !self.pending.is_empty() {
            results.push((self.inside_think, std::mem::take(&mut self.pending)));
        }
        results
    }
}
