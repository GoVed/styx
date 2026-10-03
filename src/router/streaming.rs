use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::tags::{parse_ifm_block, parse_json_block};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolTagFormat {
    Ifm,
    Json,
}

const KNOWN_TAGS: &[&str] = &[
    "<think>", "</think>",
    "<thought>", "</thought>",
    "<ifm|think>", "</ifm|think>",
    "<ifm|tool_calls>", "</ifm|tool_calls>",
    "<ifm|tool_call>", "</ifm|tool_call>",
    "<function_calls>", "</function_calls>",
    "<tool_calls>", "</tool_calls>",
    "<tool_call>", "</tool_call>",
];

/// Stateful parser to extract thoughts and native XML/JSON tool calls during token streaming
pub struct StreamTagParser {
    inside_think: bool,
    inside_tool_call: Option<ToolTagFormat>,
    pending: String,
    tool_buffer: String,
    tool_index: usize,
}

impl StreamTagParser {
    pub fn new() -> Self {
        Self {
            inside_think: false,
            inside_tool_call: None,
            pending: String::new(),
            tool_buffer: String::new(),
            tool_index: 0,
        }
    }

    pub fn process(&mut self, incoming: &str) -> Vec<StreamChunk> {
        let mut results = Vec::new();
        self.pending.push_str(incoming);

        loop {
            // 1. If currently inside a tool call block, buffer into tool_buffer and check for closing tag
            if let Some(format) = self.inside_tool_call {
                self.tool_buffer.push_str(&std::mem::take(&mut self.pending));

                let (closed, block_end, tag_len) = match format {
                    ToolTagFormat::Ifm => {
                        if let Some(idx) = self.tool_buffer.find("</ifm|tool_calls>") {
                            (true, idx, "</ifm|tool_calls>".len())
                        } else if !self.tool_buffer.contains("<ifm|tool_calls>")
                            && let Some(idx) = self.tool_buffer.find("</ifm|tool_call>")
                        {
                            (true, idx, "</ifm|tool_call>".len())
                        } else {
                            (false, 0, 0)
                        }
                    }
                    ToolTagFormat::Json => {
                        let tags = ["</function_calls>", "</tool_calls>", "</tool_call>"];
                        let found = tags.iter().filter_map(|t| self.tool_buffer.find(t).map(|idx| (idx, t.len()))).min_by_key(|(idx, _)| *idx);
                        if let Some((idx, len)) = found {
                            (true, idx, len)
                        } else {
                            (false, 0, 0)
                        }
                    }
                };

                if closed {
                    let block = self.tool_buffer[..block_end].to_string();
                    let leftover = self.tool_buffer[block_end + tag_len..].to_string();
                    self.tool_buffer.clear();
                    self.inside_tool_call = None;
                    self.pending = leftover;

                    let calls = match format {
                        ToolTagFormat::Ifm => parse_ifm_block(&block),
                        ToolTagFormat::Json => parse_json_block(&block),
                    };

                    for (name, args) in calls {
                        results.push(StreamChunk::ToolCallDelta {
                            index: self.tool_index,
                            id: Some(format!("call_{}", Uuid::new_v4().simple())),
                            name: Some(name),
                            arguments_delta: args,
                        });
                        self.tool_index += 1;
                    }
                    continue;
                } else {
                    break;
                }
            }

            // 2. If inside think tag
            if self.inside_think {
                if let Some(open_idx) = self.pending.find('<') {
                    if open_idx > 0 {
                        let text = self.pending[..open_idx].to_string();
                        results.push(StreamChunk::Thought(text));
                        self.pending.drain(..open_idx);
                    }

                    if self.pending.starts_with("</think>") {
                        self.pending.drain(.."</think>".len());
                        self.inside_think = false;
                        continue;
                    } else if self.pending.starts_with("</thought>") {
                        self.pending.drain(.."</thought>".len());
                        self.inside_think = false;
                        continue;
                    } else if self.pending.starts_with("</ifm|think>") {
                        self.pending.drain(.."</ifm|think>".len());
                        self.inside_think = false;
                        continue;
                    } else if ["</think>", "</thought>", "</ifm|think>"].iter().any(|t| t.starts_with(&self.pending)) {
                        break;
                    } else {
                        results.push(StreamChunk::Thought("<".to_string()));
                        self.pending.drain(..1);
                        continue;
                    }
                } else {
                    if !self.pending.is_empty() {
                        results.push(StreamChunk::Thought(std::mem::take(&mut self.pending)));
                    }
                    break;
                }
            }

            // 3. Normal text stream
            if let Some(open_idx) = self.pending.find('<') {
                if open_idx > 0 {
                    let text = self.pending[..open_idx].to_string();
                    results.push(StreamChunk::Token(text));
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
                } else if self.pending.starts_with("<ifm|think>") {
                    self.pending.drain(.."<ifm|think>".len());
                    self.inside_think = true;
                    continue;
                } else if self.pending.starts_with("</think>") {
                    self.pending.drain(.."</think>".len());
                    continue;
                } else if self.pending.starts_with("</thought>") {
                    self.pending.drain(.."</thought>".len());
                    continue;
                } else if self.pending.starts_with("</ifm|think>") {
                    self.pending.drain(.."</ifm|think>".len());
                    continue;
                } else if self.pending.starts_with("<ifm|tool_calls>") {
                    self.pending.drain(.."<ifm|tool_calls>".len());
                    self.inside_tool_call = Some(ToolTagFormat::Ifm);
                    self.tool_buffer.push_str("<ifm|tool_calls>");
                    continue;
                } else if self.pending.starts_with("<ifm|tool_call>") {
                    self.pending.drain(.."<ifm|tool_call>".len());
                    self.inside_tool_call = Some(ToolTagFormat::Ifm);
                    self.tool_buffer.push_str("<ifm|tool_call>");
                    continue;
                } else if self.pending.starts_with("<function_calls>") {
                    self.pending.drain(.."<function_calls>".len());
                    self.inside_tool_call = Some(ToolTagFormat::Json);
                    continue;
                } else if self.pending.starts_with("<tool_calls>") {
                    self.pending.drain(.."<tool_calls>".len());
                    self.inside_tool_call = Some(ToolTagFormat::Json);
                    continue;
                } else if self.pending.starts_with("<tool_call>") {
                    self.pending.drain(.."<tool_call>".len());
                    self.inside_tool_call = Some(ToolTagFormat::Json);
                    continue;
                } else if self.pending.starts_with("</ifm|tool_calls>") {
                    self.pending.drain(.."</ifm|tool_calls>".len());
                    continue;
                } else if self.pending.starts_with("</ifm|tool_call>") {
                    self.pending.drain(.."</ifm|tool_call>".len());
                    continue;
                } else if self.pending.starts_with("</function_calls>") {
                    self.pending.drain(.."</function_calls>".len());
                    continue;
                } else if self.pending.starts_with("</tool_calls>") {
                    self.pending.drain(.."</tool_calls>".len());
                    continue;
                } else if self.pending.starts_with("</tool_call>") {
                    self.pending.drain(.."</tool_call>".len());
                    continue;
                } else if KNOWN_TAGS.iter().any(|t| t.starts_with(&self.pending)) {
                    break;
                } else {
                    results.push(StreamChunk::Token("<".to_string()));
                    self.pending.drain(..1);
                    continue;
                }
            } else {
                if !self.pending.is_empty() {
                    results.push(StreamChunk::Token(std::mem::take(&mut self.pending)));
                }
                break;
            }
        }

        results
    }

