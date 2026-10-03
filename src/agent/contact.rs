/// Helper for extracting and resolving contact information, target recipient,
/// and language dialect from conversation history and incoming tool events.

#[derive(Debug, Clone, Default)]
pub struct ResolvedContact {
    pub sender_name: String,
    pub channel_id: String,
    pub channel_name: String,
    pub target_to: String,
    pub detected_dialect: String,
    pub relationship: String,
    pub recipient_gender: String,
    pub incoming_text: String,
}

pub fn resolve_contact_from_history(history_texts: &[String], memory_context: &str) -> ResolvedContact {
    let mut contact = ResolvedContact::default();

    for msg in history_texts.iter().rev() {
        if msg.contains("[INCOMING TOOL EVENT:") {
            for line in msg.lines() {
                let trimmed = line.trim();
                if let Some(s) = trimmed.strip_prefix("Sender:") {
                    contact.sender_name = s.trim().to_string();
                } else if let Some(c) = trimmed.strip_prefix("Channel:") {
                    let channel_str = c.trim();
                    if let Some(open_p) = channel_str.rfind('(')
                        && let Some(close_p) = channel_str.rfind(')')
                        && close_p > open_p
                    {
                        contact.channel_id = channel_str[open_p + 1..close_p].trim().to_string();
                        let group_part = channel_str[..open_p].trim();
                        contact.channel_name = group_part
                            .strip_prefix("Group:")
                            .unwrap_or(group_part)
                            .trim()
                            .to_string();
                    } else if let Some(direct) = channel_str.strip_prefix("Direct:") {
                        contact.channel_id = direct.trim().to_string();
                        contact.channel_name = direct.trim().to_string();
                    } else {
                        contact.channel_id = channel_str.to_string();
                    }
                } else if let Some(m) = trimmed.strip_prefix("Message:") {
                    contact.incoming_text = m.trim().trim_matches('"').to_string();
                }
            }
            break;
        }
    }

    // Target recipient preference: channel_id (JID) first, then sender name
    contact.target_to = if !contact.channel_id.is_empty() {
        contact.channel_id.clone()
    } else if !contact.sender_name.is_empty() {
        contact.sender_name.clone()
    } else {
        "external_contact".to_string()
    };

    detect_dialect_and_context(&mut contact, memory_context);
    contact
}

fn detect_dialect_and_context(contact: &mut ResolvedContact, memory_context: &str) {
    let lower_incoming = contact.incoming_text.to_lowercase();
    let _lower_sender = contact.sender_name.to_lowercase();
    let _lower_channel = contact.channel_name.to_lowercase();
    let lower_mem = memory_context.to_lowercase();

    let gujlish_words = [
        "jaldi", "avo", "aav", "aavo", "aje", "kal", "late", "ramay", "pachi", "karyu",
        "che", "bhai", "kem", "nathi", "chale", "su", "shu", "haji", "tame", "tamaru",
        "mane", "tane", "karo", "karu", "bau", "sarun", "pan", "chho",
    ];

    let hinglish_words = [
        "kya", "hai", "aao", "aaj", "baat", "nahi", "hoga", "yaar", "hum", "tum",
        "mera", "tera", "kaha", "bolo", "achha",
    ];

    let spanish_words = [
        "hola", "como", "estas", "amigo", "bueno", "gracias", "por", "favor", "bien",
    ];

    let has_gujlish_words = gujlish_words.iter().any(|&w| lower_incoming.split_whitespace().any(|tok| tok.trim_matches(|c: char| !c.is_alphabetic()) == w));
    let has_hinglish_words = hinglish_words.iter().any(|&w| lower_incoming.split_whitespace().any(|tok| tok.trim_matches(|c: char| !c.is_alphabetic()) == w));
    let has_spanish_words = spanish_words.iter().any(|&w| lower_incoming.split_whitespace().any(|tok| tok.trim_matches(|c: char| !c.is_alphabetic()) == w));

    if lower_mem.contains("brother") || lower_mem.contains("family") {
        contact.detected_dialect = if has_gujlish_words || lower_mem.contains("gujlish") { "gujlish" } else { "english" }.to_string();
        contact.relationship = "family".to_string();
        contact.recipient_gender = "neutral".to_string();
    } else if has_gujlish_words || lower_mem.contains("gujlish") {
        contact.detected_dialect = "gujlish".to_string();
        contact.relationship = "peer / friend".to_string();
        contact.recipient_gender = "neutral".to_string();
    } else if has_hinglish_words || lower_mem.contains("hinglish") {
        contact.detected_dialect = "hinglish".to_string();
        contact.relationship = "peer / friend".to_string();
        contact.recipient_gender = "neutral".to_string();
    } else if has_spanish_words || lower_mem.contains("spanish") {
        contact.detected_dialect = "spanish".to_string();
        contact.relationship = "friend / colleague".to_string();
        contact.recipient_gender = "neutral".to_string();
    } else {
        contact.detected_dialect = "english".to_string();
        contact.relationship = "contact / peer".to_string();
        contact.recipient_gender = "neutral".to_string();
    }
}

