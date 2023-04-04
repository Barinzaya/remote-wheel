use anyhow::{Error as AnyError, Result as AnyResult};
use glam::{Quat, Vec3A};
use serde::Deserialize;

use crate::vmc::{
    avatar::Pose,
    bone::{Bone, Limb},
    device::{ForwardPose, Wheel},
};

mod glue;

#[derive(Debug)]
pub enum Technique {
    Glue(glue::Technique),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "type")]
pub enum TechniqueConfig {
    Glue(glue::TechniqueConfig),
}

impl Technique {
    pub fn pose_forward(&self, wheel: &Wheel, f: impl FnMut(Bone, f32, ForwardPose)) {
        match *self {
            Technique::Glue(ref g) => g.pose_forward(wheel, f),
        }
    }

    pub fn pose_inverse(&self, pose: &Pose, wheel: &Wheel, f: impl FnMut(Limb, f32, Vec3A, Quat)) {
        match *self {
            Technique::Glue(ref g) => g.pose_inverse(pose, wheel, f),
        }
    }

    pub fn update(&mut self, dt: f64, tracking: &Pose) {
        match *self {
            Technique::Glue(ref mut g) => g.update(dt, tracking),
        }
    }
}

impl Default for TechniqueConfig {
    fn default() -> Self {
        TechniqueConfig::Glue(Default::default())
    }
}

impl TryFrom<TechniqueConfig> for Technique {
    type Error = AnyError;

    fn try_from(config: TechniqueConfig) -> AnyResult<Self> {
        match config {
            TechniqueConfig::Glue(glue) => Ok(Technique::Glue(glue.into())),
        }
    }
}
