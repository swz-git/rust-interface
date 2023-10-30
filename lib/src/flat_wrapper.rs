use flatbuffers::FlatBufferBuilder;

use crate::rlbot_generated::rlbot::flat::{self};
pub use crate::rlbot_generated::rlbot::flat::{
    BallBouncinessOption, BallMaxSpeedOption, BallSizeOption, BallTypeOption, BallWeightOption,
    BoostOption, BoostStrengthOption, DemolishOption, ExistingMatchBehavior, GameMap, GameMode,
    GameSpeedOption, GravityOption, MatchLength, MaxScore, OvertimeOption, RespawnTimeOption,
    RumbleOption, SeriesLengthOption,
};

#[derive(Default)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn to_flat<'a, 'b>(
        &'a self,
        mut builder: &'b mut FlatBufferBuilder<'a>,
    ) -> flatbuffers::WIPOffset<flat::Color<'a>> {
        flat::Color::create(
            &mut builder,
            &flat::ColorArgs {
                a: self.a,
                r: self.r,
                g: self.g,
                b: self.b,
            },
        )
    }
}

pub struct LoadoutPaint {
    pub car_paint_id: i32,
    pub decal_paint_id: i32,
    pub wheels_paint_id: i32,
    pub boost_paint_id: i32,
    pub antenna_paint_id: i32,
    pub hat_paint_id: i32,
    pub trails_paint_id: i32,
    pub goal_explosion_paint_id: i32,
}

impl LoadoutPaint {
    pub fn to_flat<'a, 'b>(
        &'a self,
        mut builder: &'b mut FlatBufferBuilder<'a>,
    ) -> flatbuffers::WIPOffset<flat::LoadoutPaint<'a>> {
        flat::LoadoutPaint::create(
            &mut builder,
            &flat::LoadoutPaintArgs {
                carPaintId: self.car_paint_id,
                decalPaintId: self.decal_paint_id,
                wheelsPaintId: self.wheels_paint_id,
                boostPaintId: self.boost_paint_id,
                antennaPaintId: self.antenna_paint_id,
                hatPaintId: self.hat_paint_id,
                trailsPaintId: self.trails_paint_id,
                goalExplosionPaintId: self.goal_explosion_paint_id,
            },
        )
    }
}

pub struct PlayerLoadout {
    pub team_color_id: i32,
    pub custom_color_id: i32,
    pub car_id: i32,
    pub decal_id: i32,
    pub wheels_id: i32,
    pub boost_id: i32,
    pub antenna_id: i32,
    pub hat_id: i32,
    pub paint_finish_id: i32,
    pub custom_finish_id: i32,
    pub engine_audio_id: i32,
    pub trails_id: i32,
    pub goal_explosion_id: i32,
    pub loadout_paint: Option<LoadoutPaint>,
    /// Sets the primary color of the car to the swatch that most closely matches the provided
    /// RGB color value. If set, this overrides teamColorId.
    pub primary_color_lookup: Option<Color>,
    /// Sets the secondary color of the car to the swatch that most closely matches the provided
    /// RGB color value. If set, this overrides customColorId.
    pub secondary_color_lookup: Option<Color>,
}

impl PlayerLoadout {
    pub fn to_flat<'a, 'b>(
        &'a self,
        mut builder: &'b mut FlatBufferBuilder<'a>,
    ) -> flatbuffers::WIPOffset<flat::PlayerLoadout<'a>> {
        let loadout_paint = match &self.loadout_paint {
            Some(x) => Some(x.to_flat(&mut builder)),
            None => None,
        };
        let primary_color_lookup = match &self.primary_color_lookup {
            Some(x) => Some(x.to_flat(&mut builder)),
            None => None,
        };
        let secondary_color_lookup = match &self.secondary_color_lookup {
            Some(x) => Some(x.to_flat(&mut builder)),
            None => None,
        };
        flat::PlayerLoadout::create(
            &mut builder,
            &flat::PlayerLoadoutArgs {
                teamColorId: self.team_color_id,
                customColorId: self.custom_color_id,
                carId: self.car_id,
                decalId: self.decal_id,
                wheelsId: self.wheels_id,
                boostId: self.boost_id,
                antennaId: self.antenna_id,
                hatId: self.hat_id,
                paintFinishId: self.paint_finish_id,
                customFinishId: self.custom_finish_id,
                engineAudioId: self.engine_audio_id,
                trailsId: self.trails_id,
                goalExplosionId: self.goal_explosion_id,
                loadoutPaint: loadout_paint,
                primaryColorLookup: primary_color_lookup,
                secondaryColorLookup: secondary_color_lookup,
            },
        )
    }
}

pub enum PlayerClass {
    RLBotPlayer,
    HumanPlayer,
    PsyonixBotPlayer(f32),
    PartyMemberBotPlayer,
}

pub struct PlayerConfiguration {
    pub player_class: PlayerClass,
    pub name: Option<String>,
    pub team: i32,
    pub loadout: Option<PlayerLoadout>,
    pub spawn_id: i32,
}