pub fn extract_draft_text(user_prompt: &str) -> String {
    let trimmed = user_prompt.trim();

    if let Some(first_quote) = trimmed.find('"')
        && let Some(last_quote) = trimmed.rfind('"')
        && last_quote > first_quote
    {
        return trimmed[first_quote + 1..last_quote].trim().to_string();
    }

    let prefixes = [
        "Translate and send:", "translate and send:", "Translate & send:",
        "translate & send:", "Send:", "send:", "Send it:", "send it:",
    ];

    for p in prefixes {
        if let Some(stripped) = trimmed.strip_prefix(p) {
            return stripped.trim().trim_matches('"').to_string();
        }
    }

    trimmed.to_string()
}

pub fn build_translate_and_send_directive(user_prompt: &str, contact: &ResolvedContact) -> String {
    let draft = extract_draft_text(user_prompt);
    let channel_display = if !contact.channel_name.is_empty() {
        format!("Group: {}", contact.channel_name)
    } else {
        contact.target_to.clone()
    };

    format!(
        "\n\n[MANDATORY SYSTEM DIRECTIVE: TRANSLATE AND SEND]\n\
        The operator confirmed sending this reply to {} ({}).\n\
        English draft to send: \"{}\"\n\
        Target language: \"{}\"\n\
        Target recipient (to): \"{}\"\n\n\
        STEP 1: You MUST execute the `translate` tool immediately on this turn:\n\
        Call: translate(\n  \
          text: \"{}\",\n  \
          target_lang: \"{}\",\n  \
          tone: \"casual\",\n  \
          context: {{\n    \
            \"relationship\": \"{}\",\n    \
            \"formality\": \"casual\",\n    \
            \"recipient_gender\": \"{}\",\n    \
            \"speaker_gender\": \"male\",\n    \
            \"age_group\": \"peer\"\n  \
          }}\n\
        )\n\n\
        CRITICAL RULES:\n\
        - Do NOT output conversational text before or instead of calling `translate`.\n\
        - Do NOT output the English draft \"{}\" into the chat.\n\
        - After translation returns, you will invoke `send_message(to: \"{}\", message: <translated_text>).\n",
        contact.sender_name, channel_display, draft, contact.detected_dialect, contact.target_to,
        draft, contact.detected_dialect, contact.relationship, contact.recipient_gender,
        draft, contact.target_to
    )
}

pub fn build_send_confirmation_directive(user_prompt: &str, contact: &ResolvedContact) -> String {
    let draft = extract_draft_text(user_prompt);
    let channel_display = if !contact.channel_name.is_empty() {
        format!("Group: {}", contact.channel_name)
    } else {
        contact.target_to.clone()
    };

    format!(
        "\n\n[MANDATORY SYSTEM DIRECTIVE: SEND MESSAGE]\n\
        The operator confirmed sending this message to {} ({}).\n\
        Message to send: \"{}\"\n\
        Target recipient (to): \"{}\"\n\n\
        YOU MUST IMMEDIATELY INVOKE THE `send_message` TOOL:\n\
        Call: send_message(to: \"{}\", message: \"{}\")\n\n\
        CRITICAL RULES:\n\
        - Do NOT output conversational text before or instead of calling `send_message`.\n\
        - Invoke the tool directly.\n",
        contact.sender_name, channel_display, draft, contact.target_to,
        contact.target_to, draft
    )
}

pub fn is_translate_and_send_intent(user_prompt: &str) -> bool {
    let p = user_prompt.trim().to_lowercase();
    p.starts_with("translate and send") || p.starts_with("translate & send")
}

pub fn is_send_confirmation_intent(user_prompt: &str) -> bool {
    let p = user_prompt.trim().to_lowercase();
    p == "send it" || p.starts_with("send it to") || p == "send" || p == "yes send it"
        || p.contains("use proper tool to send")
        || (p.starts_with("send ") && (p.contains("message") || p.contains("to ") || p.contains("description")))
        || ((p.contains("send") || p.contains("post") || p.contains("forward") || p.contains("description")) && p.contains("group"))
}

pub fn is_reaction_intent(user_prompt: &str) -> bool {
    let p = user_prompt.trim().to_lowercase();
    p.contains("react") || p.contains("reaction")
        || (p.contains("emoji") && (p.contains("send") || p.contains("add")))
        || (p.starts_with("like") && p.contains("message"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_draft_text() {
        assert_eq!(
            extract_draft_text("Translate and send: \"Coming now! What's the address?\""),
            "Coming now! What's the address?"
        );
        assert_eq!(
            extract_draft_text("Send: \"Got it! On my way!\""),
            "Got it! On my way!"
        );
    }

    #[test]
    fn test_resolve_contact_from_history() {
        let history = vec![
            "[INCOMING TOOL EVENT: WHATSAPP]\nSender: Test Peer\nChannel: Group: Peer Group (120363000000000001@g.us)\nMessage: \"Jaldi avo aje, Kal jetli late nai ramay pachi\"".to_string()
        ];
        let contact = resolve_contact_from_history(&history, "");
        assert_eq!(contact.sender_name, "Test Peer");
        assert_eq!(contact.channel_id, "120363000000000001@g.us");
        assert_eq!(contact.channel_name, "Peer Group");
        assert_eq!(contact.target_to, "120363000000000001@g.us");
        assert_eq!(contact.detected_dialect, "gujlish");
    }
}
