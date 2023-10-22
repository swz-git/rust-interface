use std::{io, net::TcpStream};

use flatbuffers::FlatBufferBuilder;
use thiserror::Error;

#[allow(non_snake_case)]
mod rlbot_generated;
use rlbot_generated::rlbot::flat::{self, *};

#[derive(Error, Debug)]
pub enum MatchManagerError {
    #[error("Connection to RLBot failed")]
    Connection(#[from] io::Error),
    #[error("Failed to start match")]
    MatchStart(String),
}

pub struct RLBotConnection {
    stream: TcpStream,
}

pub fn connect(addr: &str) -> Result<RLBotConnection, MatchManagerError> {
    let mut stream = TcpStream::connect(addr)?;
    Ok(RLBotConnection { stream })
}

pub fn start_match(conn: RLBotConnection) -> Result<(), MatchManagerError> {
    let match_settings_flat = {
        let mut builder = FlatBufferBuilder::new();
        flat::MatchSettings::create(
            &mut builder,
            &MatchSettingsArgs {
                playerConfigurations: None,
                gameMode: todo!(),
                gameMap: todo!(),
                skipReplays: false,
                instantStart: true,
                mutatorSettings: None,
                existingMatchBehavior: todo!(),
                enableLockstep: false,
                enableRendering: false,
                enableStateSetting: false,
                autoSaveReplay: false,
                gameMapUpk: None,
            },
        )
    };

    todo!()
}
