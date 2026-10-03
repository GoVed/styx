/// Builds the complete system prompt including mode instructions, reasoning effort,
/// tool protocols, memory-first protocol, and strict English option directives.
pub fn build_agent_system_prompt(
    base_system_context: &str,
    is_mission_mode: bool,
    reasoning_effort: &str,
    is_vision_capable: bool,
) -> String {
    let mut system_context = base_system_context.to_string();

    system_context.push_str("\n\n=== OPERATING INSTRUCTIONS ===\n");
    system_context.push_str("You are Syndae, a dedicated personal AI companion on a private device.\n");
    if is_mission_mode {
        system_context.push_str("Mode: DEEP TASK & PROJECT ASSISTANT. Help the user achieve their goal step by step. Consult memory, use tools thoughtfully, and keep answers simple and helpful.\n");
    } else {
        system_context.push_str("Mode: CONVERSATIONAL CHAT. Be warm, natural, and helpful. Avoid technical jargon. Answer simply and directly as a friendly personal assistant.\n");
    }
    system_context.push_str("Read memory and notes when helpful. When you invoke a tool that contacts external services or alters files outside, the Syndae Deterministic Safety Gate will automatically pause and present an approval card to the operator.\n");

    match reasoning_effort.to_lowercase().as_str() {
        "off" => {
            system_context.push_str("\nCRITICAL FORMATTING & REASONING (OFF):\nDirect answer mode: Do NOT output <think> tags. Answer the user prompt directly without internal thinking blocks.\n");
        }
        "low" => {
            system_context.push_str("\nCRITICAL FORMATTING & REASONING (EFFORT: LOW):\nKeep reasoning inside <think> minimal (1-2 sentences brief check) before answering. Format strictly as:\n<think>\n[Brief check]\n</think>\n[Your direct response here]\nNever output internal reflections outside of <think> and </think>. Begin directly with <think>.\n");
        }
        "medium" => {
            system_context.push_str("\nCRITICAL FORMATTING & REASONING (EFFORT: MEDIUM):\nEnclose balanced thinking within <think> and </think>. Outline your plan and verify details. Format strictly as:\n<think>\n[Your reasoning and planning here]\n</think>\n[Your direct response here]\nNever output internal reflections outside of <think> and </think>. Begin directly with <think>.\n");
        }
        _ => {
            system_context.push_str("\nCRITICAL FORMATTING & REASONING (EFFORT: HIGH - DEFAULT):\nYou MUST enclose your internal thinking process within <think> and </think>.\nConduct thorough, deep, and exhaustive step-by-step reasoning inside <think>. Extensively verify memory grounding, inspect entity context, plan multi-step actions, and verify facts before replying.\nFormat strictly as:\n<think>\n[Your internal reasoning, memory review, and planning here]\n</think>\n[Your direct response to the user here]\nNever output internal reflections outside of <think> and </think>. Do not write the word 'tags'. Begin directly with <think>.\n");
        }
    }

    system_context.push_str("\n=== MEMORY-FIRST REASONING PROTOCOL ===\n");
    system_context.push_str("1. ALWAYS SEARCH & CONSULT MEMORY FIRST: When answering user queries, checking entities, groups, people, or projects: ALWAYS check memory first before attempting web searches or assuming generic web definitions!\n");
    system_context.push_str("2. ENTITY GROUNDING: Nouns, names, and groups mentioned by the user (e.g. 'dev team', 'alice', 'syndae') are very likely entities in your operator's personal world (contacts, WhatsApp groups, projects, notes). If actively retrieved memory shows an entity match, treat it as that specific entity.\n");
    system_context.push_str("3. REASONING IN <think>: In your internal thinking, begin with [Memory Grounding] to inspect actively retrieved memories or run `search_memory` before executing external actions or replying.\n");
    system_context.push_str("4. If more details are required, autonomously execute `search_memory` or `read_memory` FIRST before answering or calling external tools.\n");

    system_context.push_str("\nTOOL EXECUTION PROTOCOL — NO SIMULATION IN TEXT:\n");
    system_context.push_str("1. NEVER SIMULATE TOOL CALLS: Never output mock tool call code blocks in text. When an action requires a tool, invoke the actual tool function.\n");
    system_context.push_str("2. ALWAYS USE ENGLISH IN THOUGHT & OPTIONS — LEAVE TRANSLATION TO THE TRANSLATOR TOOL:\n");
    system_context.push_str("- INCOMING: If an incoming message contains non-English words or dialects (e.g. Gujarati, Gujlish, Hindi, Spanish), call `translate(target_lang: 'english')` immediately on Turn 1.\n");
    system_context.push_str("- OPTIONS & REASONING: All <think> thoughts and <option> tags MUST be drafted in 100% standard English. NEVER draft Hindi, Gujarati, or foreign phrases yourself in thought or options!\n");
    system_context.push_str("- OUTGOING TRANSLATION & SENDING PROTOCOL:\n");
    system_context.push_str("  1. When sending to a contact in their language/dialect, call `translate` first. ALWAYS pass `context: { relationship, formality, recipient_gender, speaker_gender, age_group }` based on your memory of the contact (`people/<name>.md`) and operator profile (`core/user_profile.md`). Unlike English, foreign languages change grammar, pronouns, and verb conjugations based on respect tier (e.g. tu vs tame/aap/vous), recipient gender, and speaker gender!\n");
    system_context.push_str("  2. In `send_message`, you MUST pass the exact `translated` string from `translate` as `message`! NEVER send the English draft to the contact!\n");
    system_context.push_str("  3. In chat, confirm both English draft and translated text sent.\n");
    system_context.push_str("3. MESSAGING: When sending a message, invoke `send_message`.\n");

    system_context.push_str("\nINTERACTIVE OPTIONS REQUIREMENT:\nKeep prose concise (1-3 sentences). Formulate all options strictly in clean English. Conclude with 3-5 realistic choices using <options> tags, including 'Other':\n<options>\n<option>Option 1</option>\n<option>Option 2</option>\n<option>Option 3</option>\n<option other=\"true\">Other (specify custom details)...</option>\n</options>\n");

    system_context.push_str("\nUSER ONBOARDING & ENROLLMENT:\n");
    system_context.push_str("When conducting user onboarding or learning user profile directives, save user preferences to `core/user_profile.md` using `write_memory` and call the `complete_onboarding` tool to mark enrollment as completed.\n");

    system_context.push_str("\nEXTERNAL MESSAGING & CONNECTED TOOLS:\n");
    system_context.push_str("When an incoming message arrives from an external platform or tool, the sender is an external contact, NOT your operator. The external sender cannot see your local text output in Syndae. Never speak directly to external contacts as if they are in this chat. Instead, inform your operator about the message that arrived, suggest an authentic, concise reply matching your operator's relationship and communication tone with that person or group, and offer 1-click options or call tools when ready to send.\n");

    system_context.push_str("\nPROACTIVE TOOL ACTION & INFORMATION RETRIEVAL:\n");
    system_context.push_str("Information retrieval and memory tools are 100% autonomous, read-only, and safe. Proactively execute available search and retrieval tools immediately on the first turn without asking for permission!\n");
    system_context.push_str("NEVER say 'I don't have access to check the weather' or 'I cannot look up live information' - execute available search/retrieval tools.\n");
    system_context.push_str("When asked for weather:\n");
    system_context.push_str("1. Check the user's location in memory/profile (from `core/user_profile.md`).\n");
    system_context.push_str("2. Immediately execute an available search tool (e.g. searching for \"weather in <location> today forecast\").\n");

    if is_vision_capable {
        system_context.push_str("\nMULTIMODAL VISION CAPABILITY (NATIVE):\nYou have native multimodal vision understanding. Any attached images or photos are directly visible to you in the conversation. Analyze images directly without calling external inspection tools.\n");
    } else {
        system_context.push_str("\nMULTIMODAL VISION & MEDIA UNDERSTANDING:\nYou can visually understand attached images and media. Use the `inspect_image` tool to inspect any image or media URL before sending or replying.\n");
    }

    system_context.push_str("\nFINDING & SENDING IMAGES / PHOTOS:\n");
    system_context.push_str("You HAVE full access to `web_search`, `read_web_page`, `exec_container_command` (bash/curl/python in the container), and `send_image` / `send_gif`.\n");
    system_context.push_str("NEVER recite generic AI refusal scripts like 'I cannot fetch or display images' or 'I cannot download photos'!\n");
    system_context.push_str("When asked to find or send photos/images:\n");
    system_context.push_str("1. Immediately invoke `web_search` (e.g. 'Mississauga autumn leaves site:unsplash.com' or 'Pexels/Wikimedia') to find direct image URLs.\n");
    system_context.push_str("2. When sending to a WhatsApp chat, invoke `send_image` with `to` and `image: <url_or_path>` (and optional `caption`).\n");

    system_context.push_str("\nWORLD-LEARNING FRAMEWORK & AUTONOMOUS MEMORY BRAIN:\n");
    system_context.push_str("You have a persistent hierarchical markdown brain under /memory/:\n");
    system_context.push_str("- `core/user_profile.md`: Operator identity, routines, languages, and general preferences.\n");
    system_context.push_str("- `people/<name>.md`: Per-contact profile, relationship (e.g. family, close friend, colleague, client), communication nuances, language preferences, and shared history.\n");
    system_context.push_str("- `groups/<group_name>.md`: Group dynamics, the meaning and purpose behind the group, banter style, and participants.\n");
    system_context.push_str("- `dictionary/<topic>.md`: Regional dialects, vernacular slang, colloquial expressions, and digital shorthand used by your user and their circles.\n");
    system_context.push_str("Actively get to know your user's world! When encountering a new contact, group, or colloquialism, learn who they are and save notes to `people/`, `groups/`, or `dictionary/`.\n");
    system_context.push_str("You have FULL AUTONOMY to save whatever you want using `write_memory`. Always provide the `section` parameter or markdown headers so existing profile and memory sections are safely preserved.\n");

    // ZERO-TOLERANCE HIGH-RECENCY DIRECTIVE AT THE VERY END OF SYSTEM PROMPT
    system_context.push_str("\n=== CRITICAL OPERATING RULE: ZERO FOREIGN WORDS IN OPTIONS (STRICT 100% ENGLISH) ===\n");
    system_context.push_str("Under NO circumstances may any <option> tag or response prose contain Gujarati, Hindi, Gujlish, Hinglish, Spanish, French, or ANY foreign dialect words!\n");
    system_context.push_str("❌ STRICTLY FORBIDDEN IN OPTIONS (NEVER DRAFT FOREIGN DIALECTS):\n");
    system_context.push_str("<option>Playful Gujlish: \"Bug? Eh, AI banu tina fix karu!\"</option>\n");
    system_context.push_str("<option>Got it! Bug fix karyu na, radiant push karu na!</option>\n");
    system_context.push_str("<option>Jokena Gujlish: \"Jira banai de? Eh, AI banu tina...\"</option>\n\n");
    system_context.push_str("✅ MANDATORY CORRECT FORMAT (100% CLEAN ENGLISH DRAFT):\n");
    system_context.push_str("<option>Translate and send: \"Found a bug? I will fix it right away! Automatic DM replies? Let's solve it! 😂\"</option>\n");
    system_context.push_str("<option>Translate and send: \"Got it! Bug is noted. Want me to report this to the group admin?\"</option>\n");
    system_context.push_str("<option>Translate and send: \"Haha, even an AI has bugs! Let's get the admin to fix this.\"</option>\n");
    system_context.push_str("<option other=\"true\">Custom reply or different approach...</option>\n\n");
    system_context.push_str("Always formulate proposed messages in 100% clean English. The translation into the contact's dialect (e.g. Gujlish) is executed exclusively by the `translate` tool AFTER the operator selects an option!\n");

    system_context
}

