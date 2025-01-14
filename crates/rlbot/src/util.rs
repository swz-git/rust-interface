use std::env;

pub struct RLBotEnvironment {
    /// Will fallback to 127.0.0.1:23234
    pub server_addr: String,
    /// No fallback and therefor Option<>
    pub agent_id: Option<String>,
}

impl RLBotEnvironment {
    // Reads from environment variables RLBOT_SERVER_ADDR/(RLBOT_SERVER_IP & RLBOT_SERVER_PORT) and RLBOT_AGENT_ID
    pub fn from_env() -> Self {
        let server_addr = env::var("RLBOT_SERVER_ADDR").unwrap_or(format!(
            "{}:{}",
            env::var("RLBOT_SERVER_IP").unwrap_or("127.0.0.1".into()),
            env::var("RLBOT_SERVER_PORT").unwrap_or("23234".into())
        ));
        let mut agent_id = env::var("RLBOT_AGENT_ID").ok();

        agent_id = match agent_id {
            Some(s) if s.is_empty() => None,
            _ => agent_id,
        };

        Self {
            server_addr,
            agent_id,
        }
    }
}