impl PlayerConfiguration {
    pub fn to_flat<'a, 'b>(
        &'a self,
        mut builder: &'b mut FlatBufferBuilder<'a>,
    ) -> flatbuffers::WIPOffset<flat::PlayerConfiguration<'a>> {
        let name = match &self.name {
            Some(x) => Some(builder.create_string(&x)),
            None => None,
        };
        let loadout = match &self.loadout {
            Some(x) => Some(x.to_flat(&mut builder)),
            None => None,
        };
        let (variety, variety_type) = match self.player_class {
            PlayerClass::RLBotPlayer => {
                let player = flat::RLBotPlayer::create(builder, &flat::RLBotPlayerArgs {});
                (
                    Some(player.as_union_value()),
                    flat::PlayerClass(player.value() as u8),
                )
            }
            PlayerClass::HumanPlayer => {
                let player = flat::HumanPlayer::create(builder, &flat::HumanPlayerArgs {});
                (
                    Some(player.as_union_value()),
                    flat::PlayerClass(player.value() as u8),
                )
            }
            PlayerClass::PsyonixBotPlayer(skill) => {
                let player = flat::PsyonixBotPlayer::create(
                    builder,
                    &flat::PsyonixBotPlayerArgs { botSkill: skill },
                );
                (
                    Some(player.as_union_value()),
                    flat::PlayerClass(player.value() as u8),
                )
            }
            PlayerClass::PartyMemberBotPlayer => {
                let player =
                    flat::PartyMemberBotPlayer::create(builder, &flat::PartyMemberBotPlayerArgs {});
                (
                    Some(player.as_union_value()),
                    flat::PlayerClass(player.value() as u8),
                )
            }
        };
        flat::PlayerConfiguration::create(
            &mut builder,
            &flat::PlayerConfigurationArgs {
                variety,
                variety_type,
                name,
                team: self.team,
                loadout,
                spawnId: self.spawn_id,
            },
        )
    }
}

pub struct MutatorSettings {
    pub match_length: MatchLength,
    pub max_score: MaxScore,
    pub overtime_option: OvertimeOption,
    pub series_length_option: SeriesLengthOption,
    pub game_speed_option: GameSpeedOption,
    pub ball_max_speed_option: BallMaxSpeedOption,
    pub ball_type_option: BallTypeOption,
    pub ball_weight_option: BallWeightOption,
    pub ball_size_option: BallSizeOption,
    pub ball_bounciness_option: BallBouncinessOption,
    pub boost_option: BoostOption,
    pub rumble_option: RumbleOption,
    pub boost_strength_option: BoostStrengthOption,
    pub gravity_option: GravityOption,
    pub demolish_option: DemolishOption,
    pub respawn_time_option: RespawnTimeOption,
}

impl MutatorSettings {
    pub fn to_flat<'a, 'b>(
        &'a self,
        mut builder: &'b mut FlatBufferBuilder<'a>,
    ) -> flatbuffers::WIPOffset<flat::MutatorSettings<'a>> {
        flat::MutatorSettings::create(
            &mut builder,
            &flat::MutatorSettingsArgs {
                matchLength: self.match_length,
                maxScore: self.max_score,
                overtimeOption: self.overtime_option,
                seriesLengthOption: self.series_length_option,
                gameSpeedOption: self.game_speed_option,
                ballMaxSpeedOption: self.ball_max_speed_option,
                ballTypeOption: self.ball_type_option,
                ballWeightOption: self.ball_weight_option,
                ballSizeOption: self.ball_size_option,
                ballBouncinessOption: self.ball_bounciness_option,
                boostOption: self.boost_option,
                rumbleOption: self.rumble_option,
                boostStrengthOption: self.boost_strength_option,
                gravityOption: self.gravity_option,
                demolishOption: self.demolish_option,
                respawnTimeOption: self.respawn_time_option,
            },
        )
    }
}

pub struct MatchSettings {
    pub player_configurations: Option<Vec<PlayerConfiguration>>,
    pub game_mode: GameMode,
    pub game_map: GameMap,
    pub skip_replays: bool,
    pub instant_start: bool,
    pub mutator_settings: Option<MutatorSettings>,
    pub existing_match_behavior: ExistingMatchBehavior,
    pub enable_lockstep: bool,
    pub enable_rendering: bool,
    pub enable_state_setting: bool,
    pub auto_save_replay: bool,
    pub game_map_upk: Option<String>,
}

impl MatchSettings {
    pub fn to_flat<'a, 'b>(
        &'a self,
        mut builder: &'b mut FlatBufferBuilder<'a>,
    ) -> flatbuffers::WIPOffset<flat::MatchSettings<'a>> {
        let player_configurations = match &self.player_configurations {
            Some(x) => Some(
                x.iter()
                    .map(|x| x.to_flat(&mut builder))
                    .collect::<Vec<_>>(),
            ),
            None => None,
        };
        let player_configurations = match player_configurations {
            Some(x) => Some(builder.create_vector(&x)),
            None => None,
        };
        let mutator_settings = match &self.mutator_settings {
            Some(x) => Some(x.to_flat(&mut builder)),
            None => None,
        };
        let game_map_upk = match &self.game_map_upk {
            Some(x) => Some(builder.create_string(&x)),
            None => None,
        };
        flat::MatchSettings::create(
            &mut builder,
            &flat::MatchSettingsArgs {
                playerConfigurations: player_configurations,
                gameMode: self.game_mode,
                gameMap: self.game_map,
                skipReplays: self.skip_replays,
                instantStart: self.instant_start,
                mutatorSettings: mutator_settings,
                existingMatchBehavior: self.existing_match_behavior,
                enableLockstep: self.enable_lockstep,
                enableRendering: self.enable_rendering,
                enableStateSetting: self.enable_state_setting,
                autoSaveReplay: self.auto_save_replay,
                gameMapUpk: game_map_upk,
            },
        )
    }
}