    pub fn flush(&mut self) -> Vec<StreamChunk> {
        let mut results = Vec::new();
        if self.inside_tool_call.is_some() || !self.tool_buffer.is_empty() {
            let full = format!("{}{}", self.tool_buffer, self.pending);
            self.tool_buffer.clear();
            self.pending.clear();
            self.inside_tool_call = None;

            let calls = if full.contains("<ifm|") {
                parse_ifm_block(&full)
            } else {
                parse_json_block(&full)
            };

            if !calls.is_empty() {
                for (name, args) in calls {
                    results.push(StreamChunk::ToolCallDelta {
                        index: self.tool_index,
                        id: Some(format!("call_{}", Uuid::new_v4().simple())),
                        name: Some(name),
                        arguments_delta: args,
                    });
                    self.tool_index += 1;
                }
            } else if !full.is_empty() {
                results.push(StreamChunk::Token(full));
            }
        } else if self.inside_think {
            if !self.pending.is_empty() {
                results.push(StreamChunk::Thought(std::mem::take(&mut self.pending)));
            }
        } else if !self.pending.is_empty() {
            results.push(StreamChunk::Token(std::mem::take(&mut self.pending)));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_tag_parser_ifm_tool_call() {
        let mut parser = StreamTagParser::new();
        let chunks = parser.process("<ifm|tool_calls>\n<ifm|tool_call>send_message\n<ifm|arg_key>message</ifm|arg_key>\n<ifm|arg_value>Hello world</ifm|arg_value>\n<ifm|arg_key>to</ifm|arg_key>\n<ifm|arg_value>Engineering Team</ifm|arg_value>\n</ifm|tool_call>\n</ifm|tool_calls>");
        assert_eq!(chunks.len(), 1);
        if let StreamChunk::ToolCallDelta { name, arguments_delta, .. } = &chunks[0] {
            assert_eq!(name.as_deref(), Some("send_message"));
            let parsed: serde_json::Value = serde_json::from_str(arguments_delta).unwrap();
            assert_eq!(parsed["message"], "Hello world");
            assert_eq!(parsed["to"], "Engineering Team");
        } else {
            panic!("Expected ToolCallDelta, got {:?}", chunks[0]);
        }
    }

    #[test]
    fn test_stream_tag_parser_json_tool_call() {
        let mut parser = StreamTagParser::new();
        let chunks = parser.process("<function_calls>\n[{\"name\": \"translate\", \"arguments\": {\"text\": \"hi\", \"target_lang\": \"gujlish\"}}]}\n</function_calls>");
        assert_eq!(chunks.len(), 1);
        if let StreamChunk::ToolCallDelta { name, arguments_delta, .. } = &chunks[0] {
            assert_eq!(name.as_deref(), Some("translate"));
            let parsed: serde_json::Value = serde_json::from_str(arguments_delta).unwrap();
            assert_eq!(parsed["text"], "hi");
            assert_eq!(parsed["target_lang"], "gujlish");
        } else {
            panic!("Expected ToolCallDelta, got {:?}", chunks[0]);
        }
    }

    #[test]
    fn test_stream_tag_parser_thoughts_and_text() {
        let mut parser = StreamTagParser::new();
        let c1 = parser.process("<think>analyzing</think>Hello ");
        let c2 = parser.process("world!");
        let all = [c1, c2].concat();
        assert_eq!(all, vec![
            StreamChunk::Thought("analyzing".into()),
            StreamChunk::Token("Hello ".into()),
            StreamChunk::Token("world!".into()),
        ]);
    }
}
