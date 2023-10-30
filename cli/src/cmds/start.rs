use clap::{Args, ValueEnum};
use color_eyre::eyre::Result;
use log::info;
use rlbot_match_manager_lib::{flat_wrapper::*, RLBotConnection};

#[derive(Clone, ValueEnum)]
enum PsyonixBotDifficulty {
    Rookie,
    Pro,
    AllStar,
}

impl PsyonixBotDifficulty {
    fn value(&self) -> f32 {
        match self {
            PsyonixBotDifficulty::Rookie => 0f32,
            PsyonixBotDifficulty::Pro => 0.5f32,
            PsyonixBotDifficulty::AllStar => 1f32,
        }
    }
}

#[derive(ValueStruct)]
struct UnknownRLBot {
    name: String,
    team: usize,
}

#[derive(Args)]
pub struct CommandArgs {
    /// Add a unknown RLBot with name
    #[arg(long)]
    player_unknown_rlbot: Vec<UnknownRLBot>,
    /// Add a human to the match
    #[arg(long)]
    player_human: Option<bool>,
    /// Add a psyonix bot
    #[arg(long)]
    player_psyonix: Vec<PsyonixBotDifficulty>,
}

impl CommandArgs {
    pub fn run(&self) -> Result<()> {
        info!("Connecting to RLBot");

        let mut rlbot_connection = RLBotConnection::new("127.0.0.1:23234").expect("connection");

        info!("Starting match");

        let player_configuration = vec![];

        for unknown_rlbot_name in self.player_unknown_rlbot {
            player_configuration.push(PlayerConfiguration {
                player_class: PlayerClass::RLBotPlayer,
                name: Some(unknown_rlbot_name),
                team,
            })
        }

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

        rlbot_connection.start_match(match_settings)?;

        info!("");

        Ok(())
    }
}
