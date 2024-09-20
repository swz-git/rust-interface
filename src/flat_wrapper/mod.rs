// #[allow(non_snake_case, unused, clippy::all)]
// pub(crate) mod generated;

#[path = "rlbot.rs"]
#[allow(non_snake_case, unused, clippy::all)]
pub(crate) mod rlbot_flat;
pub use rlbot_flat::rlbot::flat as rlbot;

impl Default for rlbot::GameTickPacket {
    fn default() -> Self {
        todo!()
    }
}

// mod renamed_objects;

// pub mod rlbot {
//     pub use super::renamed_objects::*;
// }

#[cfg(feature = "glam")]
impl Into<glam::Vec3> for rlbot::Vector3 {
    fn into(self) -> glam::Vec3 {
        glam::Vec3::new(self.x, self.y, self.z)
    }
}

#[cfg(feature = "glam")]
impl Into<glam::Vec3A> for rlbot::Vector3 {
    fn into(self) -> glam::Vec3A {
        glam::Vec3A::new(self.x, self.y, self.z)
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec3> for rlbot::Vector3 {
    fn from(value: glam::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec3A> for rlbot::Vector3 {
    fn from(value: glam::Vec3A) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
