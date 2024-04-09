use rlbot_lib::{
    rlbot::{
        ExistingMatchBehavior, GameMap, GameMode, MatchLength, MatchSettings, MutatorSettings,
        PlayerClass, PlayerConfiguration, PlayerLoadout, RLBotPlayer,
    },
    Packet, RLBotConnection,
};

fn main() {
    println!("Connecting");

    let mut rlbot_connection = RLBotConnection::new("127.0.0.1:23234").expect("connection");

    println!("Starting match");

    let match_settings = MatchSettings {
        playerConfigurations: Some(vec![
            PlayerConfiguration {
                variety: PlayerClass::RLBotPlayer(Box::new(RLBotPlayer {})),
                name: Some("BOT1".to_owned()),
                team: 0,
                loadout: Some(Box::new(PlayerLoadout::default())),
                spawnId: 0,
            },
            PlayerConfiguration {
                variety: PlayerClass::RLBotPlayer(Box::new(RLBotPlayer {})),
                name: Some("BOT2".to_owned()),
                team: 1,
                loadout: Some(Box::new(PlayerLoadout::default())),
                spawnId: 1,
            },
        ]),
        gameMode: GameMode::Soccer,
        gameMap: GameMap::DFHStadium,
        // mutatorSettings CANNOT be None, otherwise RLBot will crash
        mutatorSettings: Some(Box::new(MutatorSettings {
            matchLength: MatchLength::Unlimited,
            ..Default::default()
        })),
        existingMatchBehavior: ExistingMatchBehavior::Restart,
        enableRendering: true,
        ..Default::default()
    };

    rlbot_connection
        .send_packet(Packet::MatchSettings(match_settings))
        .expect("start_match");
}
