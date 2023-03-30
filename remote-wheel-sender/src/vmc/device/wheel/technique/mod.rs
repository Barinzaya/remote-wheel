use anyhow::{Error as AnyError, Result as AnyResult};
use glam::{Quat, Vec3A};
use serde::Deserialize;

use crate::vmc::{bone::Bone, device::Wheel, TrackingData};

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
    pub fn pose(&self, bone: Bone, wheel: &Wheel) -> Option<(Vec3A, Quat)> {
        match *self {
            Technique::Glue(ref g) => g.pose(bone, wheel),
        }
    }

    pub fn update(&mut self, dt: f64, tracking: &TrackingData) {
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
