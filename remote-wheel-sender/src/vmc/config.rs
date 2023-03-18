use std::net::{Ipv4Addr, SocketAddr};

use hashbrown::HashMap;
use linear_map::LinearMap;
use serde::Deserialize;
use string_cache::DefaultAtom;

use super::device::Device;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    enabled: bool,
    pub(super) report_interval: Option<f64>,

    pub(super) input: InputConfig,
    pub(super) output: OutputConfig,

    pub(super) device: HashMap<DefaultAtom, Device>,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct InputConfig {
    pub(super) address: SocketAddr,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct OutputConfig {
    pub(super) address: SocketAddr,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct AxisOutputConfig {
    pub(super) on_update: EventConfig<[f32; 2]>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ButtonOutputConfig {
    #[serde(default)]
    pub(super) on_update: EventConfig<[f32; 2]>,

    #[serde(default)]
    pub(super) on_press: EventConfig<f32>,

    #[serde(default)]
    pub(super) on_release: EventConfig<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct EventConfig<T> {
    pub(super) blendshape: LinearMap<DefaultAtom, T>,
    pub(super) device: LinearMap<DefaultAtom, T>,
}

impl Config {
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            enabled: false,
            input: InputConfig::default(),
            output: OutputConfig::default(),
            report_interval: Some(60.0),

            device: HashMap::new(),
        }
    }
}

impl Default for InputConfig {
    fn default() -> InputConfig {
        InputConfig {
            address: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 3332),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> OutputConfig {
        OutputConfig {
            address: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 3333),
        }
    }
}
