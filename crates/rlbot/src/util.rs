use std::{env, io::Write, mem};

use crate::{Packet, RLBotConnection, RLBotError};

pub struct RLBotEnvironment {
    /// Will fallback to 127.0.0.1:23234
    pub server_addr: String,
    /// No fallback and therefor Option<>
    pub agent_id: Option<String>,
}

impl RLBotEnvironment {
    // Reads from environment variables RLBOT_SERVER_ADDR/(RLBOT_SERVER_IP & RLBOT_SERVER_PORT) and RLBOT_AGENT_ID
    #[must_use]
    pub fn from_env() -> Self {
        let server_addr = env::var("RLBOT_SERVER_ADDR").unwrap_or_else(|_| {
            format!(
                "{}:{}",
                env::var("RLBOT_SERVER_IP").unwrap_or_else(|_| "127.0.0.1".into()),
                env::var("RLBOT_SERVER_PORT").unwrap_or_else(|_| "23234".into())
            )
        });

        let agent_id = env::var("RLBOT_AGENT_ID").ok().filter(|s| !s.is_empty());

        Self {
            server_addr,
            agent_id,
        }
    }
}

/// A queue of packets to be sent to RLBotServer
pub struct PacketQueue {
    internal_queue: Vec<Packet>,
}

impl Default for PacketQueue {
    fn default() -> Self {
        Self::new(16)
    }
}

impl PacketQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            internal_queue: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, packet: impl Into<Packet>) {
        self.internal_queue.push(packet.into());
    }

    pub(crate) fn empty(&mut self) -> Vec<Packet> {
        mem::take(&mut self.internal_queue)
    }
}

pub(crate) fn write_multiple_packets(
    connection: &mut RLBotConnection,
    packets: impl Iterator<Item = Packet>,
) -> Result<(), RLBotError> {
    let to_write = packets
        // convert Packet to Vec<u8> that RLBotServer can understand
        .flat_map(|x| {
            let data_type_bin = x.data_type().to_be_bytes().to_vec();
            let payload = x.build(&mut connection.builder);
            let data_len_bin = u16::try_from(payload.len())
                .expect("Payload can't be greater than a u16")
                .to_be_bytes()
                .to_vec();

            [data_type_bin, data_len_bin, payload].concat()
        })
        .collect::<Vec<_>>();

    connection.stream.write_all(&to_write)?;
    connection.stream.flush()?;

    Ok(())
}
