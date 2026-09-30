use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyTier {
    Autonomous,
    RequireApproval,
    Blocked,
}

impl std::fmt::Display for PolicyTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyTier::Autonomous => write!(f, "AUTONOMOUS"),
            PolicyTier::RequireApproval => write!(f, "REQUIRE_APPROVAL"),
            PolicyTier::Blocked => write!(f, "BLOCKED"),
        }
    }
}

impl PolicyTier {
    pub fn from_str_lenient(s: &str) -> Self {
        match s.to_uppercase().trim() {
            "AUTONOMOUS" | "AUTO" => PolicyTier::Autonomous,
            "REQUIRE_APPROVAL" | "ASK" | "APPROVAL" => PolicyTier::RequireApproval,
            "BLOCKED" | "BLOCK" => PolicyTier::Blocked,
            _ => PolicyTier::RequireApproval,
        }
    }

    pub fn default_for_tool(name: &str) -> (Self, &'static str) {
        let n = name.to_lowercase();

        // Safe autonomous internal tools
        if n == "write_memory" || n == "complete_onboarding" || n == "read_memory" || n == "search_memory" || n == "inspect_image" {
            return (PolicyTier::Autonomous, "LOW");
        }

        // High-risk mutating patterns
        if n.contains("exec")
            || n.contains("bash")
            || n.contains("shell")
            || n.contains("kill")
            || n.contains("drop")
        {
            return (PolicyTier::RequireApproval, "CRITICAL");
        }

        if n.contains("send")
            || n.contains("write")
            || n.contains("delete")
            || n.contains("remove")
            || n.contains("restart")
            || n.contains("modify")
            || n.contains("update")
            || n.contains("create")
        {
            return (PolicyTier::RequireApproval, "HIGH");
        }

        // Safe read-only patterns
        (PolicyTier::Autonomous, "LOW")
    }
}
