use std::{env, f32::consts::PI};

use rlbot_interface::{
    rlbot::{ControllerState, PlayerInput, ReadyMessage},
    Packet, RLBotConnection,
};

fn main() {
    println!("Connecting");

    let rlbot_addr = env::var("RLBOT_CORE_ADDR").unwrap_or("127.0.0.1:23234".to_owned());

    let mut rlbot_connection = RLBotConnection::new(&rlbot_addr).expect("connection");

    println!("Running!");

    let car_index: u32 = env::var("RLBOT_INDEX")
        .map(|x| x.parse().unwrap())
        .unwrap_or(0);

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
        let Some(ball) = game_tick_packet.balls.get(0) else {
            continue;
        };
        let target = &ball.physics;
        let car = game_tick_packet
            .players
            .get(car_index as usize)
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
                player_index: car_index,
                controller_state: Box::new(controller),
            }))
            .unwrap();
    }
}