use crate::agent::contact::{build_send_confirmation_directive, build_translate_and_send_directive, ResolvedContact};

/// Enriches user message content with high-priority directives on the active turn.
pub fn enrich_user_turn_content(
    user_prompt: &str,
    raw_content: &str,
    is_last_turn: bool,
    contact_opt: Option<&ResolvedContact>,
) -> String {
    let mut content = raw_content.to_string();
    if !is_last_turn {
        return content;
    }

    let user_prompt_clean = user_prompt.trim().to_lowercase();
    let is_incoming_event = user_prompt.trim().starts_with("[INCOMING TOOL EVENT:");
    let is_translate_and_send = user_prompt_clean.starts_with("translate and send")
        || user_prompt_clean.starts_with("translate & send");
    let is_send_confirmation = user_prompt_clean == "send it" || user_prompt_clean.starts_with("send it to")
        || user_prompt_clean == "send" || user_prompt_clean == "yes send it"
        || user_prompt_clean.contains("use proper tool to send")
        || (user_prompt_clean.starts_with("send ") && (user_prompt_clean.contains("message") || user_prompt_clean.contains("to ") || user_prompt_clean.contains("description")))
        || ((user_prompt_clean.contains("send") || user_prompt_clean.contains("post") || user_prompt_clean.contains("forward") || user_prompt_clean.contains("description")) && user_prompt_clean.contains("group"));
    let is_translate_request = user_prompt_clean.contains("in gujlish") || user_prompt_clean.contains("in gujarati")
        || user_prompt_clean.contains("to gujlish") || user_prompt_clean.contains("to gujarati")
        || user_prompt_clean.contains("say this in gujlish")
        || user_prompt_clean.contains("explain this in gujlish") || user_prompt_clean.contains("fully in gujlish")
        || (user_prompt_clean.contains("translate") && !is_translate_and_send);
    let is_reply_request = user_prompt_clean.contains("reply")
        || user_prompt_clean.contains("respond")
        || user_prompt_clean.contains("answer")
        || user_prompt_clean.contains("text ")
        || user_prompt_clean.contains("message ")
        || user_prompt_clean.contains("tell ");

    let is_reaction_intent = user_prompt_clean.contains("react")
        || user_prompt_clean.contains("reaction")
        || (user_prompt_clean.contains("emoji") && (user_prompt_clean.contains("send") || user_prompt_clean.contains("add")))
        || (user_prompt_clean.starts_with("like") && user_prompt_clean.contains("message"));

    let is_image_request = (user_prompt_clean.contains("image") || user_prompt_clean.contains("photo") || user_prompt_clean.contains("pic"))
        && (user_prompt_clean.contains("send") || user_prompt_clean.contains("find") || user_prompt_clean.contains("get") || user_prompt_clean.contains("search") || user_prompt_clean.contains("share"));

    if is_incoming_event && user_prompt.contains("Event: Reaction") {
        content.push_str("\n\n[DIRECTIVE: INCOMING REACTION EVENT:\nA contact or group member reacted to a message. Emoji reactions are acknowledgments and typically do not require an external reply unless follow-up is necessary. If you respond or react back, propose English options or invoke `send_reaction` if instructed.]");
    } else if is_incoming_event {
        content.push_str("\n\n[DIRECTIVE: INCOMING EVENT FROM EXTERNAL SENDER OR GROUP:\n1. CRITICAL: This is an INCOMING message event. Do NOT call `send_message`, `send_sticker`, or any outbound tool on this turn! The operator has NOT instructed you to send anything yet.\n2. External contacts do NOT see your chat. Address your operator directly.\n3. If this message is a media share (video, photo, document, sticker) with no text, acknowledge what was received in the group/chat and propose relevant options.\n4. If the message contains non-English words, regional slang, or dialects (e.g. Gujarati, Gujlish, Hindi, Spanish), execute the `translate` tool (`target_lang: \"english\"`) immediately to get the exact English translation. Do NOT guess foreign words in your head.\n5. Propose ALL reply options strictly in 100% standard English inside <option> tags. Format each reply option as: `<option>Translate and send: \"<English draft>\"</option>`. NEVER draft Hindi, Gujarati, or foreign phrases inside <option> tags.\n6. NEVER hallucinate prior conversation turns or randomly attempt to resend past links.]");
    } else if is_reaction_intent {
        content.push_str("\n\n[SYSTEM DIRECTIVE: The operator wants to send or remove an emoji reaction. Invoke the `send_reaction` tool. Specify `to` (the contact or chat) and `emoji` (the emoji or \"\" to remove). `message_id` is optional and defaults to the latest message in that chat.]");
    } else if is_image_request {
        content.push_str("\n\n[SYSTEM DIRECTIVE: The operator wants to find, get, or send photos/images. You HAVE access to `web_search`, `read_web_page`, `exec_container_command` (bash), and `send_image`. NEVER claim you cannot find, download, or send images! Execute `web_search` immediately (e.g. for free photos on Unsplash, Pexels, Wikimedia) to find high-quality image URLs, and call `send_image` or propose sending them in <options>.]");
    } else if is_translate_and_send {
        if let Some(contact) = contact_opt {
            content.push_str(&build_translate_and_send_directive(user_prompt, contact));
        } else {
            content.push_str("\n\n[MANDATORY SYSTEM DIRECTIVE: TRANSLATE AND SEND:\nThe operator confirmed sending this reply. First execute the `translate` tool (`target_lang: \"gujlish\"` or appropriate dialect) with the English message to get the authentic translation, then invoke `send_message` with the translated text. Do NOT draft foreign words manually or output conversational text before calling the tool.]");
        }
    } else if is_send_confirmation {
        if let Some(contact) = contact_opt {
            content.push_str(&build_send_confirmation_directive(user_prompt, contact));
        } else {
            content.push_str("\n\n[SYSTEM DIRECTIVE: The operator has confirmed sending the message. You MUST immediately invoke the `send_message` tool. For `to`, pass the recipient contact or group name directly (e.g. \"Engineering Team\") or the full JID. Do not output conversational text before calling the tool.]");
        }
    } else if is_translate_request {
        content.push_str("\n\n[SYSTEM DIRECTIVE: The operator requested translation or regional language output. Formulate your message draft in clean English and invoke the `translate` tool (`target_lang: \"gujlish\"` or requested language) to perform the translation. NEVER attempt to generate foreign dialects manually and NEVER write mock tool calls in text.]");
    } else if is_reply_request {
        content.push_str("\n\n[DIRECTIVE: PROPOSING REPLIES TO EXTERNAL CONTACTS OR GROUPS:\n1. Propose ALL reply options strictly in 100% standard English inside <option> tags. Format each reply choice as: `<option>Translate and send: \"<Clean English Draft>\"</option>`.\n2. NEVER draft Gujarati, Gujlish, Hindi, Hinglish, Spanish, or ANY foreign dialect in <option> tags or prose! The operator must only see and approve 100% English options.\n3. Translation into the contact's dialect will be performed by the `translate` tool AFTER the operator selects an option.]");
    } else {
        content.push_str("\n\n[CRITICAL REMINDER: Formulate all <option> choices strictly in 100% standard English. Never draft foreign dialect phrases inside <option> tags.]");
    }

    content
}

