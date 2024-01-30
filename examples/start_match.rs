use rlbot_interface::{
    rlbot::{
        ExistingMatchBehavior, GameMode, Human, LoadoutPaint, MatchLength, MatchSettings,
        MutatorSettings, PlayerClass, PlayerConfiguration, PlayerLoadout, RLBot,
    },
    Packet, RLBotConnection,
};

fn main() {
    println!("Connecting");

    let mut rlbot_connection = RLBotConnection::new("127.0.0.1:23234").expect("connection");

    println!("Starting match");

    let match_settings = MatchSettings {
        player_configurations: vec![
            PlayerConfiguration {
                variety: PlayerClass::RLBot(Box::new(RLBot {})),
                name: "BOT1".to_owned(),
                team: 0,
                loadout: Some(Box::new(PlayerLoadout {
                    loadout_paint: Some(Box::new(LoadoutPaint::default())),
                    ..Default::default()
                })),
                spawn_id: 0,
                ..Default::default()
            },
            PlayerConfiguration {
                variety: PlayerClass::Human(Box::new(Human {})),
                name: "".to_owned(),
                team: 1,
                loadout: None,
                spawn_id: 1,
                ..Default::default()
            },
        ],
        game_mode: GameMode::Soccer,
        game_map_upk: "UtopiaStadium_P".to_owned(),
        // mutatorSettings CANNOT be None, otherwise RLBot will crash (this is true for v4, maybe not v5)
        mutator_settings: Some(Box::new(MutatorSettings {
            match_length: MatchLength::Unlimited,
            ..Default::default()
        })),
        existing_match_behavior: ExistingMatchBehavior::Restart,
        enable_rendering: true,
        ..Default::default()
    };

    rlbot_connection
        .send_packet(Packet::MatchSettings(match_settings))
        .expect("start_match");
}
