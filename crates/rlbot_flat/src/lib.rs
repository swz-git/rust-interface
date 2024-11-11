#[allow(clippy::all, dead_code)]
pub(crate) mod planus_flat;
pub use planus;
pub use planus_flat::rlbot::flat;

#[cfg(feature = "glam")]
pub use glam;

#[cfg(feature = "glam")]
impl From<flat::Vector3> for glam::Vec3 {
    fn from(value: flat::Vector3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "glam")]
impl From<flat::Vector3> for glam::Vec3A {
    fn from(value: flat::Vector3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec3> for flat::Vector3 {
    fn from(value: glam::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec3A> for flat::Vector3 {
    fn from(value: glam::Vec3A) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
