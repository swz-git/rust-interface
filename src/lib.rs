use std::{
    error::Error,
    io::{self, Write},
    net::TcpStream,
};

use flatbuffers::FlatBufferBuilder;
use thiserror::Error;

#[allow(non_snake_case)]
pub mod rlbot_generated;
use rlbot_generated::rlbot::flat;

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
}

pub fn connect(addr: &str) -> Result<RLBotConnection, MatchManagerError> {
    let stream = TcpStream::connect(addr)?;
    Ok(RLBotConnection { stream })
}

pub fn start_match(conn: &mut RLBotConnection) -> Result<(), MatchManagerError> {
    let match_settings_flat = {
        let mut builder = FlatBufferBuilder::new();
        let test_str = builder.create_string("Test");
        let player_configurations_vec = vec![
            flat::PlayerConfiguration::create(
                &mut builder,
                &flat::PlayerConfigurationArgs {
                    variety: None,
                    variety_type: flat::PlayerClass::RLBotPlayer,
                    name: Some(test_str),
                    team: 1,
                    loadout: None,
                    spawnId: 0,
                },
            ),
            flat::PlayerConfiguration::create(
                &mut builder,
                &flat::PlayerConfigurationArgs {
                    variety: None,
                    variety_type: flat::PlayerClass::RLBotPlayer,
                    name: Some(test_str),
                    team: 0,
                    loadout: None,
                    spawnId: 0,
                },
            ),
        ];
        let player_configurations_builder_vec = builder.create_vector(&player_configurations_vec);
        let match_settings = flat::MatchSettings::create(
            &mut builder,
            &flat::MatchSettingsArgs {
                playerConfigurations: Some(player_configurations_builder_vec),
                gameMode: flat::GameMode::Soccer,
                gameMap: flat::GameMap::ThrowbackStadium,
                skipReplays: false,
                instantStart: true,
                mutatorSettings: None,
                existingMatchBehavior: flat::ExistingMatchBehavior::Restart,
                enableLockstep: false,
                enableRendering: false,
                enableStateSetting: false,
                autoSaveReplay: false,
                gameMapUpk: None,
            },
        );
        builder.finish(match_settings, None);
        let data = builder.finished_data().to_owned();
        data
    };

    conn.send_packet(PacketDataType::MatchSettings, &match_settings_flat)
        .map_err(|x| MatchManagerError::MatchStart(x.to_string()))
}
