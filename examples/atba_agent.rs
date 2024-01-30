use std::env;

use rlbot_interface::{
    agents::{run_agent, Agent},
    rlbot::ControllerState,
    RLBotConnection,
};

struct AtbaAgent {
    index: i32,
}

impl Agent for AtbaAgent {
    fn new(index: i32, _connection: &mut RLBotConnection) -> Self {
        Self { index }
    }
    fn tick(
        &mut self,
        game_tick_packet: rlbot_interface::rlbot::GameTickPacket,
        _connection: &mut RLBotConnection,
    ) -> ControllerState {
        let target = game_tick_packet.ball.physics;
        let car = game_tick_packet
            .players
            .get(self.index as usize)
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

        controller
    }
}
fn main() {
    println!("Connecting");

    let rlbot_addr = env::var("RLBOT_CORE_ADDR").unwrap_or("127.0.0.1:23234".to_owned());

    let rlbot_connection = RLBotConnection::new(&rlbot_addr).expect("connection");

    println!("Running!");

    let index = env::var("RLBOT_INDEX")
        .map(|x| x.parse().unwrap())
        .unwrap_or(0);

    run_agent::<AtbaAgent>(index, rlbot_connection).expect("to run agent");
}
