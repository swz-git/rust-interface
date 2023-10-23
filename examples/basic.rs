use rlbot_match_manager::{
    self,
    flat_wrapper::{
        ExistingMatchBehavior, GameMap, GameMode, MatchSettings, PlayerClass, PlayerConfiguration,
    },
    RLBotConnection,
};

fn main() {
    println!("Connecting");

    let mut rlbot_connection = RLBotConnection::new("127.0.0.1:23234").expect("connection");

    println!("Starting match");

    let match_settings = MatchSettings {
        player_configurations: Some(vec![
            PlayerConfiguration {
                player_class: PlayerClass::RLBotPlayer,
                name: Some("Test bot".to_owned()),
                team: 0,
                loadout: None,
                spawn_id: 0,
            },
            PlayerConfiguration {
                player_class: PlayerClass::RLBotPlayer,
                name: Some("Test bot 2".to_owned()),
                team: 1,
                loadout: None,
                spawn_id: 1,
            },
        ]),
        game_mode: GameMode::Soccer,
        game_map: GameMap::DFHStadium,
        skip_replays: false,
        instant_start: true,
        mutator_settings: None,
        existing_match_behavior: ExistingMatchBehavior::Restart,
        enable_lockstep: false,
        enable_rendering: true,
        enable_state_setting: true,
        auto_save_replay: false,
        game_map_upk: None,
    };

    rlbot_connection
        .start_match(match_settings)
        .expect("start_match")
}
