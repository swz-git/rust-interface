use std::{
    collections::VecDeque,
    io::Write,
    mem,
    sync::Arc,
    thread::{self},
};

use crate::{flat::*, Packet, RLBotConnection, RLBotError};

#[allow(unused_variables)]
pub trait Agent {
    fn new(controllable_info: ControllableInfo) -> Self;
    fn tick(&mut self, game_packet: &GamePacket, packet_queue: &mut PacketQueue) -> ();
    fn on_field_info(&mut self, field_info: &FieldInfo, packet_queue: &mut PacketQueue) -> () {}
    fn on_match_settings(
        &mut self,
        match_settings: &MatchConfiguration,
        packet_queue: &mut PacketQueue,
    ) -> () {
    }
    fn on_match_comm(&mut self, match_comm: &MatchComm, packet_queue: &mut PacketQueue) -> () {}
    fn on_ball_prediction(
        &mut self,
        ball_prediction: &BallPrediction,
        packet_queue: &mut PacketQueue,
    ) -> () {
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AgentError {
    #[error("Agent panicked")]
    AgentPanic,
    #[error("RLBot failed")]
    PacketParseError(#[from] crate::RLBotError),
}

/// A queue of packets to be sent to RLBotServer
pub struct PacketQueue {
    internal_queue: Vec<Packet>,
}

impl PacketQueue {
    pub fn new() -> Self {
        PacketQueue {
            internal_queue: Vec::with_capacity(16),
        }
    }
    pub fn push(&mut self, packet: impl Into<Packet>) {
        self.internal_queue.push(packet.into());
    }
    fn empty(&mut self) -> Vec<Packet> {
        mem::take(&mut self.internal_queue)
    }
}

/// Run multiple agents on one thread each. They share a connection.
/// Ok(()) means a successful exit; one of the bots received a None packet.
pub fn run_agents<T: Agent>(
    connection_settings: ConnectionSettings,
    mut connection: RLBotConnection,
) -> Result<(), AgentError> {
    connection.send_packet(connection_settings)?;

    let mut packets_to_process = VecDeque::new();

    // Wait for ControllableTeamInfo to know which indices we control
    let controllable_team_info = loop {
        let packet = connection.recv_packet()?;
        if let Packet::ControllableTeamInfo(x) = packet {
            break x;
        } else {
            packets_to_process.push_back(packet);
            continue;
        }
    };

    let mut threads = vec![];

    let (outgoing_sender, outgoing_recver) =
        kanal::bounded::<Vec<Packet>>(controllable_team_info.controllables.len());
    for (i, controllable_info) in controllable_team_info.controllables.iter().enumerate() {
        let (incoming_sender, incoming_recver) = kanal::bounded::<Arc<Packet>>(1);
        let controllable_info = controllable_info.clone();

        let outgoing_sender = outgoing_sender.clone();

        threads.push((
            incoming_sender,
            thread::Builder::new()
                .name(format!(
                    "Agent thread {i} (spawn_id: {} index: {})",
                    controllable_info.spawn_id, controllable_info.index
                ))
                .spawn(move || {
                    let mut bot = T::new(controllable_info);
                    let mut outgoing_queue_local = PacketQueue::new();

                    loop {
                        let Ok(packet) = incoming_recver.recv() else {
                            panic!("channel recv failed")
                        };

                        match &*packet {
                            Packet::None => break,
                            Packet::GamePacket(x) => bot.tick(x, &mut outgoing_queue_local),
                            Packet::FieldInfo(x) => bot.on_field_info(x, &mut outgoing_queue_local),
                            Packet::MatchConfiguration(x) => {
                                bot.on_match_settings(x, &mut outgoing_queue_local)
                            }
                            Packet::MatchComm(x) => bot.on_match_comm(x, &mut outgoing_queue_local),
                            Packet::BallPrediction(x) => {
                                bot.on_ball_prediction(x, &mut outgoing_queue_local)
                            }
                            _ => unreachable!() /* The rest of the packets are only client -> server */
                        }

                        outgoing_sender.send(outgoing_queue_local.empty()).expect("Couldn't send outgoing");
                    }
                    drop(incoming_recver);
                    drop(outgoing_sender);
                })
                .unwrap(),
        ));
    }
    // drop never-again-used copy of outgoing_sender
    // NO NOT REMOVE, otherwise outgoing_recver.recv() will never error
    // which we rely on for clean exiting
    drop(outgoing_sender);

    // We only need to send one init complete with the first
    // spawn id even though we may be running multiple bots.
    if controllable_team_info.controllables.is_empty() {
        // run no bots? no problem, done
        return Ok(());
    };

    connection.send_packet(Packet::InitComplete)?;

    // Main loop, broadcast packet to all of the bots, then wait for all of the outgoing vecs
    // Rust limited to 32 for now, hopefully fixed in the future though not really a big deal
    let mut to_send: [Vec<Packet>; 32] = Default::default();
    'main_loop: loop {
        let mut maybe_packet = packets_to_process.pop_front();
        if maybe_packet.is_none() && connection.stream.peek(&mut 0u16.to_be_bytes()).is_ok() {
            maybe_packet = Some(connection.recv_packet()?);
        };

        if let Some(packet) = maybe_packet {
            let arc = Arc::new(packet);
            for (incoming_sender, _) in threads.iter() {
                let Ok(_) = incoming_sender.send(arc.clone()) else {
                    return Err(AgentError::AgentPanic);
                };
            }
        }

        for reserved_packet_spot in to_send.iter_mut().take(threads.len()) {
            if let Ok(messages) = outgoing_recver.recv() {
                *reserved_packet_spot = messages;
            } else {
                break 'main_loop;
            }
        }

        write_multiple_packets(
            &mut connection,
            mem::take(&mut to_send).into_iter().flatten(),
        )?;
    }

    Ok(())
}

fn write_multiple_packets(
    connection: &mut RLBotConnection,
    packets: impl Iterator<Item = Packet>,
) -> Result<(), RLBotError> {
    let to_write = packets
        // convert Packet to Vec<u8> that RLBotServer can understand
        .flat_map(|x| {
            let data_type_bin = x.data_type().to_be_bytes().to_vec();
            let payload = x.build(&mut connection.builder);
            let data_len_bin = (payload.len() as u16).to_be_bytes().to_vec();

            [data_type_bin, data_len_bin, payload].concat()
        })
        .collect::<Vec<_>>();

    connection.stream.write_all(&to_write)?;
    connection.stream.flush()?;

    Ok(())
}
