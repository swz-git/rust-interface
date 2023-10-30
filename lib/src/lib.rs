use std::{
    error::Error,
    io::{self, Write},
    net::TcpStream,
};

use flatbuffers::FlatBufferBuilder;
use thiserror::Error;

#[allow(non_snake_case)]
mod rlbot_generated;

pub mod flat_wrapper;

#[derive(Error, Debug)]
pub enum MatchManagerError {
    #[error("Connection to RLBot failed")]
    Connection(#[from] io::Error),
    #[error("Failed to start match")]
    MatchStart(String),
}

#[allow(dead_code)]
enum PacketDataType {
    GameTickPacket = 1,
    FieldInfo = 2,
    MatchSettings = 3,
    PlayerInput = 4,
    DesiredGameState = 7,
    RenderGroup = 8,
    QuickChat = 9,
    BallPrediction = 10,
    ReadyMessage = 11,
    MessagePacket = 12,
}
pub struct RLBotConnection {
    stream: TcpStream,
}

impl RLBotConnection {
    fn send_packet(
        &mut self,
        data_type: PacketDataType,
        payload: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let data_type_bin = (data_type as u16).to_be_bytes();
        let data_len_bin = (payload.len() as u16).to_be_bytes();

        self.stream.write_all(&data_type_bin)?;
        self.stream.write_all(&data_len_bin)?;
        self.stream.write_all(payload)?;
        Ok(())
    }

    pub fn start_match(
        &mut self,
        match_settings: flat_wrapper::MatchSettings,
    ) -> Result<(), MatchManagerError> {
        let match_settings_flat = {
            let mut builder = FlatBufferBuilder::new();
            let match_settings_flat = match_settings.to_flat(&mut builder);
            builder.finish(match_settings_flat, None);
            let data = builder.finished_data().to_owned();
            data
        };

        self.send_packet(PacketDataType::MatchSettings, &match_settings_flat)
            .map_err(|x| MatchManagerError::MatchStart(x.to_string()))
    }

    pub fn new(addr: &str) -> Result<RLBotConnection, MatchManagerError> {
        let stream = TcpStream::connect(addr)?;
        Ok(RLBotConnection { stream })
    }
}
