use anyhow::{ensure, Error as AnyError, Result as AnyResult};
use glam::{EulerRot, Quat, Vec3, Vec3A};
use serde::Deserialize;
use std::f32::consts::TAU;

use crate::vmc::bone::Bone;

#[derive(Debug)]
pub struct Wheel {
    pos: Vec3A,
    rot: Quat,
    radius: f32,

    angle: f32,
    base_rot: Quat,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct WheelConfig {
    position: [f32; 3],
    rotation: [f32; 3],
    radius: f32,
}

impl TryFrom<WheelConfig> for Wheel {
    type Error = AnyError;

    fn try_from(config: WheelConfig) -> AnyResult<Self> {
        ensure!(config.radius > 0.0, "Wheel radius must be positive.");

        let rot = Quat::from_euler(
            EulerRot::YXZ,
            config.rotation[1].to_radians(),
            config.rotation[0].to_radians(),
            config.rotation[2].to_radians(),
        );

        Ok(Wheel {
            pos: config.position.into(),
            radius: config.radius,
            rot,

            angle: 0.0,
            base_rot: rot,
        })
    }
}

impl Default for WheelConfig {
    fn default() -> WheelConfig {
        WheelConfig {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            radius: 0.17,
        }
    }
}

impl Wheel {
    pub fn pose(&self, bone: Bone) -> Option<(Vec3, Quat)> {
        match bone {
            Bone::LeftHand => {
                let grab_pos = self.pos + self.rot * Vec3A::NEG_X * self.radius;
                Some((
                    grab_pos.into(),
                    self.rot * Quat::from_euler(EulerRot::YXZ, 0.25 * TAU, -0.25 * TAU, 0.0),
                ))
            }

            Bone::RightHand => {
                let grab_pos = self.pos + self.rot * Vec3A::X * self.radius;
                Some((
                    grab_pos.into(),
                    self.rot * Quat::from_euler(EulerRot::YXZ, -0.25 * TAU, -0.25 * TAU, 0.0),
                ))
            }

            _ => None,
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.angle = value;
        self.rot = self.base_rot * Quat::from_rotation_z(-value.to_radians());
    }
}
