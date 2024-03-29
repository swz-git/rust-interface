use std::env;

use rlbot_interface::{
    rlbot::{ControllerState, PlayerInput, ReadyMessage},
    Packet, RLBotConnection,
};

fn main() {
    println!("Connecting");

    let rlbot_addr = env::var("RLBOT_CORE_ADDR").unwrap_or("127.0.0.1:23234".to_owned());

    let mut rlbot_connection = RLBotConnection::new(&rlbot_addr).expect("connection");

    println!("Running!");

    let car_index = env::var("RLBOT_INDEX")
        .map(|x| x.parse().unwrap())
        .unwrap_or(0);

    rlbot_connection
        .send_packet(Packet::ReadyMessage(ReadyMessage {
            wants_ball_predictions: true,
            wants_quick_chat: false,
            wants_game_messages: true,
            ..ReadyMessage::default()
        }))
        .unwrap();

    loop {
        let Packet::GameTickPacket(game_tick_packet) = rlbot_connection.recv_packet().unwrap()
        else {
            continue;
        };
        let target = game_tick_packet.ball.physics;
        let car = game_tick_packet
            .players
            .get(car_index)
            .unwrap()
            .physics
            .clone();

        let bot_to_target_angle = (target.location.clone().y - car.location.clone().y)
            .atan2(target.location.x - car.location.x);

        let mut bot_front_to_target_angle = bot_to_target_angle - car.rotation.yaw;

        if bot_front_to_target_angle > 3.14 {
            bot_front_to_target_angle -= 2. * 3.14
        };
        if bot_front_to_target_angle < -3.14 {
            bot_front_to_target_angle += 2. * 3.14
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
                player_index: car_index as i32,
                controller_state: Box::new(controller),
            }))
            .unwrap();
    }
}
