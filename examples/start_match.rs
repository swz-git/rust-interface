use rlbot_interface::{
    rlbot::{
        ExistingMatchBehavior, GameMode, Human, MatchLength, MatchSettings, MutatorSettings,
        PlayerClass, PlayerConfiguration, PlayerLoadout, RLBot,
    },
    RLBotConnection,
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
                    loadout_paint: Some(Box::default()),
                    ..Default::default()
                })),
                spawn_id: Default::default(),
                root_dir: Default::default(),
                agent_id: "rlbot/rust-example-bot".into(),
                run_command: Default::default(),
                hivemind: Default::default(),
            },
            PlayerConfiguration {
                variety: PlayerClass::Human(Box::new(Human {})),
                name: "".to_owned(),
                team: 1,
                loadout: None,
                spawn_id: Default::default(),
                root_dir: Default::default(),
                agent_id: Default::default(),
                run_command: Default::default(),
                hivemind: Default::default(),
            },
        ],
        game_mode: GameMode::Soccer,
        game_map_upk: "UtopiaStadium_P".into(),
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
        .send_packet(match_settings)
        .expect("start_match");
}
