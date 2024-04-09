use std::env;

use rlbot_lib::{
    rlbot::{ControllerState, PlayerInput, ReadyMessage},
    Packet, RLBotConnection,
};

const DEFAULT_CAR_ID: usize = 0;

fn main() {
    println!("Connecting");

    let mut rlbot_connection = RLBotConnection::new("127.0.0.1:23234").expect("connection");

    println!("Running!");

    rlbot_connection
        .send_packet(Packet::ReadyMessage(ReadyMessage {
            wantsBallPredictions: true,
            wantsQuickChat: false,
            wantsGameMessages: true,
        }))
        .unwrap();

    let car_id = env::var("CAR_ID")
        .map(|x| x.parse().unwrap())
        .unwrap_or(DEFAULT_CAR_ID);

    loop {
        let Packet::GameTickPacket(game_tick_packet) = rlbot_connection.recv_packet().unwrap()
        else {
            continue;
        };
        let target = game_tick_packet.ball.unwrap().physics.unwrap();
        let car = game_tick_packet
            .players
            .unwrap()
            .get(car_id)
            .unwrap()
            .physics
            .clone()
            .unwrap();

        let bot_to_target_angle = (target.location.clone().unwrap().y
            - car.location.clone().unwrap().y)
            .atan2(target.location.unwrap().x - car.location.unwrap().x);

        let mut bot_front_to_target_angle = bot_to_target_angle - car.rotation.unwrap().yaw;

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
                playerIndex: car_id as i32,
                controllerState: Some(Box::new(controller)),
            }))
            .unwrap();
    }
}
