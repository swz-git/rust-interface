use std::env::args;

use rlbot::{
    flat::{
        ExistingMatchBehavior, GameMode, Human, MatchLength, MatchSettings, MutatorSettings,
        PlayerClass, PlayerConfiguration, RLBot,
    },
    RLBotConnection,
};

fn main() {
    println!("Connecting");

    let mut rlbot_connection = RLBotConnection::new("127.0.0.1:23234").expect("connection");

    println!("Starting match");

    // Usage: ./start_match 5 your-agent-id
    let bots_to_add = args()
        .skip(1)
        .next()
        .map(|x| x.parse().unwrap())
        .unwrap_or(1);
    let agent_id = args()
        .skip(2)
        .next()
        .unwrap_or("rlbot/rust-example-bot".into());

    let mut player_configurations = (0..bots_to_add)
        .into_iter()
        .map(|i| PlayerConfiguration {
            variety: PlayerClass::RLBot(Box::new(RLBot {})),
            name: format!("BOT{i}"),
            team: i % 2,
            root_dir: "".into(),
            run_command: "".into(),
            loadout: None,
            spawn_id: 0, // RLBotServer will set this
            agent_id: agent_id.clone(),
            hivemind: true,
        })
        .collect::<Vec<_>>();

    // Also add a human
    player_configurations.push(PlayerConfiguration {
        variety: PlayerClass::Human(Box::new(Human {})),
        name: "".to_owned(),
        team: 1,
        loadout: None,
        spawn_id: Default::default(),
        root_dir: Default::default(),
        agent_id: Default::default(),
        run_command: Default::default(),
        hivemind: Default::default(),
    });

    let match_settings = MatchSettings {
        player_configurations,
        game_mode: GameMode::Soccer,
        game_map_upk: "UtopiaStadium_P".into(),
        // mutatorSettings CANNOT be None, otherwise RLBot will crash (this is true for v4, maybe not v5)
        mutator_settings: Some(Box::new(MutatorSettings {
            match_length: MatchLength::Unlimited,
            ..Default::default()
        })),
        existing_match_behavior: ExistingMatchBehavior::Restart,
        enable_rendering: true,
        enable_state_setting: true,
        ..Default::default()
    };

    rlbot_connection
        .send_packet(match_settings)
        .expect("start_match");
}