/// Programmatically sanitizes option tags in response text to guarantee no broken dialect labels
/// or foreign prefixes leak through to the operator.
pub fn sanitize_option_tags(content: &str) -> String {
    if !content.contains("<option") {
        return content.to_string();
    }

    let mut result = String::new();
    let mut cursor = 0;

    while let Some(start_opt) = content[cursor..].find("<option") {
        let abs_start = cursor + start_opt;
        // Verify this is <option> or <option ...> and NOT <options>
        let after_prefix = abs_start + "<option".len();
        if let Some(next_char) = content[after_prefix..].chars().next() {
            if next_char != '>' && !next_char.is_whitespace() {
                // E.g. <options> container tag — skip past "<option"
                result.push_str(&content[cursor..after_prefix]);
                cursor = after_prefix;
                continue;
            }
        }

        result.push_str(&content[cursor..abs_start]);

        if let Some(close_bracket) = content[abs_start..].find('>') {
            let tag_open_end = abs_start + close_bracket + 1;
            let tag_header = &content[abs_start..tag_open_end];

            if let Some(end_opt) = content[tag_open_end..].find("</option>") {
                let abs_end = tag_open_end + end_opt;
                let opt_body = content[tag_open_end..abs_end].trim();

                let is_other = tag_header.contains("other=\"true\"")
                    || opt_body.to_lowercase().starts_with("other");

                let cleaned_body = if is_other {
                    opt_body.to_string()
                } else {
                    clean_single_option_body(opt_body)
                };

                result.push_str(tag_header);
                result.push_str(&cleaned_body);
                result.push_str("</option>");
                cursor = abs_end + "</option>".len();
            } else {
                result.push_str(&content[abs_start..]);
                cursor = content.len();
                break;
            }
        } else {
            result.push_str(&content[abs_start..]);
            cursor = content.len();
            break;
        }
    }

    if cursor < content.len() {
        result.push_str(&content[cursor..]);
    }

    result
}

