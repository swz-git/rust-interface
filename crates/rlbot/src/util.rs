use std::env;

pub struct RLBotEnvironment {
    pub server_addr: String,
    pub agent_id: String,
}

impl RLBotEnvironment {
    // Reads from environment variables RLBOT_SERVER_ADDR/RLBOT_SERVER_PORT and RLBOT_AGENT_ID
    pub fn from_env() -> Self {
        let server_addr = env::var("RLBOT_SERVER_ADDR").unwrap_or(format!(
            "127.0.0.1:{}",
            env::var("RLBOT_SERVER_PORT").unwrap_or("23234".into())
        ));
        let agent_id = env::var("RLBOT_AGENT_ID").unwrap_or("".into());

        Self {
            server_addr,
            agent_id,
        }
    }
}
