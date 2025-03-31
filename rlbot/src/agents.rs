use std::{mem, sync::Arc, thread};

use monoio::io::{AsyncReadRent, AsyncReadRentExt as _, AsyncWriteRentExt as _};

use crate::{
    Packet, RLBotConnection, RLBotError, StartingInfo,
    flat::*,
    util::{PacketQueue, build_multiple_packets, write_multiple_packets},
};

#[allow(unused_variables)]
pub trait Agent {
    // TODO: Maybe pass a struct?
    fn new(
        team: u32,
        controllable_info: ControllableInfo,
        match_configuration: Arc<MatchConfiguration>,
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
    // TODO: Maybe pass a struct?
    agent_id: String,
    wants_ball_predictions: bool,
    wants_comms: bool,
    mut connection: RLBotConnection,
) -> Result<(), AgentError> {
    connection.send_packet(ConnectionSettings {
        agent_id: agent_id.clone(),
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

    let match_configuration = Arc::new(match_configuration);
    let field_info = Arc::new(field_info);

    let num_threads = controllable_team_info.controllables.len();
    let mut threads = Vec::with_capacity(num_threads);

    let (outgoing_sender, outgoing_recver) = kanal::unbounded::<Vec<Packet>>();
    for (i, controllable_info) in controllable_team_info.controllables.into_iter().enumerate() {
        let (incoming_sender, incoming_recver) = kanal::unbounded::<Arc<Packet>>();
        let match_configuration = match_configuration.clone();
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
                        match_configuration,
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

    // Write initial packets like SetLoadout and also append a InitComplete to signal that we are ready to play
    write_multiple_packets(
        &mut connection,
        to_send
            .iter_mut()
            .flat_map(mem::take)
            .chain([Packet::InitComplete]),
    )?;

    #[cfg(target_os = "linux")]
    type MonoIoDriver = monoio::IoUringDriver;
    #[cfg(not(target_os = "linux"))]
    type MonoIoDriver = monoio::LegacyDriver;

    let incoming_senders_async = threads
        .iter()
        .map(|(sync_sender, _)| sync_sender.clone_async())
        .collect::<Vec<_>>();

    async fn handle_packet(
        stream: &mut monoio::net::TcpStream,
        incoming_senders: &[kanal::AsyncSender<Arc<Packet>>],
        packet_type: u16,
        packet_len: u16,
    ) -> Result<(), AgentError> {
        let (packet_read_result, packet_raw) =
            stream.read_exact(vec![0u8; packet_len as usize]).await;

        packet_read_result.map_err(RLBotError::from)?;

        let packet =
            Arc::new(Packet::from_payload(packet_type, &packet_raw).map_err(RLBotError::from)?);

        for sender in incoming_senders {
            sender
                .send(packet.clone())
                .await
                .expect("incoming_sender.send failed");
        }

        Ok(())
    }

    let outgoing_recver = outgoing_recver.clone();

    monoio::start::<MonoIoDriver, _>(async move {
        let mut stream = monoio::net::TcpStream::from_std(connection.stream)
            .expect("couldn't convert std stream to monoio stream");
        let mut builder = rlbot_flat::planus::Builder::with_capacity(u16::MAX as usize);
        let incoming_senders = incoming_senders_async;
        let outgoing_recver = outgoing_recver.to_async();

        loop {
            monoio::select! {
                u32_r = stream.read_u32() => {
                    let bytes = u32_r.map_err(RLBotError::from)?.to_be_bytes();
                    let packet_type = u16::from_be_bytes([bytes[0], bytes[1]]);
                    let packet_len = u16::from_be_bytes([bytes[2], bytes[3]]);
                    handle_packet(&mut stream, &incoming_senders, packet_type, packet_len).await?;
                }
                packets = outgoing_recver.recv() => {
                    stream.write_all(build_multiple_packets(&mut builder, packets.expect("outgoing_recver.recv failed").into_iter())).await.0.map_err(RLBotError::from)?;
                }
            }
        }
        Ok::<(), AgentError>(())
    })?;

    for (_, handle) in threads {
        handle.join().unwrap();
    }

    Ok(())
}

fn run_agent<T: Agent>(
    incoming_recver: kanal::Receiver<Arc<Packet>>,
    team: u32,
    controllable_info: ControllableInfo,
    match_configuration: Arc<MatchConfiguration>,
    field_info: Arc<FieldInfo>,
    outgoing_sender: kanal::Sender<Vec<Packet>>,
) {
    let mut outgoing_queue_local = PacketQueue::default();
    let mut bot = T::new(
        team,
        controllable_info,
        match_configuration,
        field_info,
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

        outgoing_sender
            .send(outgoing_queue_local.empty())
            .expect("Couldn't send outgoing");
    }

    drop(incoming_recver);
    drop(outgoing_sender);
}
