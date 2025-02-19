use std::f32::consts::PI;

use rlbot::{
    agents::{run_agents, Agent, PacketQueue},
    flat::{ConnectionSettings, ControllableInfo, ControllerState, PlayerInput},
    util::RLBotEnvironment,
    RLBotConnection,
};

struct AtbaAgent {
    controllable_info: ControllableInfo,
}

impl Agent for AtbaAgent {
    fn new(controllable_info: ControllableInfo) -> Self {
        Self { controllable_info }
    }
    fn tick(&mut self, game_packet: &rlbot::flat::GamePacket, packet_queue: &mut PacketQueue) {
        let Some(ball) = game_packet.balls.first() else {
            // If theres no ball, theres nothing to chase, don't do anything
            return;
        };

        // We're not in the gtp, skip this tick
        if game_packet.players.len() <= self.controllable_info.index as usize {
            return;
        }

        let target = &ball.physics;
        let car = game_packet
            .players
            .get(self.controllable_info.index as usize)
            .unwrap()
            .physics;

        let bot_to_target_angle = f32::atan2(
            target.location.y - car.location.y,
            target.location.x - car.location.x,
        );

        let mut bot_front_to_target_angle = bot_to_target_angle - car.rotation.yaw;

        bot_front_to_target_angle = (bot_front_to_target_angle + PI).rem_euclid(2. * PI) - PI;

        let mut controller = ControllerState::default();

        if bot_front_to_target_angle > 0. {
            controller.steer = 1.;
        } else {
            controller.steer = -1.;
        }

        controller.throttle = 1.;

        packet_queue.push(PlayerInput {
            player_index: self.controllable_info.index,
            controller_state: controller,
        });
    }
}
fn main() {
    let RLBotEnvironment {
        server_addr,
        agent_id,
    } = RLBotEnvironment::from_env();
    let agent_id = agent_id.unwrap_or("rlbot/rust-example/atba_agent".into());

    println!("Connecting");

    let rlbot_connection = RLBotConnection::new(&server_addr).expect("connection");

    println!("Running!");

    // The hivemind field in your bot.toml file decides if rlbot core is going to
    // start your bot as one or multiple instances of your binary/exe.
    // If the hivemind field is set to true, one instance of your bot will handle
    // all of the bots in a team.

    // Blocking
    run_agents::<AtbaAgent>(
        ConnectionSettings {
            agent_id: agent_id.clone(),
            wants_ball_predictions: true,
            wants_comms: true,
            close_between_matches: true,
        },
        rlbot_connection,
    )
    .expect("run_agents crashed");

    println!("Agent(s) with agent_id `{agent_id}` exited nicely")
}
