use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPolicyRecord {
    pub channel_id: String,
    pub protocol: String,
    pub channel_name: Option<String>,
    pub is_group: bool,
    pub policy: String, // "all", "mentions", "muted", "manual"
    pub mention_keywords: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDefaultsRecord {
    pub default_group_policy: String,  // default "mentions"
    pub default_direct_policy: String, // default "all"
    pub mention_keywords: String,      // default "syndae,assistant,ai,bot"
}

impl Database {
    pub async fn record_channel_if_not_exists(
        &self,
        channel_id: &str,
        protocol: &str,
        channel_name: Option<&str>,
        is_group: bool,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        let default_policy = if is_group { "mentions" } else { "all" };
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR IGNORE INTO channel_policies (channel_id, protocol, channel_name, is_group, policy, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![channel_id, protocol, channel_name, is_group as i32, default_policy, now],
        )?;
        if let Some(name) = channel_name {
            if !name.is_empty() {
                conn.execute(
                    "UPDATE channel_policies SET channel_name = ?1, updated_at = ?2 WHERE channel_id = ?3 AND (channel_name IS NULL OR channel_name = '')",
                    params![name, now, channel_id],
                )?;
            }
        }
        Ok(())
    }

    pub async fn get_channel_policy(&self, channel_id: &str) -> Result<Option<ChannelPolicyRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT channel_id, protocol, channel_name, is_group, policy, mention_keywords, updated_at
             FROM channel_policies WHERE channel_id = ?1",
        )?;
        let mut rows = stmt.query(params![channel_id])?;
        if let Some(r) = rows.next()? {
            let is_group_int: i32 = r.get(3)?;
            Ok(Some(ChannelPolicyRecord {
                channel_id: r.get(0)?,
                protocol: r.get(1)?,
                channel_name: r.get(2)?,
                is_group: is_group_int != 0,
                policy: r.get(4)?,
                mention_keywords: r.get(5)?,
                updated_at: r.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn set_channel_policy(
        &self,
        channel_id: &str,
        protocol: &str,
        channel_name: Option<&str>,
        is_group: bool,
        policy: &str,
        mention_keywords: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO channel_policies (channel_id, protocol, channel_name, is_group, policy, mention_keywords, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(channel_id) DO UPDATE SET
                policy = excluded.policy,
                channel_name = COALESCE(excluded.channel_name, channel_policies.channel_name),
                is_group = excluded.is_group,
                mention_keywords = excluded.mention_keywords,
                updated_at = excluded.updated_at",
            params![
                channel_id,
                protocol,
                channel_name,
                is_group as i32,
                policy,
                mention_keywords,
                now
            ],
        )?;
        Ok(())
    }

    pub async fn list_channel_policies(&self, protocol: Option<&str>) -> Result<Vec<ChannelPolicyRecord>> {
        let conn = self.conn.lock().await;
        let mut result = Vec::new();
        let query = if protocol.is_some() {
            "SELECT channel_id, protocol, channel_name, is_group, policy, mention_keywords, updated_at
             FROM channel_policies WHERE protocol = ?1 ORDER BY updated_at DESC"
        } else {
            "SELECT channel_id, protocol, channel_name, is_group, policy, mention_keywords, updated_at
             FROM channel_policies ORDER BY updated_at DESC"
        };
        let mut stmt = conn.prepare(query)?;
        let mut rows = if let Some(p) = protocol {
            stmt.query(params![p])?
        } else {
            stmt.query([])?
        };
        while let Some(r) = rows.next()? {
            let is_group_int: i32 = r.get(3)?;
            result.push(ChannelPolicyRecord {
                channel_id: r.get(0)?,
                protocol: r.get(1)?,
                channel_name: r.get(2)?,
                is_group: is_group_int != 0,
                policy: r.get(4)?,
                mention_keywords: r.get(5)?,
                updated_at: r.get(6)?,
            });
        }
        Ok(result)
    }

    pub async fn get_channel_defaults(&self) -> Result<ChannelDefaultsRecord> {
        let default_group_policy = self
            .get_setting("tool_default_group_policy")
            .await?
            .unwrap_or_else(|| "mentions".to_string());
        let default_direct_policy = self
            .get_setting("tool_default_direct_policy")
            .await?
            .unwrap_or_else(|| "all".to_string());
        let mention_keywords = self
            .get_setting("tool_mention_keywords")
            .await?
            .unwrap_or_else(|| "syndae,assistant,ai,bot".to_string());
        Ok(ChannelDefaultsRecord {
            default_group_policy,
            default_direct_policy,
            mention_keywords,
        })
    }

    pub async fn set_channel_defaults(&self, defaults: &ChannelDefaultsRecord) -> Result<()> {
        self.set_setting("tool_default_group_policy", &defaults.default_group_policy).await?;
        self.set_setting("tool_default_direct_policy", &defaults.default_direct_policy).await?;
        self.set_setting("tool_mention_keywords", &defaults.mention_keywords).await?;
        Ok(())
    }

    pub async fn evaluate_channel_trigger(
        &self,
        channel_id: &str,
        protocol: &str,
        channel_name: Option<&str>,
        is_group: bool,
        text: &str,
        reply_info: &str,
        operator_name: &str,
        is_memory_ignored: bool,
    ) -> Result<(bool, String, String)> {
        if is_memory_ignored {
            return Ok((false, "muted".into(), "Channel marked as ignored in operator memory directives".into()));
        }

        let _ = self.record_channel_if_not_exists(channel_id, protocol, channel_name, is_group).await;

        let defaults = self.get_channel_defaults().await.unwrap_or(ChannelDefaultsRecord {
            default_group_policy: "mentions".into(),
            default_direct_policy: "all".into(),
            mention_keywords: "syndae,assistant,ai,bot".into(),
        });

        let policy_record = self.get_channel_policy(channel_id).await?;
        let effective_policy = match policy_record.as_ref().map(|p| p.policy.as_str()) {
            Some(p) => p.to_string(),
            None => {
                if is_group {
                    defaults.default_group_policy
                } else {
                    defaults.default_direct_policy
                }
            }
        };

        match effective_policy.to_lowercase().as_str() {
            "muted" | "block" | "blocked" | "ignore" | "ignored" => {
                Ok((false, "muted".into(), "Channel trigger policy is set to Muted".into()))
            }
            "manual" => {
                Ok((false, "manual".into(), "Channel trigger policy is set to Manual".into()))
            }
            "all" | "always" => {
                Ok((true, "all".into(), "Channel trigger policy is set to Always Respond".into()))
            }
            "mentions" | _ => {
                if !is_group && policy_record.is_none() {
                    return Ok((true, "all".into(), "Direct 1-on-1 chat default".into()));
                }

                let text_lower = text.to_lowercase();
                let reply_lower = reply_info.to_lowercase();

                let mut keywords: Vec<String> = vec!["syndae".to_string(), "bot".to_string(), "assistant".to_string()];
                if !operator_name.is_empty() {
                    keywords.push(operator_name.to_lowercase());
                }

                let custom_kws = policy_record
                    .as_ref()
                    .and_then(|p| p.mention_keywords.as_deref())
                    .unwrap_or(&defaults.mention_keywords);

                for kw in custom_kws.split(',') {
                    let trimmed = kw.trim().to_lowercase();
                    if !trimmed.is_empty() {
                        keywords.push(trimmed);
                    }
                }

                let has_keyword = keywords.iter().any(|kw| {
                    text_lower.contains(kw) || text_lower.contains(&format!("@{}", kw))
                });

                let is_reply_to_me = !reply_lower.is_empty()
                    && (reply_lower.contains("syndae")
                        || (!operator_name.is_empty() && reply_lower.contains(&operator_name.to_lowercase())));

                if has_keyword || is_reply_to_me {
                    Ok((true, "mentions".into(), "Mention keyword or reply detected in message".into()))
                } else {
                    Ok((
                        false,
                        "mentions".into(),
                        "Group message ignored: trigger policy is Mentions Only and no mention was found".into(),
                    ))
                }
            }
        }
    }
}
