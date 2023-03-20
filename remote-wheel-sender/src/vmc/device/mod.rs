use anyhow::{Error as AnyError, Result as AnyResult};
use glam::{Quat, Vec3, Vec3A};
use serde::Deserialize;
use string_cache::DefaultAtom;

use super::bone::Bone;

mod wheel;
pub use wheel::{Wheel, WheelConfig};

#[derive(Debug, Deserialize)]
#[serde(try_from = "DeviceConfig")]
pub enum Device {
    Wheel(Wheel),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "type")]
enum DeviceConfig {
    Wheel(WheelConfig),
}

impl Device {
    pub fn pose(&self, hand: Bone) -> Option<(Vec3, Quat)> {
        match *self {
            Device::Wheel(ref w) => w.pose(hand),
        }
    }

    pub fn set_value(&mut self, value: f32) {
        match *self {
            Device::Wheel(ref mut w) => w.set_value(value),
        }
    }

    pub fn trackers(&self, f: impl FnMut(DefaultAtom, Vec3A, Quat)) {
        match *self {
            Device::Wheel(ref w) => w.trackers(f),
        }
    }
}

impl TryFrom<DeviceConfig> for Device {
    type Error = AnyError;

    fn try_from(config: DeviceConfig) -> AnyResult<Self> {
        match config {
            DeviceConfig::Wheel(w) => Ok(Device::Wheel(w.try_into()?)),
        }
    }
}
