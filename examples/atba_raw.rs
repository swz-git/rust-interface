use std::{env, f32::consts::PI};

use rlbot_interface::{
    rlbot::{ControllerState, PlayerInput, ReadyMessage},
    Packet, RLBotConnection,
};

fn main() {
    let spawn_ids = env::var("RLBOT_SPAWN_IDS")
        .map(|x| {
            x.split(',')
                .map(|x| x.parse::<i32>().expect("int in RLBOT_SPAWN_IDS"))
                .collect::<Vec<_>>()
        })
        .unwrap_or(vec![]);

    if spawn_ids.len() != 1 {
        panic!("The raw atba example code does not support hiveminds, please only pass one spawn_id or disable the hivemind field in bot.toml")
    }

    let spawn_id = spawn_ids[0];

    println!("Connecting");

    let rlbot_addr = env::var("RLBOT_CORE_ADDR").unwrap_or("127.0.0.1:23234".to_owned());

    let mut rlbot_connection = RLBotConnection::new(&rlbot_addr).expect("connection");

    println!("Running!");

    rlbot_connection
        .send_packet(Packet::ReadyMessage(ReadyMessage {
            wants_ball_predictions: true,
            wants_comms: true,
            close_after_match: true,
        }))
        .unwrap();

    loop {
        let Packet::GameTickPacket(game_tick_packet) = rlbot_connection.recv_packet().unwrap()
        else {
            continue;
        };

        let Some(bot_index) = game_tick_packet
            .players
            .iter()
            .position(|x| x.spawn_id == spawn_id)
        else {
            // If we aren't in the game, don't do anything
            continue;
        };

        let Some(ball) = game_tick_packet.balls.get(0) else {
            continue;
        };
        let target = &ball.physics;
        let car = game_tick_packet
            .players
            .get(bot_index as usize)
            .unwrap()
            .physics
            .clone();

        let bot_to_target_angle = f32::atan2(
            target.location.clone().y - car.location.clone().y,
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
            .send_packet(Packet::PlayerInput(PlayerInput {
                player_index: bot_index as u32,
                controller_state: Box::new(controller),
            }))
            .unwrap();
    }
}
