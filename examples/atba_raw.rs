use std::{env, f32::consts::PI};

use rlbot_interface::{
    rlbot::{ConnectionSettings, ControllerState, PlayerInput},
    Packet, RLBotConnection,
};

fn main() {
    let agent_id = env::var("RLBOT_AGENT_ID").unwrap_or("rlbot/rust-example-bot".into());

    println!("Connecting");

    let rlbot_addr = env::var("RLBOT_CORE_ADDR").unwrap_or("127.0.0.1:23234".to_owned());

    let mut rlbot_connection = RLBotConnection::new(&rlbot_addr).expect("connection");

    println!("Running!");

    rlbot_connection
        .send_packet(ConnectionSettings {
            wants_ball_predictions: true,
            wants_comms: true,
            close_after_match: true,
            agent_id,
        })
        .unwrap();

    let mut packets_to_process = vec![];

    // Wait for Controllable(Team)Info to know which indices we control
    let controllable_team_info = loop {
        let packet = rlbot_connection.recv_packet().unwrap();
        if let Packet::ControllableTeamInfo(x) = packet {
            break x;
        } else {
            packets_to_process.push(packet);
            continue;
        }
    };

    if controllable_team_info.controllables.len() != 1 {
        panic!("The raw atba example code does not support hiveminds, please disable the hivemind field in bot.toml")
    }

    let controllable_info = controllable_team_info
        .controllables
        .first()
        .expect("controllables.len() = 1");

    rlbot_connection.send_packet(Packet::InitComplete).unwrap();

    loop {
        let Packet::GamePacket(game_tick_packet) = packets_to_process
            .pop()
            .unwrap_or(rlbot_connection.recv_packet().unwrap())
        else {
            continue;
        };

        let Some(ball) = game_tick_packet.balls.first() else {
            continue;
        };
        let target = &ball.physics;
        let car = game_tick_packet
            .players
            .get(controllable_info.index as usize)
            .unwrap()
            .physics
            .clone();

        let bot_to_target_angle = f32::atan2(
            target.location.y - car.location.y,
            target.location.x - car.location.x,
        );

        let mut bot_front_to_target_angle = bot_to_target_angle - car.rotation.yaw;

        if bot_front_to_target_angle > PI {
            bot_front_to_target_angle -= 2. * PI
        };
        if bot_front_to_target_angle < -PI {
            bot_front_to_target_angle += 2. * PI
        };

        let mut controller = ControllerState::default();

        if bot_front_to_target_angle > 0. {
            controller.steer = 1.;
        } else {
            controller.steer = -1.;
        }

        controller.throttle = 1.;

        rlbot_connection
            .send_packet(PlayerInput {
                player_index: controllable_info.index,
                controller_state: controller,
            })
            .unwrap();
    }
}
