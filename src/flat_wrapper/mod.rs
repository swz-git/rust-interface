#[allow(clippy::all, dead_code)]
pub(crate) mod planus_flat;
pub use planus_flat::rlbot::flat as rlbot;

#[cfg(feature = "glam")]
impl From<rlbot::Vector3> for glam::Vec3 {
    fn from(value: rlbot::Vector3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "glam")]
impl From<rlbot::Vector3> for glam::Vec3A {
    fn from(value: rlbot::Vector3) -> Self {
        Self::new(value.x, value.y, value.z)
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
