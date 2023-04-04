use anyhow::{ensure, Context as _, Error as AnyError, Result as AnyResult};
use glam::{EulerRot, Quat, Vec3A};
use serde::Deserialize;
use string_cache::DefaultAtom;

mod technique;

use crate::vmc::{avatar::Pose, bone::Bone};
use technique::{Technique, TechniqueConfig};

#[derive(Debug)]
pub struct Wheel {
    pos: Vec3A,
    rot: Quat,
    radius: f32,

    angle: f32,
    base_rot: Quat,
    tracker: Option<DefaultAtom>,

    technique: Technique,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct WheelConfig {
    position: [f32; 3],
    rotation: [f32; 3],
    radius: f32,
    tracker: Option<DefaultAtom>,
    technique: TechniqueConfig,
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
            tracker: config.tracker,

            technique: config
                .technique
                .try_into()
                .context("Failed to initialize driving wheel handling technique")?,
        })
    }
}

impl Default for WheelConfig {
    fn default() -> WheelConfig {
        WheelConfig {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            radius: 0.17,
            tracker: None,
            technique: TechniqueConfig::default(),
        }
    }
}

impl Wheel {
    pub fn pose(&self, bone: Bone) -> Option<(Vec3A, Quat)> {
        self.technique.pose(bone, self)
    }

    pub fn set_value(&mut self, value: f32) {
        self.angle = value;
        self.rot = self.base_rot * Quat::from_rotation_z(-value.to_radians());
    }

    pub fn trackers(&self, mut f: impl FnMut(DefaultAtom, Vec3A, Quat)) {
        if let Some(ref tracker) = self.tracker.clone() {
            f(tracker.clone(), self.pos, self.rot);
        }
    }

    pub fn update(&mut self, dt: f64, tracking: &Pose) {
        self.technique.update(dt, tracking)
    }
}
