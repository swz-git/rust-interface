use crate::{rlbot::*, Packet, RLBotConnection};

#[allow(unused_variables)]
pub trait Bot {
    fn new(index: i32, connection: &mut RLBotConnection) -> Self;
    fn tick(
        &mut self,
        game_tick_packet: GameTickPacket,
        connection: &mut RLBotConnection,
    ) -> ControllerState;
    fn on_field_info(&mut self, field_info: FieldInfo) {}
    fn on_match_settings(&mut self, match_settings: MatchSettings) {}
    fn on_quick_chat(&mut self, quick_chat: QuickChat) {}
    fn on_ball_prediction(&mut self, ball_prediction: BallPrediction) {}
    fn on_message_packet(&mut self, message_packet: MessagePacket) {}
}

pub fn run_bot<T: Bot>(
    index: i32,
    mut connection: RLBotConnection,
) -> Result<(), crate::RLBotError> {
    let mut bot = T::new(index, &mut connection);

    connection.send_packet(Packet::ReadyMessage(ReadyMessage {
        wantsBallPredictions: true,
        wantsQuickChat: true,
        wantsGameMessages: true,
    }))?;

    loop {
        let packet = match connection.recv_packet() {
            Ok(packet) => packet,
            Err(e) => Err(e)?,
        };

        match packet {
            Packet::GameTickPacket(x) => {
                let controller_state = bot.tick(x, &mut connection);
                connection.send_packet(Packet::PlayerInput(PlayerInput {
                    playerIndex: index,
                    controllerState: Some(Box::new(controller_state)),
                }))?;
            }
            Packet::FieldInfo(x) => bot.on_field_info(x),
            Packet::MatchSettings(x) => bot.on_match_settings(x),
            Packet::QuickChat(x) => bot.on_quick_chat(x),
            Packet::BallPrediction(x) => bot.on_ball_prediction(x),
            Packet::MessagePacket(x) => bot.on_message_packet(x),
            _ => { /* The rest of the packets are only client -> server */ }
        }
    }

    // Ok(())
}
