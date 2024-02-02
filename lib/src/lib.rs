use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use flatbuffers::{FlatBufferBuilder, InvalidFlatbuffer};
use thiserror::Error;

pub mod bots;

pub(crate) mod flat_wrapper;

pub mod rlbot {
    pub use crate::flat_wrapper::*;
}

use rlbot::*;

#[derive(Error, Debug)]
pub enum PacketParseError {
    #[error("Invalid data type: {0}")]
    InvalidDataType(u16),
    #[error("Unpacking flatbuffer failed")]
    InvalidFlatbuffer(#[from] InvalidFlatbuffer),
}

#[derive(Error, Debug)]
pub enum RLBotError {
    #[error("Connection to RLBot failed")]
    Connection(#[from] io::Error),
    #[error("Parsing packet failed")]
    PacketParseError(#[from] PacketParseError),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Packet {
    GameTickPacket(GameTickPacket),
    FieldInfo(FieldInfo),
    MatchSettings(MatchSettings),
    PlayerInput(PlayerInput),
    DesiredGameState(DesiredGameState),
    RenderGroup(RenderGroup),
    QuickChat(QuickChat),
    BallPrediction(BallPrediction),
    ReadyMessage(ReadyMessage),
    MessagePacket(MessagePacket),
}

impl Packet {
    pub fn data_type(&self) -> u16 {
        match *self {
            Packet::GameTickPacket(_) => 1,
            Packet::FieldInfo(_) => 2,
            Packet::MatchSettings(_) => 3,
            Packet::PlayerInput(_) => 4,
            Packet::DesiredGameState(_) => 7,
            Packet::RenderGroup(_) => 8,
            Packet::QuickChat(_) => 9,
            Packet::BallPrediction(_) => 10,
            Packet::ReadyMessage(_) => 11,
            Packet::MessagePacket(_) => 12,
        }
    }

    pub fn build(self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();

        // TODO: make this mess nicer
        macro_rules! p {
            ($x:ident) => {{
                let root = $x.pack(&mut builder);
                builder.finish(root, None)
            }};
        }

        match self {
            Packet::GameTickPacket(x) => p!(x),
            Packet::FieldInfo(x) => p!(x),
            Packet::MatchSettings(x) => p!(x),
            Packet::PlayerInput(x) => p!(x),
            Packet::DesiredGameState(x) => p!(x),
            Packet::RenderGroup(x) => p!(x),
            Packet::QuickChat(x) => p!(x),
            Packet::BallPrediction(x) => p!(x),
            Packet::ReadyMessage(x) => p!(x),
            Packet::MessagePacket(x) => p!(x),
        }
        builder.finished_data().to_owned()
    }

    pub fn from_payload(data_type: u16, payload: Vec<u8>) -> Result<Self, PacketParseError> {
        // TODO: make this mess nicer
        macro_rules! p {
            ($x:ty) => {{
                flatbuffers::root::<$x>(&payload)?.unpack()
            }};
        }

        use flat_wrapper::rlbot_generated::rlbot::flat;

        match data_type {
            1 => Ok(Self::GameTickPacket(p!(flat::GameTickPacket))),
            2 => Ok(Self::FieldInfo(p!(flat::FieldInfo))),
            3 => Ok(Self::MatchSettings(p!(flat::MatchSettings))),
            4 => Ok(Self::PlayerInput(p!(flat::PlayerInput))),
            7 => Ok(Self::DesiredGameState(p!(flat::DesiredGameState))),
            8 => Ok(Self::RenderGroup(p!(flat::RenderGroup))),
            9 => Ok(Self::QuickChat(p!(flat::QuickChat))),
            10 => Ok(Self::BallPrediction(p!(flat::BallPrediction))),
            11 => Ok(Self::ReadyMessage(p!(flat::ReadyMessage))),
            12 => Ok(Self::MessagePacket(p!(flat::MessagePacket))),
            _ => Err(PacketParseError::InvalidDataType(data_type)),
        }
    }
}

pub struct RLBotConnection {
    stream: TcpStream,
}

impl RLBotConnection {
    pub fn send_packet(&mut self, packet: Packet) -> Result<(), RLBotError> {
        let data_type_bin = packet.data_type().to_be_bytes().to_vec();
        let payload = packet.build();
        let data_len_bin = (payload.len() as u16).to_be_bytes().to_vec();

        // Join so we make sure everything gets written in the right order
        let joined = [data_type_bin, data_len_bin, payload].concat();

        self.stream.write_all(&joined)?;
        Ok(())
    }

    pub fn recv_packet(&mut self) -> Result<Packet, RLBotError> {
        let mut buf = [0u8, 0u8];

        self.stream.read_exact(&mut buf)?;
        let data_type = u16::from_be_bytes(buf);

        self.stream.read_exact(&mut buf)?;
        let data_len = u16::from_be_bytes(buf);

        let mut buf = vec![0u8; data_len as usize];
        self.stream.read_exact(&mut buf)?;
        let data_payload = buf;

        let packet = Packet::from_payload(data_type, data_payload)?;

        Ok(packet)
    }

    pub fn new(addr: &str) -> Result<RLBotConnection, RLBotError> {
        let stream = TcpStream::connect(addr)?;
        Ok(RLBotConnection { stream })
    }
}
