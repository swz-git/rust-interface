use std::{collections::VecDeque, io::Write, iter, mem, sync::Arc, thread, time::Instant};

use futures::FutureExt;
use tokio::{io::AsyncWriteExt, task::block_in_place};

use crate::{rlbot::*, Packet, RLBotConnection, RLBotConnectionTokio, RLBotError};

#[allow(unused_variables)]
pub trait Agent {
    fn new(controllable_info: ControllableInfo) -> Self;
    fn tick(&mut self, game_tick_packet: GamePacket) -> Vec<Packet>;
    fn on_field_info(&mut self, field_info: FieldInfo) -> Vec<Packet> {
        vec![]
    }
    fn on_match_settings(&mut self, match_settings: MatchSettings) -> Vec<Packet> {
        vec![]
    }
    fn on_match_comm(&mut self, match_comm: MatchComm) -> Vec<Packet> {
        vec![]
    }
    fn on_ball_prediction(&mut self, ball_prediction: BallPrediction) -> Vec<Packet> {
        vec![]
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AgentError {
    #[error("Agent panicked")]
    AgentPanic,
    #[error("RLBot failed")]
    PacketParseError(#[from] crate::RLBotError),
}

/// Run multiple agents on one thread each. They share a connection.
/// Ok(()) means a successful exit; one of the bots received a None packet.
pub fn run_agents<T: Agent>(
    connection_settings: ConnectionSettings,
    mut connection: RLBotConnection,
) -> Result<(), AgentError> {
    connection.send_packet(connection_settings)?;

    let mut packets_to_process = VecDeque::new();

    // Wait for Controllable(Team)Info to know which indices we control
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

    let (thread_send, main_recv) = kanal::bounded(0);
    for (i, controllable_info) in controllable_team_info.controllables.iter().enumerate() {
        let (main_send, thread_recv) = kanal::bounded::<Packet>(0);
        let thread_send = thread_send.clone();
        let controllable_info = controllable_info.clone();

        threads.push((
            main_send,
            thread::Builder::new()
                .name(format!(
                    "Agent thread {i} (spawn_id: {} index: {})",
                    controllable_info.spawn_id, controllable_info.index
                ))
                .spawn(move || {
                    let mut bot = T::new(controllable_info);

                    while let Ok(packet) = thread_recv.recv() {
                        match packet {
                            Packet::None => break,
                            Packet::GamePacket(x) => {
                                thread_send.send(bot.tick(x)).unwrap();
                            }
                            Packet::FieldInfo(x) => thread_send.send(bot.on_field_info(x)).unwrap(),
                            Packet::MatchSettings(x) => {
                                thread_send.send(bot.on_match_settings(x)).unwrap()
                            }
                            Packet::MatchComm(x) => thread_send.send(bot.on_match_comm(x)).unwrap(),
                            Packet::BallPrediction(x) => {
                                thread_send.send(bot.on_ball_prediction(x)).unwrap()
                            }
                            _ => { /* The rest of the packets are only client -> server */ }
                        }
                    }
                    drop(thread_send);
                    drop(thread_recv);
                })
                .unwrap(),
        ));
    }
    // drop never-again-used copy of thread_send
    // NO NOT REMOVE, otherwise main_recv.recv() will never error
    // which we rely on for clean exiting
    drop(thread_send);

    // We only need to send one init complete with the first
    // spawn id even though we may be running multiple bots.
    if controllable_team_info.controllables.is_empty() {
        // run no bots? no problem, done
        return Ok(());
    };

    connection.send_packet(Packet::InitComplete)?;

    // Main loop, broadcast packet to all of the bots, then wait for all of the responses
    let mut to_send: Vec<Packet> = Vec::with_capacity(controllable_team_info.controllables.len());
    'main_loop: loop {
        let packet = packets_to_process
            .pop_front()
            .unwrap_or(connection.recv_packet()?);

        for (sender, _) in threads.iter() {
            let Ok(_) = sender.send(packet.clone()) else {
                return Err(AgentError::AgentPanic);
            };
        }

        for (_sender, _) in threads.iter() {
            let Ok(list) = main_recv.recv() else {
                break 'main_loop;
            };
            to_send.extend(list.into_iter())
        }

        if to_send.is_empty() {
            continue; // no need to send nothing
        }

        let raw_packets = build_multiple_packets(&mut connection.builder, mem::take(&mut to_send));
        connection.stream.write_all(&raw_packets).map_err(|e| {
            let new_e: RLBotError = e.into();
            new_e
        })?;
        connection.stream.flush().map_err(|e| {
            let new_e: RLBotError = e.into();
            new_e
        })?;
    }

    for (_, thread_handle) in threads.into_iter() {
        thread_handle.join().unwrap()
    }

    Ok(())
}

/// Run multiple agents using tokio (async). They share a connection.
/// Ok(()) means a successful exit; one of the bots received a None packet.
pub async fn run_agents_tokio<T: Agent + Send + Sync + 'static>(
    connection_settings: ConnectionSettings,
    mut connection: RLBotConnectionTokio,
) -> Result<(), AgentError> {
    connection.send_packet(connection_settings).await?;

    let mut packets_to_process = VecDeque::new();

    // Wait for Controllable(Team)Info to know which indices we control
    let controllable_team_info = loop {
        let packet = connection.recv_packet().await?;
        if let Packet::ControllableTeamInfo(x) = packet {
            break x;
        } else {
            packets_to_process.push_back(packet);
            continue;
        }
    };

    let mut threads = vec![];

    let (thread_send, main_recv) = kanal::bounded_async(controllable_team_info.controllables.len());
    for (i, controllable_info) in controllable_team_info.controllables.iter().enumerate() {
        let (main_send, thread_recv) = kanal::bounded_async::<Packet>(0);
        let thread_send = thread_send.clone();
        let controllable_info = controllable_info.clone();

        threads.push((
            main_send,
            tokio::spawn(async move {
                let mut bot = T::new(controllable_info);

                while let Ok(packet) = thread_recv.recv().await {
                    match packet {
                        Packet::None => break,
                        Packet::GamePacket(x) => {
                            thread_send.send(bot.tick(x)).await.expect("thread send");
                        }
                        Packet::FieldInfo(x) => {
                            thread_send
                                .send(bot.on_field_info(x))
                                .await
                                .expect("thread send");
                        }
                        Packet::MatchSettings(x) => {
                            thread_send
                                .send(bot.on_match_settings(x))
                                .await
                                .expect("thread send");
                        }
                        Packet::MatchComm(x) => {
                            thread_send
                                .send(bot.on_match_comm(x))
                                .await
                                .expect("thread send");
                        }
                        Packet::BallPrediction(x) => {
                            thread_send
                                .send(bot.on_ball_prediction(x))
                                .await
                                .expect("thread send");
                        }
                        _ => { /* The rest of the packets are only client -> server */ }
                    }
                }
                drop(thread_send);
                drop(thread_recv);
            }),
        ));
    }
    // drop never-again-used copy of thread_send
    // NO NOT REMOVE, otherwise main_recv.recv() will never error
    // which we rely on for clean exiting
    drop(thread_send);

    // We only need to send one init complete with the first
    // spawn id even though we may be running multiple bots.
    if controllable_team_info.controllables.is_empty() {
        // run no bots? no problem, done
        return Ok(());
    };

    connection.send_packet(Packet::InitComplete).await?;

    'main_loop: loop {
        let inst = Instant::now();
        let packet = packets_to_process
            .pop_front()
            .unwrap_or(connection.recv_packet().await?);
        // let packet = Arc::new(packet);

        // dbg!(Instant::now().duration_since(inst));
        let inst = Instant::now();

        for (sender, _thread) in &threads {
            if let Err(x) = sender.send(packet.clone()).await {
                return Err(AgentError::AgentPanic);
            }
        }

        // dbg!(Instant::now().duration_since(inst));
        let inst = Instant::now();

        for _i in 0..threads.len() {
            let r = main_recv.recv().await.unwrap();
            if r.len() == 0 {
                continue;
            }
            connection
                .stream
                .write_all(&build_multiple_packets(&mut connection.builder, r))
                .await
                .map_err(|e| {
                    let new_e: RLBotError = e.into();
                    new_e
                })?;
        }
        connection.stream.flush().await.unwrap();

        // dbg!(Instant::now().duration_since(inst));
    }

    // Ok(())
}

fn build_multiple_packets(builder: &mut planus::Builder, packets: Vec<Packet>) -> Vec<u8> {
    packets
        .into_iter()
        // convert Packet to Vec<u8> that rlbot can understand
        .map(|x| {
            let data_type_bin = x.data_type().to_be_bytes().to_vec();
            let payload = x.build(builder);
            let data_len_bin = (payload.len() as u16).to_be_bytes().to_vec();

            [data_type_bin, data_len_bin, payload].concat()
        })
        .collect::<Vec<_>>()
        // Join all raw packets together
        .concat()
}