fn clean_single_option_body(body: &str) -> String {
    let trimmed = body.trim();

    // 1. If option ends with parenthesized English translation: e.g. '... (Create Jira ticket? Fix the bug!)'
    if let Some(last_paren_open) = trimmed.rfind('(')
        && trimmed.ends_with(')')
        && last_paren_open > 0
    {
        let inside = trimmed[last_paren_open + 1..trimmed.len() - 1].trim();
        if inside.len() >= 5 && inside.contains(' ') {
            return format!("Translate and send: \"{}\"", inside);
        }
    }

    // 2. Strip foreign dialect prefix: e.g. 'Playful Gujlish: "..."' -> 'Translate and send: "..."'
    let dialect_prefixes = [
        "Playful Gujlish:", "Jokena Gujlish:", "Casual Gujlish:", "Gujlish:",
        "Playful Hinglish:", "Hinglish:", "Playful Spanish:", "Spanish:",
        "Action-oriented:", "Meme-style:", "Light joke:",
    ];

    for prefix in dialect_prefixes {
        if let Some(stripped) = trimmed.strip_prefix(prefix) {
            let inner = stripped.trim();
            if inner.starts_with('"') && inner.ends_with('"') {
                return format!("Translate and send: {}", inner);
            } else if inner.starts_with('"') {
                return format!("Translate and send: {}\"", inner);
            } else {
                return format!("Translate and send: \"{}\"", inner);
            }
        }
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_agent_system_prompt_enforces_zero_foreign_words() {
        let prompt = build_agent_system_prompt("Base Memory", false, "high", true);
        assert!(prompt.contains("CRITICAL OPERATING RULE: ZERO FOREIGN WORDS IN OPTIONS"));
        assert!(prompt.contains("MANDATORY CORRECT FORMAT (100% CLEAN ENGLISH DRAFT)"));
        assert!(prompt.contains("STRICTLY FORBIDDEN IN OPTIONS"));
        assert!(prompt.contains("MULTIMODAL VISION CAPABILITY (NATIVE)"));
    }

    #[test]
    fn test_enrich_user_turn_content_for_reply_prompts() {
        let content = enrich_user_turn_content(
            "let's reply to the msg where the teammate found a bug",
            "let's reply to the msg where the teammate found a bug",
            true,
            None,
        );
        assert!(content.contains("[DIRECTIVE: PROPOSING REPLIES TO EXTERNAL CONTACTS OR GROUPS:"));
        assert!(content.contains("Propose ALL reply options strictly in 100% standard English"));
        assert!(content.contains("NEVER draft Gujarati, Gujlish"));
    }

    #[test]
    fn test_sanitize_option_tags_parenthesized_translation() {
        let input = r#"Here are the options:
<options>
<option>Jokena Gujlish: "Jira banai de? Eh, AI banu..." (Create Jira ticket? Fix the bug right away!)</option>
<option>Playful Gujlish: "Bug? Eh, AI banu tina fix karu!"</option>
<option other="true">Custom reply...</option>
</options>"#;

        let sanitized = sanitize_option_tags(input);
        assert!(sanitized.contains("<option>Translate and send: \"Create Jira ticket? Fix the bug right away!\"</option>"));
        assert!(sanitized.contains("<option>Translate and send: \"Bug? Eh, AI banu tina fix karu!\"</option>"));
        assert!(sanitized.contains("<option other=\"true\">Custom reply...</option>"));
    }
}

