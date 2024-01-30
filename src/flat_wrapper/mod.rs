#[allow(non_snake_case, unused, clippy::all)]
pub(crate) mod generated;

mod renamed_objects;

pub mod rlbot {
    pub use super::renamed_objects::*;
}

#[cfg(feature = "glam")]
impl From<rlbot::Vector3> for glam::Vec3 {
    fn from(val: rlbot::Vector3) -> Self {
        glam::Vec3 {
            x: val.x,
            y: val.y,
            z: val.z,
        }
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
impl From<rlbot::Vector3> for glam::Vec3A {
    fn from(val: rlbot::Vector3) -> Self {
        glam::Vec3A::new(val.x, val.y, val.z)
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

#[cfg(feature = "glam")]
impl From<&rlbot::Vector3> for glam::Vec3 {
    fn from(val: &rlbot::Vector3) -> Self {
        glam::Vec3 {
            x: val.x,
            y: val.y,
            z: val.z,
        }
    }
}

#[cfg(feature = "glam")]
impl From<&glam::Vec3> for rlbot::Vector3 {
    fn from(value: &glam::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "glam")]
impl From<&rlbot::Vector3> for glam::Vec3A {
    fn from(val: &rlbot::Vector3) -> Self {
        glam::Vec3A::new(val.x, val.y, val.z)
    }
}

#[cfg(feature = "glam")]
impl From<&glam::Vec3A> for rlbot::Vector3 {
    fn from(value: &glam::Vec3A) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
