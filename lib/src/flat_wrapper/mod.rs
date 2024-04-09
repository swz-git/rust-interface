#[allow(non_snake_case, unused_imports)]
#[allow(clippy::all)]
pub(super) mod rlbot_generated;

use glam::Vec3A;
// export structs without annoying suffix >:(
pub use rlbot_generated::rlbot::flat::{
    BallInfoObject as BallInfo, BallPredictionObject as BallPrediction,
    BallRigidBodyStateObject as BallRigidBodyState, BoolObject as Bool, BoostPadObject as BoostPad,
    BoostPadStateObject as BoostPadState, BoxShapeObject as BoxShape,
    CollisionShapeObject as CollisionShape, ColorObject as Color,
    ConsoleCommandObject as ConsoleCommand, ControllerStateObject as ControllerState,
    CylinderShapeObject as CylinderShape, DesiredBallStateObject as DesiredBallState,
    DesiredBoostStateObject as DesiredBoostState, DesiredCarStateObject as DesiredCarState,
    DesiredGameInfoStateObject as DesiredGameInfoState, DesiredGameStateObject as DesiredGameState,
    DesiredPhysicsObject as DesiredPhysics, DropShotBallInfoObject as DropShotBallInfo,
    DropshotTileObject as DropshotTile, FieldInfoObject as FieldInfo, FloatObject as Float,
    GameInfoObject as GameInfo, GameMessageObject as GameMessage,
    GameMessageWrapperObject as GameMessageWrapper, GameTickPacketObject as GameTickPacket,
    GoalInfoObject as GoalInfo, HumanPlayerObject as HumanPlayer,
    LoadoutPaintObject as LoadoutPaint, MatchSettingsObject as MatchSettings,
    MessagePacketObject as MessagePacket, MutatorSettingsObject as MutatorSettings,
    PartyMemberBotPlayerObject as PartyMemberBotPlayer, PhysicsObject as Physics,
    PlayerClassObject as PlayerClass, PlayerConfigurationObject as PlayerConfiguration,
    PlayerInfoObject as PlayerInfo, PlayerInputChangeObject as PlayerInputChange,
    PlayerInputObject as PlayerInput, PlayerLoadoutObject as PlayerLoadout,
    PlayerRigidBodyStateObject as PlayerRigidBodyState, PlayerSpectateObject as PlayerSpectate,
    PlayerStatEventObject as PlayerStatEvent, PredictionSliceObject as PredictionSlice,
    PsyonixBotPlayerObject as PsyonixBotPlayer, QuaternionObject as Quaternion,
    QuickChatMessagesObject as QuickChatMessages, QuickChatObject as QuickChat,
    RLBotPlayerObject as RLBotPlayer, ReadyMessageObject as ReadyMessage,
    RenderGroupObject as RenderGroup, RenderMessageObject as RenderMessage,
    RigidBodyStateObject as RigidBodyState, RigidBodyTickObject as RigidBodyTick,
    RotatorObject as Rotator, RotatorPartialObject as RotatorPartial, ScoreInfoObject as ScoreInfo,
    SphereShapeObject as SphereShape, TeamInfoObject as TeamInfo, TinyBallObject as TinyBall,
    TinyPacketObject as TinyPacket, TinyPlayerObject as TinyPlayer, TouchObject as Touch,
    Vector3Object as Vector3, Vector3PartialObject as Vector3Partial,
};

#[cfg(feature = "glam")]
impl From<Vector3> for glam::Vec3 {
    fn from(val: Vector3) -> Self {
        glam::Vec3 {
            x: val.x,
            y: val.y,
            z: val.z,
        }
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec3> for Vector3 {
    fn from(value: glam::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "glam")]
impl From<Vector3> for glam::Vec3A {
    fn from(val: Vector3) -> Self {
        Vec3A::new(val.x, val.y, val.z)
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec3A> for Vector3 {
    fn from(value: glam::Vec3A) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "glam")]
impl From<&Vector3> for glam::Vec3 {
    fn from(val: &Vector3) -> Self {
        glam::Vec3 {
            x: val.x,
            y: val.y,
            z: val.z,
        }
    }
}

#[cfg(feature = "glam")]
impl From<&glam::Vec3> for Vector3 {
    fn from(value: &glam::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "glam")]
impl From<&Vector3> for glam::Vec3A {
    fn from(val: &Vector3) -> Self {
        Vec3A::new(val.x, val.y, val.z)
    }
}

#[cfg(feature = "glam")]
impl From<&glam::Vec3A> for Vector3 {
    fn from(value: &glam::Vec3A) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

// export enums
pub use rlbot_generated::rlbot::flat::{
    BallBouncinessOption, BallMaxSpeedOption, BallSizeOption, BallTypeOption, BallWeightOption,
    BoostOption, BoostStrengthOption, DemolishOption, ExistingMatchBehavior, GameMap, GameMode,
    GameSpeedOption, GravityOption, MatchLength, MaxScore, OvertimeOption, QuickChatSelection,
    RenderType, RespawnTimeOption, RumbleOption, SeriesLengthOption, TileState,
};
