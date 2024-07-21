use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use flatbuffers::{FlatBufferBuilder, InvalidFlatbuffer};
use thiserror::Error;

pub mod agents;

#[cfg(feature = "glam")]
pub use glam;

pub(crate) mod flat_wrapper;

pub use crate::flat_wrapper::rlbot;

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
    None,
    GameTickPacket(GameTickPacket),
    FieldInfo(FieldInfo),
    StartCommand(StartCommand),
    MatchSettings(MatchSettings),
    PlayerInput(PlayerInput),
    DesiredGameState(DesiredGameState),
    RenderGroup(RenderGroup),
    RemoveRenderGroup(RemoveRenderGroup),
    MatchComm(MatchComm),
    BallPrediction(BallPrediction),
    ReadyMessage(ReadyMessage),
    StopCommand(StopCommand),
}

impl Packet {
    pub fn data_type(&self) -> u16 {
        match *self {
            Packet::None => 0,
            Packet::GameTickPacket(_) => 1,
            Packet::FieldInfo(_) => 2,
            Packet::StartCommand(_) => 3,
            Packet::MatchSettings(_) => 4,
            Packet::PlayerInput(_) => 5,
            Packet::DesiredGameState(_) => 6,
            Packet::RenderGroup(_) => 7,
            Packet::RemoveRenderGroup(_) => 8,
            Packet::MatchComm(_) => 9,
            Packet::BallPrediction(_) => 10,
            Packet::ReadyMessage(_) => 11,
            Packet::StopCommand(_) => 12,
        }
    }

    pub fn build(self, builder: &mut FlatBufferBuilder) -> Vec<u8> {
        // TODO: make this mess nicer
        macro_rules! p {
            ($x:ident) => {{
                builder.reset();
                let root = $x.pack(builder);
                builder.finish(root, None);
                builder.finished_data().to_owned()
            }};
        }

        match self {
            // 0u16 (be data_type), 1u16 (be data_length), 0u8 (empty payload)
            Packet::None => [0u8, 0u8, 0u8, 1u8, 0u8].to_vec(),
            Packet::GameTickPacket(x) => p!(x),
            Packet::FieldInfo(x) => p!(x),
            Packet::StartCommand(x) => p!(x),
            Packet::MatchSettings(x) => p!(x),
            Packet::PlayerInput(x) => p!(x),
            Packet::DesiredGameState(x) => p!(x),
            Packet::RenderGroup(x) => p!(x),
            Packet::RemoveRenderGroup(x) => p!(x),
            Packet::MatchComm(x) => p!(x),
            Packet::BallPrediction(x) => p!(x),
            Packet::ReadyMessage(x) => p!(x),
            Packet::StopCommand(x) => p!(x),
        }
    }

    pub fn from_payload(data_type: u16, payload: &[u8]) -> Result<Self, PacketParseError> {
        // TODO: make this mess nicer
        macro_rules! p {
            ($x:ty) => {{
                flatbuffers::root::<$x>(&payload)?.unpack()
            }};
        }

        use flat_wrapper::generated::rlbot::flat;

        match data_type {
            0 => Ok(Self::None),
            1 => Ok(Self::GameTickPacket(p!(flat::GameTickPacket))),
            2 => Ok(Self::FieldInfo(p!(flat::FieldInfo))),
            3 => Ok(Self::StartCommand(p!(flat::StartCommand))),
            4 => Ok(Self::MatchSettings(p!(flat::MatchSettings))),
            5 => Ok(Self::PlayerInput(p!(flat::PlayerInput))),
            6 => Ok(Self::DesiredGameState(p!(flat::DesiredGameState))),
            7 => Ok(Self::RenderGroup(p!(flat::RenderGroup))),
            8 => Ok(Self::RemoveRenderGroup(p!(flat::RemoveRenderGroup))),
            9 => Ok(Self::MatchComm(p!(flat::MatchComm))),
            10 => Ok(Self::BallPrediction(p!(flat::BallPrediction))),
            11 => Ok(Self::ReadyMessage(p!(flat::ReadyMessage))),
            12 => Ok(Self::StopCommand(p!(flat::StopCommand))),
            _ => Err(PacketParseError::InvalidDataType(data_type)),
        }
    }
}

pub struct RLBotConnection {
    stream: TcpStream,
    builder: FlatBufferBuilder<'static>,
    recv_buf: Vec<u8>,
}

impl RLBotConnection {
    pub fn send_packet(&mut self, packet: Packet) -> Result<(), RLBotError> {
        let data_type_bin = packet.data_type().to_be_bytes().to_vec();
        let payload = packet.build(&mut self.builder);
        let data_len_bin = (payload.len() as u16).to_be_bytes().to_vec();

        // Join so we make sure everything gets written in the right order
        let joined = [data_type_bin, data_len_bin, payload].concat();

        self.stream.write_all(&joined)?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn recv_packet(&mut self) -> Result<Packet, RLBotError> {
        let mut buf = [0u8, 0u8];

        self.stream.read_exact(&mut buf)?;
        let data_type = u16::from_be_bytes(buf);

        self.stream.read_exact(&mut buf)?;
        let data_len = u16::from_be_bytes(buf);

        self.recv_buf.resize(data_len as usize, 0);
        self.stream.read_exact(&mut self.recv_buf)?;

        let packet = Packet::from_payload(data_type, &self.recv_buf)?;

        Ok(packet)
    }

    pub fn new(addr: &str) -> Result<RLBotConnection, RLBotError> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;

        Ok(RLBotConnection {
            stream,
            builder: FlatBufferBuilder::with_capacity(1024),
            recv_buf: Vec::with_capacity(1024),
        })
    }
}
