use rlbot_interface::{rlbot::StopCommand, Packet, RLBotConnection};

fn main() {
    println!("Connecting");

    let mut rlbot_connection = RLBotConnection::new("127.0.0.1:23234").expect("connection");

    println!("Stopping match");

    rlbot_connection
        .send_packet(Packet::StopCommand(StopCommand {
            shutdown_server: false,
        }))
        .expect("start_match");
}
