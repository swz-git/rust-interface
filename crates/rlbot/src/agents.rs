use std::{
    io::Write,
    mem,
    sync::Arc,
    thread::{self},
    vec,
};

use crate::{Packet, RLBotConnection, RLBotError, StartingInfo, flat::*};

#[allow(unused_variables)]
pub trait Agent {
    fn new(
        team: u32,
        controllable_info: ControllableInfo,
        match_config: Arc<MatchConfiguration>,
        field_info: Arc<FieldInfo>,
        packet_queue: &mut PacketQueue,
    ) -> Self;
    fn tick(&mut self, game_packet: &GamePacket, packet_queue: &mut PacketQueue);
    fn on_match_comm(&mut self, match_comm: &MatchComm, packet_queue: &mut PacketQueue) {}
    fn on_ball_prediction(&mut self, ball_prediction: &BallPrediction) {}
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

/// Run multiple agents with n agents per thread. They share a connection.
/// Ok(()) means a successful exit; one of the bots received a None packet.
///
/// # Errors
///
/// Returns an error if an agent panics or if there is an error with the connection.
///
/// # Panics
///
/// Panics if a thread can't be spawned for each agent.
pub fn run_agents<T: Agent>(
    agent_id: String,
    wants_ball_predictions: bool,
    wants_comms: bool,
    mut connection: RLBotConnection,
) -> Result<(), AgentError> {
    connection.send_packet(ConnectionSettings {
        agent_id,
        wants_ball_predictions,
        wants_comms,
        close_between_matches: true,
    })?;

    let StartingInfo {
        controllable_team_info,
        match_configuration,
        field_info,
    } = connection.get_starting_info()?;

    if controllable_team_info.controllables.is_empty() {
        // run no bots? no problem, done
        return Ok(());
    }

    let match_config = Arc::new(match_configuration);
    let field_info = Arc::new(field_info);

    let num_threads = controllable_team_info.controllables.len();
    let mut threads = Vec::with_capacity(num_threads);

    let (outgoing_sender, outgoing_recver) = kanal::unbounded::<Vec<Packet>>();
    for (i, controllable_info) in controllable_team_info.controllables.into_iter().enumerate() {
        let (incoming_sender, incoming_recver) = kanal::unbounded::<Arc<Packet>>();
        let match_config = match_config.clone();
        let field_info = field_info.clone();

        let outgoing_sender = outgoing_sender.clone();

        threads.push((
            incoming_sender,
            thread::Builder::new()
                .name(format!(
                    "Agent thread {i} (index {})",
                    controllable_info.index,
                ))
                .spawn(move || {
                    run_agent::<T>(
                        incoming_recver,
                        controllable_team_info.team,
                        controllable_info,
                        match_config,
                        field_info,
                        outgoing_sender,
                    );
                })
                .unwrap(),
        ));
    }
    // drop never-again-used copy of outgoing_sender
    // DO NOT REMOVE, otherwise outgoing_recver.recv() will never error
    // which we rely on for clean exiting
    drop(outgoing_sender);

    let mut to_send: Vec<Vec<Packet>> = vec![Vec::new(); num_threads];
    for reserved_packet_spot in &mut to_send {
        if let Ok(messages) = outgoing_recver.recv() {
            *reserved_packet_spot = messages;
        } else {
            return Err(AgentError::AgentPanic);
        }
    }
    write_multiple_packets(&mut connection, to_send.iter_mut().flat_map(mem::take))?;

    // We only need to send one init complete with the first
    // spawn id even though we may be running multiple bots.
    connection.send_packet(Packet::InitComplete)?;

    // Main loop, broadcast packet to all of the bots, then wait for all of the outgoing vecs
    let mut ball_prediction = None;
    let mut game_packet = None;
    'main_loop: loop {
        connection.set_nonblocking(true)?;
        while let Ok(Some(packet)) = connection.try_recv_packet() {
            let packet = Arc::new(packet);

            match &*packet {
                Packet::None => {
                    for (incoming_sender, _) in &threads {
                        if incoming_sender.send(packet.clone()).is_err() {
                            return Err(AgentError::AgentPanic);
                        }
                    }

                    break 'main_loop;
                }
                Packet::MatchComm(_) => {
                    for (incoming_sender, _) in &threads {
                        if incoming_sender.send(packet.clone()).is_err() {
                            return Err(AgentError::AgentPanic);
                        }
                    }
                }
                Packet::BallPrediction(_) => ball_prediction = Some(packet),
                Packet::GamePacket(_) => game_packet = Some(packet),
                _ => panic!("Unexpected packet: {:?}", packet),
            }
        }
        connection.set_nonblocking(false)?;

        if let Some(game_packet) = game_packet.take() {
            if let Some(ball_prediction) = ball_prediction.take() {
                for (incoming_sender, _) in &threads {
                    if incoming_sender.send(ball_prediction.clone()).is_err() {
                        return Err(AgentError::AgentPanic);
                    }
                }
            }

            for (incoming_sender, _) in &threads {
                if incoming_sender.send(game_packet.clone()).is_err() {
                    return Err(AgentError::AgentPanic);
                }
            }

            ball_prediction = None;

            for reserved_packet_spot in &mut to_send {
                if let Ok(messages) = outgoing_recver.recv() {
                    *reserved_packet_spot = messages;
                } else {
                    break 'main_loop;
                }
            }

            write_multiple_packets(&mut connection, to_send.iter_mut().flat_map(mem::take))?;
        }
    }

    for (_, handle) in threads {
        handle.join().unwrap();
    }

    Ok(())
}

fn run_agent<T: Agent>(
    incoming_recver: kanal::Receiver<Arc<Packet>>,
    team: u32,
    controllable_info: ControllableInfo,
    match_config: Arc<MatchConfiguration>,
    field_info: Arc<FieldInfo>,
    outgoing_sender: kanal::Sender<Vec<Packet>>,
) {
    let mut outgoing_queue_local = PacketQueue::default();
    let mut bot = T::new(
        team,
        controllable_info,
        match_config.clone(),
        field_info.clone(),
        &mut outgoing_queue_local,
    );

    outgoing_sender
        .send(outgoing_queue_local.empty())
        .expect("Couldn't send outgoing");

    loop {
        let Ok(packet) = incoming_recver.recv() else {
            panic!("channel recv failed")
        };

        match &*packet {
            Packet::None => break,
            Packet::GamePacket(x) => bot.tick(x, &mut outgoing_queue_local),
            Packet::MatchComm(x) => {
                bot.on_match_comm(x, &mut outgoing_queue_local);
            }
            Packet::BallPrediction(x) => {
                bot.on_ball_prediction(x);
            }
            _ => unreachable!(), /* The rest of the packets are only client -> server */
        }

        if matches!(*packet, Packet::GamePacket(_)) {
            outgoing_sender
                .send(outgoing_queue_local.empty())
                .expect("Couldn't send outgoing");
        }
    }

    drop(incoming_recver);
    drop(outgoing_sender);
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
