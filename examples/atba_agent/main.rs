use std::{env, f32::consts::PI};

use rlbot_interface::{
    agents::{read_spawn_ids, run_agents, Agent},
    rlbot::{ControllerState, PlayerInput},
    Packet, RLBotConnection,
};

struct AtbaAgent {
    spawn_id: i32,
}

impl Agent for AtbaAgent {
    fn new(spawn_id: i32) -> Self {
        Self { spawn_id }
    }
    fn tick(&mut self, game_tick_packet: rlbot_interface::rlbot::GameTickPacket) -> Vec<Packet> {
        let mut packets_to_send = vec![];

        let Some(bot_index) = game_tick_packet
            .players
            .iter()
            .position(|x| x.spawn_id == self.spawn_id)
        else {
            // If we aren't in the game, don't do anything
            return packets_to_send;
        };

        let Some(ball) = game_tick_packet.balls.first() else {
            // If theres no ball, theres nothing to chase, don't do anything
            return packets_to_send;
        };

        let target = &ball.physics;
        let car = game_tick_packet
            .players
            .get(bot_index)
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

        packets_to_send.push(
            PlayerInput {
                player_index: bot_index as u32,
                controller_state: Box::new(controller),
            }
            .into(),
        );
        packets_to_send
    }
}
fn main() {
    println!("Connecting");

    let rlbot_addr = env::var("RLBOT_CORE_ADDR").unwrap_or("127.0.0.1:23234".to_owned());

    let rlbot_connection = RLBotConnection::new(&rlbot_addr).expect("connection");

    println!("Running!");

    // The hivemind field in your bot.toml file decides if rlbot core is going to
    // start your bot as one or multiple instances of your binary/exe.
    // If the hivemind field is set to true, one instance of your bot will handle
    // all of the bots in a team.
    let spawn_ids = read_spawn_ids();

    // Blocking
    run_agents::<AtbaAgent>(&spawn_ids, rlbot_connection).expect("to run agent");

    println!("Spawn ids {spawn_ids:?} exited nicely")
}
