use crate::{rlbot::*, Packet, RLBotConnection};

#[allow(unused_variables)]
pub trait Agent {
    fn new(index: u32, connection: &mut RLBotConnection) -> Self;
    fn tick(
        &mut self,
        game_tick_packet: GameTickPacket,
        connection: &mut RLBotConnection,
    ) -> ControllerState;
    fn on_field_info(&mut self, field_info: FieldInfo) {}
    fn on_match_settings(&mut self, match_settings: MatchSettings) {}
    fn on_match_comm(&mut self, match_comm: MatchComm) {}
    fn on_ball_prediction(&mut self, ball_prediction: BallPrediction) {}
}

/// Ok(()) means a succesfull exit; bot received a None packet.
pub fn run_agent<T: Agent>(
    index: u32,
    mut connection: RLBotConnection,
) -> Result<(), crate::RLBotError> {
    let mut bot = T::new(index, &mut connection);

    connection.send_packet(Packet::ReadyMessage(ReadyMessage {
        wants_ball_predictions: true,
        wants_comms: true,
        // wants_game_messages: true,
        close_after_match: true,
    }))?;

    loop {
        let packet = connection.recv_packet()?;

        match packet {
            Packet::None => return Ok(()),
            Packet::GameTickPacket(x) => {
                let controller_state = bot.tick(x, &mut connection);
                connection.send_packet(Packet::PlayerInput(PlayerInput {
                    player_index: index,
                    controller_state: Box::new(controller_state),
                }))?;
            }
            Packet::FieldInfo(x) => bot.on_field_info(x),
            Packet::MatchSettings(x) => bot.on_match_settings(x),
            Packet::MatchComm(x) => bot.on_match_comm(x),
            Packet::BallPrediction(x) => bot.on_ball_prediction(x),
            _ => { /* The rest of the packets are only client -> server */ }
        }
    }

    // Ok(())
}
