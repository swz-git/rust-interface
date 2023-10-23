use rlbot_match_manager;

fn main() {
    println!("Connecting");
    let mut conn = rlbot_match_manager::connect("127.0.0.1:23234").expect("connection");
    println!("Starting match");
    rlbot_match_manager::start_match(&mut conn).expect("start_match")
}
