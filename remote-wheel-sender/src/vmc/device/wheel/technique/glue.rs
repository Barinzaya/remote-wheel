use std::f32::consts::TAU;

use glam::{EulerRot, Quat, Vec2, Vec3A};
use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer,
};

use crate::vmc::{avatar::Pose, bone::Bone, device::Wheel};

#[derive(Debug)]
pub struct Technique {
    left_hand_angle: f32,
    right_hand_angle: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct TechniqueConfig {
    #[serde(default = "left_hand_default", deserialize_with = "parse_position")]
    left_hand: f32,

    #[serde(default = "right_hand_default", deserialize_with = "parse_position")]
    right_hand: f32,
}

fn left_hand_default() -> f32 {
    0.5 * TAU
}

fn right_hand_default() -> f32 {
    0.0 * TAU
}

impl Technique {
    pub fn pose(&self, bone: Bone, wheel: &Wheel) -> Option<(Vec3A, Quat)> {
        let (pos, yaw) = match bone {
            Bone::LeftHand => (self.left_hand_angle, 0.25 * TAU),
            Bone::RightHand => (self.right_hand_angle, -0.25 * TAU),
            _ => return None,
        };

        let local_pos = wheel.radius * Vec3A::from((Vec2::from_angle(pos), 0.0));
        let local_rot = Quat::from_euler(EulerRot::YXZ, yaw, yaw - yaw.signum() * pos, 0.0);

        Some((wheel.pos + wheel.rot * local_pos, wheel.rot * local_rot))
    }

    pub fn update(&mut self, dt: f64, tracking: &Pose) {
        let _ = (dt, tracking);
    }
}

fn parse_position<'de, D: Deserializer<'de>>(de: D) -> Result<f32, D::Error> {
    let raw = f32::deserialize(de)?;
    (0.0..=360.0)
        .contains(&raw)
        .then(|| f32::to_radians(90.0 - raw).rem_euclid(TAU))
        .ok_or_else(|| {
            D::Error::invalid_value(
                Unexpected::Float(raw as f64),
                &"an angle in the range [0, 360]",
            )
        })
}

impl From<TechniqueConfig> for Technique {
    fn from(config: TechniqueConfig) -> Self {
        Technique {
            left_hand_angle: config.left_hand,
            right_hand_angle: config.right_hand,
        }
    }
}

impl Default for TechniqueConfig {
    fn default() -> Self {
        TechniqueConfig {
            left_hand: 0.5 * TAU,
            right_hand: 0.0 * TAU,
        }
    }
}
